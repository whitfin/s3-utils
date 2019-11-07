//! Logging module for terminal based output control.
//!
//! Contains a custom logging implementation to disable/redirect output
//! based on command line switches baked into the application level.
use clap::ArgMatches;
use logger::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

/// Basic logger instance to allow quiet-aware logging.
struct BasicLogger {
    quiet: bool,
}

// Basic logging implementation.
impl Log for BasicLogger {
    /// Returns enabled only for s3-concat modules.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("s3_utils")
    }

    /// Logs out a `Record` when logging is enabled.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.metadata().level() == Level::Error {
                eprintln!("{}", record.args());
            } else if !self.quiet {
                println!("{}", record.args());
            }
        }
    }

    /// Flushes this logger.
    fn flush(&self) {}
}

/// Initializes the logger based on the provided arguments.
///
/// If the `-q` flag was provided, this short circuits to cull all logging.
pub fn init(args: &ArgMatches) -> Result<(), SetLoggerError> {
    let logger = Box::new(BasicLogger {
        quiet: args.is_present("quiet"),
    });
    log::set_boxed_logger(logger).map(|_| log::set_max_level(LevelFilter::Info))
}
