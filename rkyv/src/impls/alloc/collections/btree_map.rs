#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
use core::ops::ControlFlow;
#[cfg(feature = "std")]
use std::collections::BTreeMap;

use rancor::{Fallible, Source};

use crate::{
    collections::btree_map::{ArchivedBTreeMap, BTreeMapResolver},
    ser::{Allocator, Writer},
    Archive, Deserialize, Place, Serialize,
};

impl<K: Archive + Ord, V: Archive> Archive for BTreeMap<K, V>
where
    K::Archived: Ord,
{
    type Archived = ArchivedBTreeMap<K::Archived, V::Archived>;
    type Resolver = BTreeMapResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        Self::Archived::resolve_from_len(self.len(), resolver, out);
    }
}

impl<K, V, S> Serialize<S> for BTreeMap<K, V>
where
    K: Serialize<S> + Ord,
    K::Archived: Ord,
    V: Serialize<S>,
    S: Allocator + Fallible + Writer + ?Sized,
    S::Error: Source,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        Self::Archived::serialize_from_ordered_iter(self.iter(), serializer)
    }
}

impl<K, V, D> Deserialize<BTreeMap<K, V>, D>
    for ArchivedBTreeMap<K::Archived, V::Archived>
where
    K: Archive + Ord,
    K::Archived: Deserialize<K, D> + Ord,
    V: Archive,
    V::Archived: Deserialize<V, D>,
    D: Fallible + ?Sized,
{
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> Result<BTreeMap<K, V>, D::Error> {
        let mut result = BTreeMap::new();
        let r = self.visit(|ak, av| {
            let k = match ak.deserialize(deserializer) {
                Ok(k) => k,
                Err(e) => return ControlFlow::Break(e),
            };
            let v = match av.deserialize(deserializer) {
                Ok(v) => v,
                Err(e) => return ControlFlow::Break(e),
            };
            result.insert(k, v);
            ControlFlow::Continue(())
        });
        match r {
            Some(e) => Err(e),
            None => Ok(result),
        }
    }
}

impl<K, V, AK, AV> PartialEq<BTreeMap<K, V>> for ArchivedBTreeMap<AK, AV>
where
    AK: PartialEq<K>,
    AV: PartialEq<V>,
{
    fn eq(&self, other: &BTreeMap<K, V>) -> bool {
        if self.len() != other.len() {
            false
        } else {
            let mut iter = other.iter();
            self.visit(|ak, av| {
                if let Some((k, v)) = iter.next() {
                    if ak.eq(k) && av.eq(v) {
                        return ControlFlow::Continue(());
                    }
                }
                ControlFlow::Break(())
            })
            .is_none()
        }
    }
}

#[cfg(feature = "extra_impls")]
impl<K, V, AK, AV> PartialEq<ArchivedBTreeMap<AK, AV>> for BTreeMap<K, V>
where
    AK: PartialEq<K>,
    AV: PartialEq<V>,
{
    fn eq(&self, other: &ArchivedBTreeMap<AK, AV>) -> bool {
        other.eq(self)
    }
}
