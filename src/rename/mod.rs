//! Dynamic (and remote) file renaming using flexible patterns.
use clap::{App, Arg, ArgMatches, SubCommand};
use regex::Regex;
use rusoto_s3::*;

use crate::cli;
use crate::types::UtilResult;
use crate::walker::ObjectWalker;

/// Generates an appropriate `SubCommand` for this module.
pub fn cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("rename")
        .about("Renaming of files in S3 remotely")
        .args(&cli::global_args())
        .args(&[
            Arg::with_name("source")
                .help("A source pattern to use to locate files")
                .index(2)
                .required(true),
            Arg::with_name("target")
                .help("A target pattern to use to rename files into")
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

    let walker_bucket = bucket.clone();
    let mut walker = ObjectWalker::new(&s3, walker_bucket, prefix);

    // walk across all remote objects
    while let Some(object) = walker.next().await? {
        // unwrap the source key
        let key = object.key.unwrap();

        // skip non-matching files
        if !source.is_match(&key) {
            continue;
        }

        // format the target path
        let full_target = source
            .replace_all(&key, target.to_string().as_str())
            .to_string();

        // don't concat into self
        if full_target == key {
            continue;
        }

        // log out exactly what we're renaming right now
        info!("Renaming {} -> {}", key, full_target);

        // skip
        if dryrun {
            continue;
        }

        // update the target with the prefix
        let source = if key.starts_with(&bucket) {
            key.to_string()
        } else {
            format!("{}/{}", bucket, key)
        };

        // create the copy request
        let copy = CopyObjectRequest {
            key: full_target.to_string(),
            bucket: bucket.to_string(),
            copy_source: source,
            ..CopyObjectRequest::default()
        };

        // execute the copy of the object
        s3.copy_object(copy).await?;

        // log out exactly what we're doing right now
        info!("Removing {} sources...", key);

        // remove the old object after renaming
        let delete = DeleteObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..DeleteObjectRequest::default()
        };

        // execute the delete of the object
        s3.delete_object(delete).await?;
    }

    Ok(())
}
