use {icu_locale::Locale, linked_hash_map::LinkedHashMap, ordered_float::NotNan, std::hash::Hash};

const MIN_WEIGHT: f64 = 0.0;
const MAX_WEIGHT: f64 = 1.0;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum PriorityListEntries {
    /// All weights are 1.0
    Unweighted(Vec<Locale>),
    /// Weights are stored with the locale
    Weighted(Vec<(Locale, NotNan<f64>)>),
}

impl Default for PriorityListEntries {
    fn default() -> Self {
        PriorityListEntries::Unweighted(vec![])
    }
}

/// An immutable list of locales in descending priority order.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LocalePriorityList {
    entries: PriorityListEntries,
}

impl LocalePriorityList {
    /// Creates a new, empty builder.
    pub fn builder() -> LocalePriorityListBuilder {
        LocalePriorityListBuilder::new()
    }

    /// Returns an iterator over the list's locales, in descending priority order.
    pub fn locales<'a>(&'a self) -> Box<dyn Iterator<Item = &Locale> + 'a> {
        match &self.entries {
            PriorityListEntries::Unweighted(locales) => Box::new(locales.iter()),
            PriorityListEntries::Weighted(entries) => {
                Box::new(entries.iter().map(|(locale, _)| locale))
            }
        }
    }

    /// Returns the weight for the given locale, or `None` if the given locale is not in the list.
    ///
    /// Only exact equality is considered; no canonicalization or complicated comparison is used.
    ///
    /// Note that in the current implementation, this does a linear search over the locales in the
    /// list, on the assumption that in practice most lists will be short enough that the  memory
    /// overhead of an index would provide no practical advantage in speed.
    pub fn get_weight(&self, locale: &Locale) -> Option<f64> {
        match &self.entries {
            PriorityListEntries::Unweighted(locales) => {
                locales.iter().find(|x| *x == locale).map(|_| MAX_WEIGHT)
            }
            PriorityListEntries::Weighted(entries) => entries
                .iter()
                .find(|(x, _)| x == locale)
                .map(|(_, weight)| (*weight).into()),
        }
    }

    /// Returns the number of locales in the list.
    pub fn len(&self) -> usize {
        match &self.entries {
            PriorityListEntries::Unweighted(x) => x.len(),
            PriorityListEntries::Weighted(x) => x.len(),
        }
    }
}

impl IntoIterator for LocalePriorityList {
    type Item = (Locale, f64);
    type IntoIter = Box<dyn Iterator<Item = <Self as IntoIterator>::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self.entries {
            PriorityListEntries::Unweighted(locales) => {
                Box::new(locales.into_iter().map(|locale| (locale, MAX_WEIGHT)))
            }
            PriorityListEntries::Weighted(entries) => Box::new(
                entries
                    .into_iter()
                    .map(|(locale, weight)| (locale, weight.into())),
            ),
        }
    }
}

impl Into<LocalePriorityListBuilder> for LocalePriorityList {
    fn into(self) -> LocalePriorityListBuilder {
        let mut builder = LocalePriorityListBuilder::new();
        for (locale, weight) in self {
            builder.add_with_weight(locale, weight);
        }
        builder
    }
}

#[derive(Debug, Default, Clone)]
pub struct LocalePriorityListBuilder {
    entries: LinkedHashMap<Locale, f64>,
    has_weights: bool,
}

impl LocalePriorityListBuilder {
    /// Creates a new, empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a locale with the given weight.
    ///
    /// If the same locale is already present in the builder, it is overwritten with the new value.
    /// Weights are clamped to the half-open range `(0.0, 1.0]`.
    /// If the weight is undefined or less than or equal to 0, the locale is removed from the list.
    /// If the weight is greater than 1.0, it is clamped to 1.0.
    pub fn add_with_weight(&mut self, locale: Locale, weight: f64) -> &mut Self {
        // NaN or zero weight means remove the locale if already present.
        if weight.is_nan() || weight <= MIN_WEIGHT {
            self.entries.remove(&locale);
        } else if weight >= MAX_WEIGHT {
            self.entries.insert(locale, MAX_WEIGHT);
        } else {
            self.entries.insert(locale, weight);
            self.has_weights = true;
        }
        self
    }

    /// Adds a locale with a weight of `1.0`.
    pub fn add(&mut self, locale: Locale) -> &mut Self {
        self.add_with_weight(locale, MAX_WEIGHT)
    }

    /// Consumes the builder to create an immutable locale priority list, using the given weights
    /// to sort the locales in descending order and preserving the weights in the final list.
    pub fn build_with_weights(self) -> LocalePriorityList {
        self.build(true)
    }

    /// Consumes the builder to create an immutable locale priority list, using the given weights
    /// to sort the locales in descending order, but discarding them in the final list. After the
    /// list is built, all weights are set to 1.0.
    pub fn build_without_weights(self) -> LocalePriorityList {
        self.build(false)
    }

    fn build(self, preserve_weights: bool) -> LocalePriorityList {
        let LocalePriorityListBuilder {
            entries,
            has_weights,
        } = self;
        let mut entries: Vec<(Locale, NotNan<f64>)> = entries
            .into_iter()
            // .unwrap() is safe because non-finite weights are filtered out on insertion.
            .map(|(locale, weight)| (locale, NotNan::new(weight).unwrap()))
            .collect();
        if has_weights {
            // This is a stable sort, so original insertion order is preserved where weights are
            // equal. Negative for descending order.
            entries.sort_by_key(|(_, weight)| -*weight);
        }
        let entries = if preserve_weights {
            PriorityListEntries::Weighted(entries)
        } else {
            PriorityListEntries::Unweighted(entries.into_iter().map(|entry| entry.0).collect())
        };

        LocalePriorityList { entries }
    }
}
