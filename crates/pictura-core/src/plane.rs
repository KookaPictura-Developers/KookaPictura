//! Refcounted copy-on-write pixel planes.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// One image plane behind a refcount.
///
/// Cloning the owning document is a refcount bump, and a write forks only the
/// plane it touches, so a stroke or a history snapshot stops paying for pixels
/// that have not changed. A plane that is uniquely held keeps its allocation
/// and mutates in place exactly as a `Vec` did; only a shared plane copies on
/// first write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plane<T = u8>(Arc<[T]>);

impl<T> Deref for Plane<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Clone> DerefMut for Plane<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        Arc::make_mut(&mut self.0)
    }
}

impl<T> Default for Plane<T> {
    fn default() -> Self {
        Self(Arc::from(Vec::new()))
    }
}

impl<T> From<Vec<T>> for Plane<T> {
    fn from(data: Vec<T>) -> Self {
        Self(Arc::from(data))
    }
}

impl<T> Plane<T> {
    /// The plane's samples, exactly as a `Vec::as_slice` would hand them over.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}

impl<T: Clone> Plane<T> {
    /// The plane's samples, mutable; a shared plane forks first.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    /// Append `more`, mirroring `Vec::extend_from_slice`.
    ///
    /// ponytail: a slice behind a refcount cannot grow in place, so this
    /// rebuilds; the callers build small buffers.
    pub fn extend_from_slice(&mut self, more: &[T])
    where
        T: Clone,
    {
        let mut data = self.0.to_vec();
        data.extend_from_slice(more);
        self.0 = Arc::from(data);
    }

    /// Extend from an iterator, mirroring `Vec::extend`.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I)
    where
        T: Clone,
    {
        let mut data = self.0.to_vec();
        data.extend(iter);
        self.0 = Arc::from(data);
    }

    /// Drop every sample, mirroring `Vec::clear`.
    pub fn clear(&mut self) {
        self.0 = Arc::from(Vec::new());
    }

    /// Shorten the plane to `len`, mirroring `Vec::truncate`.
    ///
    /// ponytail: a slice behind a refcount cannot shrink in place, so this
    /// rebuilds; the callers are the tests that build malformed buffers.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            let mut data = self.0.to_vec();
            data.truncate(len);
            self.0 = Arc::from(data);
        }
    }
}

impl<T> AsRef<[T]> for Plane<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

/// Whether two planes share one allocation; the tests use it to show that a
/// clone is a refcount bump and that a write forks.
#[cfg(test)]
pub(crate) fn shares(a: &Plane<u8>, b: &Plane<u8>) -> bool {
    Arc::ptr_eq(&a.0, &b.0)
}

impl<'a, T> IntoIterator for &'a Plane<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: PartialEq> PartialEq<Vec<T>> for Plane<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.as_ref() == other.as_slice()
    }
}

impl<T: PartialEq> PartialEq<[T]> for Plane<T> {
    fn eq(&self, other: &[T]) -> bool {
        self.as_ref() == other
    }
}

impl<T: PartialEq, const N: usize> PartialEq<[T; N]> for Plane<T> {
    fn eq(&self, other: &[T; N]) -> bool {
        self.as_ref() == other
    }
}

impl<T: PartialEq> PartialEq<&[T]> for Plane<T> {
    fn eq(&self, other: &&[T]) -> bool {
        self.as_ref() == *other
    }
}

impl<T> std::iter::FromIterator<T> for Plane<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(Arc::from(iter.into_iter().collect::<Vec<T>>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clone_shares_and_a_write_forks() {
        let mut a: Plane<u8> = vec![1, 2, 3, 4].into();
        let b = a.clone();
        assert!(shares(&a, &b), "a clone must only bump the refcount");
        assert_eq!(a, b);

        a[0] = 9;
        assert!(!shares(&a, &b), "a write must fork the plane");
        assert_eq!(a, vec![9, 2, 3, 4]);
        assert_eq!(b, vec![1, 2, 3, 4], "the sharer keeps the old bytes");
    }

    #[test]
    fn a_unique_plane_writes_in_place() {
        let mut a: Plane<u8> = vec![1, 2, 3, 4].into();
        a.truncate(2);
        a.fill(7);
        assert_eq!(a, vec![7, 7]);
        assert_eq!(a.as_ref(), &[7u8, 7]);
    }

    #[test]
    fn planes_compare_their_bytes_against_every_operand_the_tests_use() {
        let p: Plane<u8> = vec![1, 2, 3].into();
        assert_eq!(p, vec![1, 2, 3]);
        assert_eq!(p, [1u8, 2, 3][..]);
        assert_eq!(p, &[1u8, 2, 3][..]);
        assert_eq!(&p, &[1u8, 2, 3][..]);
        assert_eq!(p, p.clone());
        assert_ne!(p, vec![1, 2]);
    }

    #[test]
    fn an_empty_plane_is_default_and_deref_coerces() {
        let p: Plane<u8> = Plane::default();
        assert_eq!(p.len(), 0);
        let s: &[u8] = &p;
        assert!(s.is_empty());
        let o: Option<Plane<u8>> = Some(vec![5u8].into());
        let d: Option<&[u8]> = o.as_deref();
        assert_eq!(d, Some(&[5u8][..]));
    }
}
