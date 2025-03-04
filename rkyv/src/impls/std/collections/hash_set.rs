use core::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
};
use std::collections::HashSet;

use rancor::{Fallible, Source};

use crate::{
    collections::swiss_table::set::{ArchivedHashSet, HashSetResolver},
    ser::{Allocator, Writer},
    Archive, Deserialize, Place, Serialize,
};

impl<K: Archive + Hash + Eq, S> Archive for HashSet<K, S>
where
    K::Archived: Hash + Eq,
{
    type Archived = ArchivedHashSet<K::Archived>;
    type Resolver = HashSetResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedHashSet::<K::Archived>::resolve_from_len(
            self.len(),
            (7, 8),
            resolver,
            out,
        );
    }
}

impl<K, S, RS> Serialize<S> for HashSet<K, RS>
where
    K::Archived: Hash + Eq,
    K: Serialize<S> + Hash + Eq,
    S: Fallible + Allocator + Writer + ?Sized,
    S::Error: Source,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedHashSet::<K::Archived>::serialize_from_iter(
            self.iter(),
            (7, 8),
            serializer,
        )
    }
}

impl<K, D, S> Deserialize<HashSet<K, S>, D> for ArchivedHashSet<K::Archived>
where
    K: Archive + Hash + Eq,
    K::Archived: Deserialize<K, D> + Hash + Eq,
    D: Fallible + ?Sized,
    S: Default + BuildHasher,
{
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> Result<HashSet<K, S>, D::Error> {
        let mut result = HashSet::with_hasher(S::default());
        for k in self.iter() {
            result.insert(k.deserialize(deserializer)?);
        }
        Ok(result)
    }
}

impl<K: Hash + Eq + Borrow<AK>, AK: Hash + Eq, S: BuildHasher>
    PartialEq<HashSet<K, S>> for ArchivedHashSet<AK>
{
    fn eq(&self, other: &HashSet<K, S>) -> bool {
        if self.len() != other.len() {
            false
        } else {
            self.iter().all(|key| other.get(key).is_some())
        }
    }
}

impl<K: Hash + Eq + Borrow<AK>, AK: Hash + Eq, S: BuildHasher>
    PartialEq<ArchivedHashSet<AK>> for HashSet<K, S>
{
    fn eq(&self, other: &ArchivedHashSet<AK>) -> bool {
        other.eq(self)
    }
}
