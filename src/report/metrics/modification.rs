//! Modification metrics tracking for S3 objects.
use rusoto_s3::Object;

use super::Metric;
use crate::report::bounded::{self, Bounded};
use crate::report::util;

/// Container struct for modificaton metrics tracked by S3.
pub struct Modification {
    earliest_file: Bounded<String>,
    latest_file: Bounded<String>,
}

/// Main implementation.
impl Modification {
    /// Constructs a new `Modification` struct.
    pub(super) fn new() -> Modification {
        Modification {
            latest_file: Bounded::new("".into()),
            earliest_file: Bounded::new("".into()),
        }
    }
}

/// Metric implementation.
impl Metric for Modification {
    /// Registers an S3 `Object` with this metric struct.
    fn register(&mut self, object: &Object) {
        bounded::apply(
            &mut self.earliest_file,
            &mut self.latest_file,
            super::get_key(object),
            super::get_modified(object),
        );
    }

    /// Prints out all internal statistics under the `modification` header.
    fn print(&self) {
        // next segment: modification
        util::log_head("modification");

        // log out the bounds of the earliest file
        util::log_bound("earliest_file", &self.earliest_file, |date| {
            util::log_pair("earliest_file_date", date);
        });

        // log out the bounds of the latest file
        util::log_bound("latest_file", &self.latest_file, |date| {
            util::log_pair("latest_file_date", date);
        });
    }
}
