//! Gather metadata about your S3 buckets.
//!
//! This utility can be used to generate a report about the provided
//! S3 bucket, including things like file sizes, modification dates, etc.
use clap::{App, ArgMatches, SubCommand};
use rusoto_s3::*;

use crate::cli;
use crate::types::UtilResult;
use crate::walker::ObjectWalker;

pub mod bounded;
pub mod metrics;
pub mod util;

/// Generates an appropriate `SubCommand` for this module.
pub fn cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("report")
        .about("Gather metadata about your S3 buckets")
        .args(&cli::global_args())
}

/// Executes this subcommand and returns a `UtilResult` to indicate success.
pub async fn exec(s3: S3Client, args: &ArgMatches<'_>) -> UtilResult<()> {
    // parse all global arguments
    let (bucket, prefix) = cli::get_bucket_pair(args);

    // create our set of metric meters
    let mut chain = metrics::chain(&prefix);
    let mut walker = ObjectWalker::new(&s3, bucket, prefix);

    // walk and check all metrics
    while let Some(object) = walker.next().await? {
        // iterate all metrics meters
        for metric in &mut chain {
            metric.register(&object);
        }
    }

    // print all statistics
    for metric in &chain {
        metric.print();
    }

    // done
    Ok(())
}
