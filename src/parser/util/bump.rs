use std::fmt::Debug;

use bumpalo::{
    boxed::Box as BBox, collections::string::String as BString, collections::vec::Vec as BVec, Bump,
};

pub type BumpString<'a> = BumpWrapper<'a, BString<'a>>;
pub type BumpBox<'a, T> = BumpWrapper<'a, BBox<'a, T>>;
pub type BumpVec<'a, T> = BumpWrapper<'a, BVec<'a, T>>;

pub struct BumpWrapper<'a, T> {
    pub(crate) data: T,
    pub(crate) bump: &'a Bump,
}

impl<'a, T> IntoIterator for BumpVec<'a, T> {
    type Item = T;
    type IntoIter = <BVec<'a, T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T> BumpVec<'a, T> {
    pub fn new_in(bump: &'a Bump) -> Self {
        BumpVec {
            bump,
            data: BVec::new_in(bump),
        }
    }

    pub fn append(&mut self, item: &mut BumpVec<'a, T>) {
        self.data.append(&mut item.data)
    }

    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    pub fn unary_in(el: T, bump: &'a Bump) -> Self {
        BumpVec {
            bump,
            data: bumpalo::vec![in &bump; el],
        }
    }

    pub fn binary_in(el1: T, el2: T, bump: &'a Bump) -> Self {
        BumpVec {
            bump,
            data: bumpalo::vec![in &bump; el1, el2],
        }
    }

    pub fn from_iterator<I: Iterator<Item = T>>(iter: I, bump: &'a Bump) -> Self {
        BumpVec {
            data: BVec::from_iter_in(iter, bump),
            bump,
        }
    }
}

impl<'a, T> BumpBox<'a, T> {
    pub fn new_in(data: T, bump: &'a Bump) -> Self {
        BumpBox {
            bump,
            data: BBox::new_in(data, bump),
        }
    }

    pub fn clone_inner(&self) -> T
    where
        T: Clone,
    {
        self.data.clone()
    }
}

impl<'a> BumpString<'a> {
    pub fn to_string(self) -> String {
        String::from(self.data.as_str())
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn new_in(bump: &'a Bump) -> Self {
        BumpString {
            data: BString::new_in(bump),
            bump,
        }
    }

    pub fn from_str(data: &str, bump: &'a Bump) -> Self {
        BumpString {
            data: {
                let mut buf = BString::with_capacity_in(data.len(), bump);
                buf.push_str(data);
                buf
            },
            bump: bump,
        }
    }
}

impl<'a, T: Clone> Clone for BumpWrapper<'a, BBox<'a, T>> {
    fn clone(&self) -> Self {
        BumpWrapper {
            data: BBox::new_in(self.data.clone(), self.bump),
            bump: self.bump,
        }
    }
}

impl<'a, T: Clone> Clone for BumpWrapper<'a, BVec<'a, T>> {
    fn clone(&self) -> Self {
        BumpWrapper {
            data: {
                let mut data = BVec::with_capacity_in(self.data.len(), self.bump);
                data.extend_from_slice(&self.data);
                data
            },
            bump: self.bump,
        }
    }
}

impl<'a> Clone for BumpWrapper<'a, BString<'a>> {
    fn clone(&self) -> Self {
        BumpWrapper {
            data: {
                let mut buf = BString::with_capacity_in(self.data.len(), self.bump);
                buf.push_str(&self.data);
                buf
            },
            bump: self.bump,
        }
    }
}

impl<'a, T: Debug> Debug for BumpWrapper<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<'a, Rhs, T: PartialEq<Rhs>> PartialEq<BumpWrapper<'a, Rhs>> for BumpWrapper<'a, T> {
    fn eq(&self, data: &BumpWrapper<'a, Rhs>) -> bool {
        self.data.eq(&data.data)
    }
}

impl<'a, U, T: Extend<U>> Extend<U> for BumpWrapper<'a, T> {
    fn extend<V>(&mut self, iter: V)
    where
        V: IntoIterator<Item = U>,
    {
        self.data.extend(iter)
    }
}
