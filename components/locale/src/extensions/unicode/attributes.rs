use super::Attribute;
use std::ops::Deref;

/// Attributes is a list of attributes (examples: `["foobar", "testing"]`, etc.)
///
/// `Attributes` stores a list of [`Attribute`] subtags in a canonical form
/// by sorting and deduplicating them.
///
/// # Example
/// ```
/// use icu_locale::extensions::unicode::{Attribute, Attributes};
///
/// let attribute1: Attribute = "foobar".parse()
///     .expect("Failed to parse a variant subtag.");
///
/// let attribute2: Attribute = "testing".parse()
///     .expect("Failed to parse a variant subtag.");
/// let mut v = vec![attribute1, attribute2];
/// v.sort();
/// v.dedup();
///
/// let attributes: Attributes = Attributes::from_vec_unchecked(v);
/// assert_eq!(attributes.to_string(), "foobar-testing");
/// ```
///
/// [`Attribute`]: ./struct.Attribute.html
#[derive(Default, Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct Attributes(Box<[Attribute]>);

impl Attributes {
    /// Creates a new `Attributes` set from a vector.
    /// The caller is expected to provide sorted and deduplicated vector as
    /// an input.
    ///
    /// # Example
    /// ```
    /// use icu_locale::extensions::unicode::{Attribute, Attributes};
    ///
    /// let attribute1: Attribute = "foobar".parse()
    ///     .expect("Parsing failed.");
    /// let attribute2: Attribute = "testing".parse()
    ///     .expect("Parsing failed.");
    /// let mut v = vec![attribute1, attribute2];
    /// v.sort();
    /// v.dedup();
    ///
    /// let attributes = Attributes::from_vec_unchecked(v);
    /// ```
    ///
    /// For performance and memory constraint environments, it is recommended
    /// for the caller to use `slice::binary_search` instead of `sort` and `dedup`.
    pub fn from_vec_unchecked(input: Vec<Attribute>) -> Self {
        Self(input.into_boxed_slice())
    }

    /// Empties the `Attributes` list.
    ///
    /// # Example
    /// ```
    /// use icu_locale::extensions::unicode::{Attribute, Attributes};
    ///
    /// let attribute1: Attribute = "foobar".parse()
    ///     .expect("Parsing failed.");
    /// let attribute2: Attribute = "testing".parse()
    ///     .expect("Parsing failed.");
    /// let mut v = vec![attribute1, attribute2];
    /// v.sort();
    /// v.dedup();
    ///
    /// let mut attributes: Attributes = Attributes::from_vec_unchecked(v);
    ///
    /// assert_eq!(attributes.to_string(), "foobar-testing");
    ///
    /// attributes.clear();
    ///
    /// assert_eq!(attributes.to_string(), "");
    /// ```
    pub fn clear(&mut self) {
        self.0 = Box::new([]);
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut initial = true;
        for variant in self.0.iter() {
            if initial {
                initial = false;
            } else {
                f.write_str("-")?;
            }
            variant.fmt(f)?;
        }
        Ok(())
    }
}

impl Deref for Attributes {
    type Target = [Attribute];

    fn deref(&self) -> &[Attribute] {
        &self.0
    }
}
