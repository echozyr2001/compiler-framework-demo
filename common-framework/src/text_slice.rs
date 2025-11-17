use std::ops::Deref;
use std::sync::Arc;

/// Immutable slice referencing a shared text buffer.
///
/// The slice keeps an `Arc<str>` alive so that it can be freely cloned and
/// moved around without worrying about lifetimes. It implements `Deref<Target =
/// str>` which allows it to be used transparently as `&str` in most places.
#[derive(Clone, Debug)]
pub struct TextSlice {
    buffer: Arc<str>,
    start: usize,
    end: usize,
}

impl TextSlice {
    /// Creates a new slice from the given shared buffer and byte range.
    pub fn new(buffer: Arc<str>, start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        debug_assert!(end <= buffer.len());
        Self { buffer, start, end }
    }

    /// Creates a slice that covers the entire buffer.
    pub fn from_arc(buffer: Arc<str>) -> Self {
        let end = buffer.len();
        Self {
            buffer,
            start: 0,
            end,
        }
    }

    /// Returns the length in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns `true` if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the underlying shared buffer.
    pub fn buffer(&self) -> Arc<str> {
        Arc::clone(&self.buffer)
    }

    /// Returns the start offset.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end offset.
    pub fn end(&self) -> usize {
        self.end
    }
}

impl std::fmt::Display for TextSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl Deref for TextSlice {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.buffer[self.start..self.end]
    }
}

impl AsRef<str> for TextSlice {
    fn as_ref(&self) -> &str {
        self
    }
}

impl PartialEq<&str> for TextSlice {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

impl PartialEq<TextSlice> for &str {
    fn eq(&self, other: &TextSlice) -> bool {
        *self == other.deref()
    }
}

impl PartialEq for TextSlice {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.end == other.end
            && Arc::ptr_eq(&self.buffer, &other.buffer)
            && self.deref() == other.deref()
    }
}

impl Eq for TextSlice {}
