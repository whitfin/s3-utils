//! File size metrics tracking for S3 objects.
use rusoto_s3::Object;

use super::Metric;
use crate::report::bounded::{self, Bounded};
use crate::report::util;

/// Container struct for file size metrics tracked by S3.
pub struct FileSize {
    total_keys: u64,
    total_space: u64,
    largest_file: Bounded<u64>,
    smallest_file: Bounded<u64>,
}

/// Main implementation.
impl FileSize {
    /// Constructs a new `FileSize` struct.
    pub(super) fn new() -> FileSize {
        FileSize {
            total_keys: 0,
            total_space: 0,
            largest_file: Bounded::new(0),
            smallest_file: Bounded::new(0),
        }
    }
}

/// Metric implementation.
impl Metric for FileSize {
    /// Registers an S3 `Object` with this metric struct.
    fn register(&mut self, object: &Object) {
        // pull various metadata
        let size = super::get_size(object);

        // count another key total
        self.total_keys += 1;
        self.total_space += size;

        // apply bounded updates
        bounded::apply(
            &mut self.smallest_file,
            &mut self.largest_file,
            super::get_key(object),
            &size,
        );
    }

    /// Prints out all internal statistics under the `file_size` header.
    fn print(&self) {
        // get average file size, protect against /0
        let average_file = match self.total_keys {
            0 => 0,
            v => self.total_space / v,
        };

        // next segment: file_size
        util::log_head("file_size");

        // log the average size as both readable and bytes
        util::log_pair("average_file_size", util::convert_bytes(average_file));
        util::log_pair("average_file_bytes", average_file);

        // log out the bounds of the largest file
        util::log_bound("largest_file", &self.largest_file, |size| {
            util::log_pair("largest_file_size", util::convert_bytes(size));
            util::log_pair("largest_file_bytes", size);
        });

        // log out the bounds of the smallest file
        util::log_bound("smallest_file", &self.smallest_file, |size| {
            util::log_pair("smallest_file_size", util::convert_bytes(size));
            util::log_pair("smallest_file_bytes", size);
        });
    }
}
