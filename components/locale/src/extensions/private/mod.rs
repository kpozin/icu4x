mod key;
use std::ops::Deref;

pub use key::Key;

use crate::parser::ParserError;

/// A list of [`Unicode Private Extensions`] as defined in [`Unicode Locale
/// Identifier`] specification.
///
/// Those extensions are intended for `pass-through` use.
///
/// # Examples
///
/// ```
/// use icu_locale::Locale;
/// use icu_locale::extensions::private::Key;
///
/// let mut loc: Locale = "en-US-x-foo-faa".parse()
///     .expect("Parsing failed.");
///
/// let key: Key = "foo".parse().expect("Parsing key failed.");
/// assert_eq!(loc.extensions.private.contains(key), true);
/// assert_eq!(loc.extensions.private.iter().next(), Some(&key)); // tags got sorted
/// loc.extensions.private.clear();
/// assert_eq!(loc.to_string(), "en-US");
/// ```
///
/// [`Unicode Private Extensions`]: https://unicode.org/reports/tr35/#pu_extensions
/// [`Unicode Locale Identifier`]: https://unicode.org/reports/tr35/#Unicode_locale_identifier
#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, PartialOrd, Ord)]
pub struct Private(Box<[Key]>);

impl Private {
    /// Returns `true` if there are no tags in the `Private` Extension List.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_locale::Locale;
    ///
    /// let mut loc: Locale = "en-US-x-foo".parse()
    ///     .expect("Parsing failed.");
    ///
    /// assert_eq!(loc.extensions.private.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn from_vec_unchecked(v: Vec<Key>) -> Self {
        Self(v.into_boxed_slice())
    }

    pub fn contains(&self, v: Key) -> bool {
        self.0.contains(&v)
    }

    pub fn clear(&mut self) {
        self.0 = Box::new([]);
    }

    pub(crate) fn try_from_iter<'a>(
        iter: &mut impl Iterator<Item = &'a [u8]>,
    ) -> Result<Self, ParserError> {
        let keys = iter
            .map(|subtag| Key::from_bytes(subtag))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(keys.into_boxed_slice()))
    }
}

impl std::fmt::Display for Private {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        f.write_str("-x")?;

        for subtag in self.0.iter() {
            write!(f, "-{}", subtag)?;
        }
        Ok(())
    }
}

impl Deref for Private {
    type Target = [Key];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
