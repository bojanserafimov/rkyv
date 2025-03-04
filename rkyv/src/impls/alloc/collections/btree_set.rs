#[cfg(not(feature = "std"))]
use alloc::collections::BTreeSet;
use core::ops::ControlFlow;
#[cfg(feature = "std")]
use std::collections::BTreeSet;

use rancor::{Fallible, Source};

use crate::{
    collections::btree_set::{ArchivedBTreeSet, BTreeSetResolver},
    ser::{Allocator, Writer},
    Archive, Deserialize, Place, Serialize,
};

impl<K: Archive + Ord> Archive for BTreeSet<K>
where
    K::Archived: Ord,
{
    type Archived = ArchivedBTreeSet<K::Archived>;
    type Resolver = BTreeSetResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedBTreeSet::<K::Archived>::resolve_from_len(
            self.len(),
            resolver,
            out,
        );
    }
}

impl<K, S> Serialize<S> for BTreeSet<K>
where
    K: Serialize<S> + Ord,
    K::Archived: Ord,
    S: Fallible + Allocator + Writer + ?Sized,
    S::Error: Source,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        Self::Archived::serialize_from_ordered_iter(self.iter(), serializer)
    }
}

impl<K, D> Deserialize<BTreeSet<K>, D> for ArchivedBTreeSet<K::Archived>
where
    K: Archive + Ord,
    K::Archived: Deserialize<K, D> + Ord,
    D: Fallible + ?Sized,
{
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> Result<BTreeSet<K>, D::Error> {
        let mut result = BTreeSet::new();
        let r = self.visit(|ak| {
            let k = match ak.deserialize(deserializer) {
                Ok(k) => k,
                Err(e) => return ControlFlow::Break(e),
            };
            result.insert(k);
            ControlFlow::Continue(())
        });
        match r {
            Some(e) => Err(e),
            None => Ok(result),
        }
    }
}

impl<K, AK: PartialEq<K>> PartialEq<BTreeSet<K>> for ArchivedBTreeSet<AK> {
    fn eq(&self, other: &BTreeSet<K>) -> bool {
        if self.len() != other.len() {
            false
        } else {
            let mut iter = other.iter();
            self.visit(|ak| {
                if let Some(k) = iter.next() {
                    if ak.eq(k) {
                        return ControlFlow::Continue(());
                    }
                }
                ControlFlow::Break(())
            })
            .is_none()
        }
    }
}

impl<K, AK: PartialEq<K>> PartialEq<ArchivedBTreeSet<AK>> for BTreeSet<K> {
    fn eq(&self, other: &ArchivedBTreeSet<AK>) -> bool {
        other.eq(self)
    }
}
