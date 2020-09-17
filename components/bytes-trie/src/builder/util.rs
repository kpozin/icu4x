pub(crate) trait StrExt {
    fn char_at(&self, index: usize) -> Option<char>;
}

impl<T> StrExt for T
where
    T: std::borrow::Borrow<str>,
{
    fn char_at(&self, index: usize) -> Option<char> {
        let s: &str = self.borrow();
        s.get(index..=index).and_then(|s| s.chars().next())
    }
}
