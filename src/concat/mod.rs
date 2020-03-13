//! Concatenate Amazon S3 files remotely using flexible patterns.
use clap::{App, Arg, ArgMatches, SubCommand};
use regex::Regex;
use rusoto_s3::*;

use std::collections::{HashMap, HashSet};

use crate::cli;
use crate::types::UtilResult;
use crate::walker::ObjectWalker;

/// Generates an appropriate `SubCommand` for this module.
pub fn cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("concat")
        .about("Concatenate Amazon S3 files remotely using flexible pattern")
        .args(&cli::global_args())
        .args(&[
            Arg::with_name("cleanup")
                .help("Removes source files after concatenation")
                .short("c")
                .long("cleanup"),
            Arg::with_name("source")
                .help("A source pattern to use to locate files")
                .index(2)
                .required(true),
            Arg::with_name("target")
                .help("A target pattern to use to concatenate files into")
                .index(3)
                .required(true),
        ])
}

/// Executes this subcommand and returns a `UtilResult` to indicate success.
pub async fn exec(s3: S3Client, args: &ArgMatches<'_>) -> UtilResult<()> {
    // parse all global arguments
    let dryrun = cli::is_dry_run(args);
    let (bucket, prefix) = cli::get_bucket_pair(args);

    // unwrap and compile the source regex (unwrap should be safe)
    let source = Regex::new(&args.value_of("source").unwrap())?;
    let target = args.value_of("target").unwrap();

    // sources and target -> upload mappings
    let mut sources: HashMap<String, HashSet<String>> = HashMap::new();
    let mut targets: HashMap<String, String> = HashMap::new();

    // walker strings to pass through
    let walker_bucket = bucket.clone();
    let walker_prefix = prefix.clone();

    // construct uploads - this is separate to allow easy handling of errors
    let walker = ObjectWalker::new(&s3, walker_bucket, walker_prefix);
    let result = construct_uploads(
        dryrun,
        &s3,
        source,
        &mut sources,
        &mut targets,
        walker,
        (&bucket, &target),
    );
    let result = result.await;

    // dry doesn't post-process
    if dryrun {
        return Ok(());
    }

    // handle errors
    if result.is_err() {
        // try to abort all requests
        for (key, upload_id) in targets {
            abort_request(
                &s3,
                key.to_string(),
                bucket.to_string(),
                upload_id.to_string(),
            )
            .await;
        }

        // passthrough
        return result;
    }

    // attempt to complete all requests
    for (key, upload_id) in targets {
        // log out to be user friendly...
        info!("Completing {}...", upload_id);

        // create a request to list parts buffer
        let parts = ListPartsRequest {
            key: key.to_string(),
            bucket: bucket.to_string(),
            upload_id: upload_id.to_string(),
            ..ListPartsRequest::default()
        };

        // carry out the request for the parts list
        let parts_result = s3.list_parts(parts).await;

        // attempt to list the pending parts
        if let Err(err) = parts_result {
            // if we can't list the parts, tell the user to help out
            error!("Unable to list pending parts for {}: {}", upload_id, err);

            // gotta abort
            abort_request(
                &s3,
                key.to_string(),
                bucket.to_string(),
                upload_id.to_string(),
            )
            .await;

            // move on
            continue;
        }

        // buffer up all completed parts
        let completed = parts_result
            .unwrap()
            .parts
            .unwrap()
            .into_iter()
            .map(|part| CompletedPart {
                e_tag: part.e_tag,
                part_number: part.part_number,
            })
            .collect();

        // create our multipart completion body
        let multipart = CompletedMultipartUpload {
            parts: Some(completed),
        };

        // create our multipart completion request
        let complete = CompleteMultipartUploadRequest {
            key: key.to_string(),
            bucket: bucket.to_string(),
            upload_id: upload_id.to_string(),
            multipart_upload: Some(multipart),
            ..CompleteMultipartUploadRequest::default()
        };

        // attempt to complete each request, abort on fail (can't short circut)
        if s3.complete_multipart_upload(complete).await.is_err() {
            // remove the upload sources
            sources.remove(&key);

            // abort now!
            abort_request(
                &s3,
                key.to_string(),
                bucket.to_string(),
                upload_id.to_string(),
            )
            .await;
        }
    }

    // only cleanup when explicit
    if !args.is_present("cleanup") {
        return result;
    }

    // iterate all upload sources
    for keys in sources.values() {
        // iterate all concat'ed
        for key in keys {
            // print that we're removing
            info!("Removing {}...", key);

            // create the removal request
            let delete = DeleteObjectRequest {
                key: key.to_string(),
                bucket: bucket.to_string(),
                ..DeleteObjectRequest::default()
            };

            // attemp to remove the objects from S3
            if s3.delete_object(delete).await.is_err() {
                error!("Unable to remove {}", key);
            }
        }
    }

    Ok(())
}

/// Constructs all upload requests based on walking the S3 tree.
///
/// This will populate the provided mappings, as they're using in the main
/// function for error handling (this allows us to use ? in this function).
async fn construct_uploads<'a>(
    dry: bool,
    s3: &S3Client,
    pattern: Regex,
    sources: &mut HashMap<String, HashSet<String>>,
    targets: &mut HashMap<String, String>,
    mut walker: ObjectWalker<'a>,
    mapping: (&str, &str),
) -> UtilResult<()> {
    // unpack the mapping tuple
    let (bucket, target) = mapping;

    // iterate all objects in the remo
    while let Some(object) = walker.next().await? {
        // unwrap the source key
        let key = object.key.unwrap();

        // skip non-matching files
        if !pattern.is_match(&key) {
            continue;
        }

        // AWS doesn't let us concat < 5MB
        if object.size.unwrap() < 5_000_000 {
            return Err(format!("Unable to concat files below 5MB: {}", key).into());
        }

        // format the source path, as well as the target
        let part_source = format!("{}/{}", bucket, key);
        let full_target = pattern
            .replace_all(&key, target.to_string().as_str())
            .to_string();

        // don't concat into self
        if full_target == key {
            continue;
        }

        // log out exactly what we're concatenating right now
        info!("Concatenating {} -> {}", key, full_target);

        // skip
        if dry {
            continue;
        }

        // ensure we have an upload identifier
        if !targets.contains_key(&full_target) {
            // initialize the upload request as needed
            let creation = CreateMultipartUploadRequest {
                bucket: bucket.to_string(),
                key: full_target.to_string(),
                ..CreateMultipartUploadRequest::default()
            };

            // init the request against AWS, and retrieve the identifier
            let created = s3.create_multipart_upload(creation).await?;
            let upload = created.upload_id.expect("upload id should exist");

            // insert the upload identifier against the target
            targets.insert(full_target.clone(), upload.clone());
            sources.insert(upload, HashSet::new());
        };

        // retrieve the upload identifier for the target
        let upload_id = targets
            .get(&full_target)
            .expect("upload identifier should always be mapped");

        // retrieve the sources list for the upload_id
        let sources = sources.get_mut(&*upload_id).unwrap();

        // create the copy request for the existing key
        let copy_request = UploadPartCopyRequest {
            bucket: bucket.to_string(),
            copy_source: part_source,
            part_number: (sources.len() + 1) as i64,
            key: full_target,
            upload_id: upload_id.to_string(),
            ..UploadPartCopyRequest::default()
        };

        // carry out the request for the part copy
        s3.upload_part_copy(copy_request).await?;

        // push the source for removal
        sources.insert(key);
    }

    // happy
    Ok(())
}

/// Aborts a multipart request in S3 by upload_id.
///
/// This can be used to abort a failed upload request, due to either the inability
/// to construct the part request, or the inability to complete the multi request.
async fn abort_request(s3: &S3Client, key: String, bucket: String, upload_id: String) {
    // print that it's being aborted
    error!("Aborting {}...", upload_id);

    // create the main abort request
    let abort = AbortMultipartUploadRequest {
        key: key.to_string(),
        bucket: bucket.to_string(),
        upload_id: upload_id.to_string(),
        ..AbortMultipartUploadRequest::default()
    };

    // attempt to abort each request, log on fail (can't short circut)
    if s3.abort_multipart_upload(abort).await.is_err() {
        error!("Unable to abort: {}", upload_id);
    }
}
