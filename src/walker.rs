//! Common object traversal structures for AWS S3.
//!
//! This module doesn't contain anything special beyond a pseudo-iterator
//! to walk over objects in S3 in a more idiomatic manner. At some point
//! (hopefully soon) this will change to use an asynchronous `Stream`,
//! when Rusoto migrates to Futures 0.3 and beyond.
use crate::types::UtilResult;
use rusoto_s3::*;
use std::future::Future;
use std::pin::Pin;

/// Pseudo `Iterator` structure to walk over `Object` types in AWS S3.
///
/// As this is a fallible iteration, a `for` style loop cannot be used
/// easily. Instead, this pattern must be used:
///
/// ```rust
/// let walker = ObjectWalker::new(...);
///
/// while let Some(object) = walker.next()? {
///     // do something...
/// }
/// ```
///
/// Even though this isn't as convenient as `for`, it's still much
/// cleaner than manually iterating the S3 object pages.
pub struct ObjectWalker<'a> {
    s3: &'a S3Client,
    token: Option<String>,
    bucket: String,
    prefix: Option<String>,
    buffer: Vec<Object>,
    finished: bool,
}

impl<'a> ObjectWalker<'a> {
    /// Construct a new `ObjectWalker` for a bucket/prefix pair.
    pub fn new(s3: &'a S3Client, bucket: String, prefix: Option<String>) -> Self {
        Self {
            s3,
            bucket,
            prefix,
            token: None,
            buffer: Vec::new(),
            finished: false,
        }
    }

    /// Attempts to fetch the next `Object` in the S3 archives.
    ///
    /// Calls can fail, which is why a `Result` is returned. Even if a call
    /// succeeds there is no guarantee an `Object` exists, which is why an
    /// `Option` is returned.
    ///
    /// Calling this method does not guarantee a call will be made to AWS;
    /// there may already be buffered data to be returned immediately.
    pub fn next(&mut self) -> Pin<Box<dyn Future<Output = UtilResult<Option<Object>>> + '_>> {
        Box::pin(async move {
            // always check the buffer first
            if !self.buffer.is_empty() {
                return Ok(Some(self.buffer.remove(0)));
            }

            // if done, no fetch
            if self.finished {
                return Ok(None);
            }

            // create a request to list objects
            let request = ListObjectsV2Request {
                bucket: self.bucket.clone(),
                prefix: self.prefix.clone(),
                continuation_token: self.token.clone(),
                ..ListObjectsV2Request::default()
            };

            // execute the request and await the response (blocking)
            let response = self.s3.list_objects_v2(request).await?;

            // check contents (although should always be there)
            if response.contents.is_none() {
                return Ok(None);
            }

            // store the page and next identifier
            self.buffer = response.contents.unwrap();
            self.token = response.next_continuation_token;

            // check for last page
            if self.token == None {
                self.finished = true;
            }

            // pass back
            self.next().await
        })
    }
}
