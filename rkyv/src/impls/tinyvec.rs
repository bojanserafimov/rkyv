use rancor::Fallible;
#[cfg(all(feature = "tinyvec", feature = "alloc"))]
use tinyvec::TinyVec;
use tinyvec::{Array, ArrayVec, SliceVec};

use crate::{
    ser::{Allocator, Writer},
    vec::{ArchivedVec, VecResolver},
    Archive, Archived, Deserialize, Place, Serialize,
};

// ArrayVec

impl<A: Array> Archive for ArrayVec<A>
where
    A::Item: Archive,
{
    type Archived = ArchivedVec<Archived<A::Item>>;
    type Resolver = VecResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedVec::resolve_from_slice(self.as_slice(), resolver, out);
    }
}

impl<A, S> Serialize<S> for ArrayVec<A>
where
    A: Array,
    A::Item: Serialize<S>,
    S: Fallible + Allocator + Writer + ?Sized,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedVec::serialize_from_slice(self.as_slice(), serializer)
    }
}

impl<A, D> Deserialize<ArrayVec<A>, D> for ArchivedVec<Archived<A::Item>>
where
    A: Array,
    A::Item: Archive,
    Archived<A::Item>: Deserialize<A::Item, D>,
    D: Fallible + ?Sized,
{
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> Result<ArrayVec<A>, D::Error> {
        let mut result = ArrayVec::new();
        for item in self.as_slice() {
            result.push(item.deserialize(deserializer)?);
        }
        Ok(result)
    }
}

// SliceVec

impl<'s, T: Archive> Archive for SliceVec<'s, T> {
    type Archived = ArchivedVec<T::Archived>;
    type Resolver = VecResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedVec::resolve_from_slice(self.as_slice(), resolver, out);
    }
}

impl<'s, T, S> Serialize<S> for SliceVec<'s, T>
where
    T: Serialize<S>,
    S: Fallible + Allocator + Writer + ?Sized,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedVec::serialize_from_slice(self.as_slice(), serializer)
    }
}

// SliceVec cannot be deserialized because it borrows backing memory

// TinyVec

#[cfg(all(feature = "tinyvec", feature = "alloc"))]
impl<A: Array> Archive for TinyVec<A>
where
    A::Item: Archive,
{
    type Archived = ArchivedVec<Archived<A::Item>>;
    type Resolver = VecResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedVec::resolve_from_slice(self.as_slice(), resolver, out);
    }
}

#[cfg(all(feature = "tinyvec", feature = "alloc"))]
impl<A, S> Serialize<S> for TinyVec<A>
where
    A: Array,
    A::Item: Serialize<S>,
    S: Fallible + Allocator + Writer + ?Sized,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedVec::serialize_from_slice(self.as_slice(), serializer)
    }
}

#[cfg(all(feature = "tinyvec", feature = "alloc"))]
impl<A, D> Deserialize<TinyVec<A>, D> for ArchivedVec<Archived<A::Item>>
where
    A: Array,
    A::Item: Archive,
    Archived<A::Item>: Deserialize<A::Item, D>,
    D: Fallible + ?Sized,
{
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> Result<TinyVec<A>, D::Error> {
        let mut result = TinyVec::new();
        for item in self.as_slice() {
            result.push(item.deserialize(deserializer)?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use rancor::{Error, Infallible};
    use tinyvec::{array_vec, Array, ArrayVec, SliceVec};

    use crate::{
        access_unchecked, deserialize, to_bytes, vec::ArchivedVec, Archived,
    };

    #[test]
    fn array_vec() {
        let value = array_vec!([i32; 10] => 10, 20, 40, 80);

        let bytes = to_bytes::<Error>(&value).unwrap();
        let archived =
            unsafe { access_unchecked::<ArchivedVec<Archived<i32>>>(&bytes) };
        assert_eq!(archived.as_slice(), &[10, 20, 40, 80]);

        let deserialized = deserialize::<ArrayVec<[i32; 10]>, _, Infallible>(
            archived,
            &mut (),
        )
        .unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn slice_vec() {
        let mut backing = [0i32; 10];
        let mut value = SliceVec::from_slice_len(backing.as_slice_mut(), 0);
        value.push(10);
        value.push(20);
        value.push(40);
        value.push(80);

        let bytes = to_bytes::<Error>(&value).unwrap();
        let archived =
            unsafe { access_unchecked::<ArchivedVec<Archived<i32>>>(&bytes) };
        assert_eq!(archived.as_slice(), &[10, 20, 40, 80]);
    }

    #[cfg(all(feature = "tinyvec", feature = "alloc"))]
    #[test]
    fn tiny_vec() {
        #[cfg(not(feature = "std"))]
        use alloc::vec;

        use tinyvec::{tiny_vec, TinyVec};

        let value = tiny_vec!([i32; 10] => 10, 20, 40, 80);

        let bytes = to_bytes::<Error>(&value).unwrap();
        let archived =
            unsafe { access_unchecked::<ArchivedVec<Archived<i32>>>(&bytes) };
        assert_eq!(archived.as_slice(), &[10, 20, 40, 80]);

        let deserialized: TinyVec<[i32; 10]> =
            deserialize::<_, _, Error>(archived, &mut ()).unwrap();
        assert_eq!(value, deserialized);
    }
}
