//! CLI bindings for all internal commands and modules.
//!
//! This module focuses on the common CLI bindings required to provide easy
//! APIs and consistency across all other modules. This is where the parent
//! CLI can be found, as well as utilities for fetching common switches and
//! values.
use clap::{App, AppSettings, Arg, ArgMatches};
use rusoto_s3::*;

use crate::types::UtilResult;

/// Constructs a new CLI application using Clap.
///
/// This will register all subcommand modules and embed all metadata. All
/// metadata is fetched dynamically from Cargo and shouldn't require to
/// be updated (ever).
pub fn build<'a, 'b>() -> App<'a, 'b> {
    App::new("")
        .name(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand(crate::concat::cmd())
        .subcommand(crate::rename::cmd())
        .subcommand(crate::report::cmd())
        .settings(&[
            AppSettings::ArgRequiredElseHelp,
            AppSettings::DisableHelpSubcommand,
            AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands,
        ])
}

/// Executes a subcommand based on the parsed arguments from the CLI.
///
/// This will pass a singleton `S3Client` to each submodule to avoid
/// having to construct a client inside each module.
pub async fn exec(s3: S3Client, args: &ArgMatches<'_>) -> UtilResult<()> {
    match args.subcommand() {
        ("concat", Some(subargs)) => crate::concat::exec(s3, subargs).await,
        ("rename", Some(subargs)) => crate::rename::exec(s3, subargs).await,
        ("report", Some(subargs)) => crate::report::exec(s3, subargs).await,
        _ => {
            build().print_help().expect("Unable to log to TTY");
            Ok(())
        }
    }
}

/// Fetches a bucket/prefix pair from the common argument set.
pub fn get_bucket_pair<'a>(args: &'a ArgMatches<'a>) -> (String, Option<String>) {
    // parse the bucket argument
    let mut splitn = args
        .value_of("bucket")
        .unwrap()
        .trim_start_matches("s3://")
        .splitn(2, '/');

    // bucket is required, prefix is optional after `/`
    (
        splitn.next().unwrap().to_string(),
        splitn.next().map(|s| s.trim_end_matches('/').to_string()),
    )
}

/// Fetches the set of global arguments which should be attached on each command.
pub fn global_args<'a, 'b>() -> [Arg<'a, 'b>; 3] {
    [
        Arg::with_name("dry")
            .help("Only print out the calculated writes")
            .short("d")
            .long("dry-run"),
        Arg::with_name("quiet")
            .help("Only prints errors during execution")
            .short("q")
            .long("quiet"),
        Arg::with_name("bucket")
            .help("An S3 bucket prefix to work within")
            .index(1)
            .required(true),
    ]
}

/// Determines if the dry-run switch was provided in this execution.
pub fn is_dry_run(args: &ArgMatches<'_>) -> bool {
    args.is_present("dry")
}
