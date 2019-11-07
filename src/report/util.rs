//! General utility module housing formatting functions.
use pretty_bytes::converter::convert;

use std::fmt::Display;

use super::bounded::Bounded;

/// Converts a byte count to a `String` representation.
pub fn convert_bytes(bytes: u64) -> String {
    convert(bytes as f64).replacen(' ', "", 1)
}

/// Logs out a bounded value, conditionally based on content.
pub fn log_bound<L, T>(label: &str, bounded: &Bounded<T>, logger: L)
where
    L: FnOnce(T) -> (),
    T: Clone,
{
    let bounded_key = bounded.key().clone();

    if bounded_key.is_none() {
        return;
    }

    let bounded_val = bounded.value().clone();
    let bounded_cnt = bounded.count();

    let key = bounded_key.unwrap();

    logger(bounded_val);
    log_pair(&format!("{}_name", label), key);

    if bounded_cnt > 1 {
        log_pair(&format!("{}_others", label), bounded_cnt);
    }
}

/// Logs a header using a common format.
pub fn log_head(label: &str) {
    println!("\n[{}]", label);
}

/// Logs a label/value pair using a common format.
pub fn log_pair<T>(label: &str, val: T)
where
    T: Display,
{
    println!("{}={}", label, val);
}

#[cfg(test)]
mod tests {

    #[test]
    fn converting_bytes_to_string() {
        let bval = 512_u64;
        let kval = bval * 512_u64;
        let mval = kval * 512_u64;
        let gval = mval * 512_u64;
        let tval = gval * 512_u64;
        let pval = tval * 512_u64;

        let bstr = super::convert_bytes(bval);
        let kstr = super::convert_bytes(kval);
        let mstr = super::convert_bytes(mval);
        let gstr = super::convert_bytes(gval);
        let tstr = super::convert_bytes(tval);
        let pstr = super::convert_bytes(pval);

        assert_eq!(bstr, "512B");
        assert_eq!(kstr, "262.14kB");
        assert_eq!(mstr, "134.22MB");
        assert_eq!(gstr, "68.72GB");
        assert_eq!(tstr, "35.18TB");
        assert_eq!(pstr, "18.01PB");
    }
}
