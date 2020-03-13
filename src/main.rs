//! Utilities and tools to provide convenience S3 APIs in a CLI.
//!
//! This tool should be used from a command line and can be used in many
//! different ways; please see the main documentation in the repository.
//!
//! Credentials must be provided via guidelines in the [AWS Documentation]
//! (https://docs.aws.amazon.com/cli/latest/userguide/cli-environment.html).
#[macro_use]
extern crate log as logger;

use rusoto_core::{credential::ChainProvider, region::Region, HttpClient};
use rusoto_s3::*;

use std::time::Duration;

mod cli;
mod log;
mod types;
mod walker;

mod concat;
mod rename;
mod report;

#[tokio::main]
async fn main() -> types::UtilResult<()> {
    // build the CLI and grab all argumentss
    let args = cli::build().get_matches();

    // initialize logging
    log::init(&args)?;

    // create client options
    let client = HttpClient::new()?;
    let region = Region::default();

    // create provided with timeout
    let mut chain = ChainProvider::new();
    chain.set_timeout(Duration::from_millis(500));

    // create the new S3 client
    let s3 = S3Client::new_with(client, chain, region);

    // delegate to the cli mod
    cli::exec(s3, &args).await
}
