//! Parent metric module exposing traits around metrics gathering.
use rusoto_s3::Object;

pub mod extensions;
pub mod file_size;
pub mod general;
pub mod modification;

use self::extensions::Extensions;
use self::file_size::FileSize;
use self::general::General;
use self::modification::Modification;

/// Metric trait to represent a metric tracker for S3.
///
/// Implementing this trait means that the structure can be used to
/// track metrics on objects stored in S3. Object instances will be
/// fed through to `register` on each entry in S3.
pub trait Metric {
    /// Registers an S3 object for statistics.
    fn register(&mut self, object: &Object);

    /// Prints the internal statistics.
    fn print(&self);
}

/// Returns a chain of `Metric` objects in deterministic order.
pub fn chain(prefix: &Option<String>) -> Vec<Box<dyn Metric>> {
    vec![
        Box::new(General::new(prefix)),
        Box::new(FileSize::new()),
        Box::new(Extensions::new()),
        Box::new(Modification::new()),
    ]
}

/// Retrieves the key of an `Object` as a `&String`.
pub fn get_key(object: &Object) -> &str {
    &*unwrap_opt(&object.key, "objects should have a key")
}

/// Retrieves the modification time of an `Object` as a `&String`.
pub fn get_modified(object: &Object) -> &String {
    unwrap_opt(&object.last_modified, "objects should have a modified date")
}

/// Retrieves the size of an `Object` as a `u64`.
pub fn get_size(object: &Object) -> u64 {
    *unwrap_opt(&object.size, "objects should have a size") as u64
}

/// Unwraps an `Option` as a reference using an `expect` label.
fn unwrap_opt<'a, V>(opt: &'a Option<V>, expect: &str) -> &'a V {
    opt.as_ref().expect(expect)
}
