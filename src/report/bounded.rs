//! Module to expose bounded value representation as structures.

/// Bounded structure to represent values which are being used
/// to keep track of a bound. Supports a key/value pair, with
/// a count to keep track of other keys which also fit the bound.
pub struct Bounded<T> {
    key: Option<String>,
    value: T,
    count: usize,
}

/// Bounded impl.
impl<T> Bounded<T> {
    /// Constructs a new `Bounded` struct from a generic value.
    pub fn new(t: T) -> Bounded<T> {
        Bounded {
            key: None,
            value: t,
            count: 0,
        }
    }

    /// Retrieves a reference to the inner key.
    pub fn key(&self) -> &Option<String> {
        &self.key
    }

    /// Retrieves a reference to the inner value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Retrieves the inner count.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Updates this `Bounded` struct with a new key/value.
    ///
    /// This is essentially a reset via mutation, rather than forcing
    /// the caller to create a new `Bounded` instance.
    pub fn update(&mut self, key: &str, t: T) {
        self.key = Some(key.into());
        self.value = t;
        self.count = 1;
    }

    /// Increments the internal count of the matched bound.
    pub fn increment(&mut self) {
        self.count += 1;
    }

    /// Determines if there is no key/value set in this struct.
    pub fn is_unset(&self) -> bool {
        self.count == 0
    }
}

/// Utility function to apply changes to lower/upper bounds based on a comparison.
pub fn apply<T>(lower: &mut Bounded<T>, upper: &mut Bounded<T>, key: &str, val: &T)
where
    T: Clone + Eq + Ord + PartialEq + PartialOrd,
{
    inner_apply(lower, key, val, |left, right| left < right);
    inner_apply(upper, key, val, |left, right| left > right);
}

/// Applies changes for a key/value based on a custom comparator.
///
/// The comparator function is provided as an argument to embed easily into different
/// types of bounds. Both lower and upper bounds are support in a single call to make
/// it more convenient to the caller (to mask away a lot of the same logic).
#[inline]
fn inner_apply<C, T>(bound: &mut Bounded<T>, key: &str, val: &T, cmp: C)
where
    T: Clone + Eq + Ord + PartialEq + PartialOrd,
    C: FnOnce(&T, &T) -> bool,
{
    if val == bound.value() {
        bound.increment();
    } else if cmp(val, bound.value()) || bound.is_unset() {
        bound.update(key, val.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::Bounded;

    #[test]
    fn updating_bounded_values() {
        let mut bounded = Bounded::new(50);

        assert!(bounded.is_unset());
        assert!(bounded.key().is_none());
        assert_eq!(bounded.value(), &50);
        assert_eq!(bounded.count(), 0);

        bounded.update("my_key", 75);

        assert!(!bounded.is_unset());
        assert_eq!(bounded.key(), &Some("my_key".into()));
        assert_eq!(bounded.value(), &75);
        assert_eq!(bounded.count(), 1);

        bounded.increment();

        assert!(!bounded.is_unset());
        assert_eq!(bounded.key(), &Some("my_key".into()));
        assert_eq!(bounded.value(), &75);
        assert_eq!(bounded.count(), 2);
    }
}
