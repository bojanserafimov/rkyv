use bytes::{Bytes, BytesMut};
use rancor::Fallible;

use crate::{
    ser::{Allocator, Writer},
    vec::{ArchivedVec, VecResolver},
    Archive, Archived, Deserialize, Place, Serialize,
};

impl Archive for Bytes {
    type Archived = ArchivedVec<u8>;
    type Resolver = VecResolver;

    #[inline]
    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedVec::resolve_from_slice(self, resolver, out);
    }
}

impl<S: Fallible + Allocator + Writer + ?Sized> Serialize<S> for Bytes {
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedVec::serialize_from_slice(self, serializer)
    }
}

impl<D: Fallible + ?Sized> Deserialize<Bytes, D> for ArchivedVec<Archived<u8>> {
    fn deserialize(&self, _deserializer: &mut D) -> Result<Bytes, D::Error> {
        let mut result = BytesMut::new();
        result.extend_from_slice(self.as_slice());
        Ok(result.freeze())
    }
}

impl<T: Archive> PartialEq<Bytes> for ArchivedVec<T>
where
    bytes::Bytes: PartialEq<[T]>,
{
    fn eq(&self, other: &Bytes) -> bool {
        other == self.as_slice()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::vec;

    use bytes::Bytes;
    use rancor::{Error, Infallible};

    use crate::{access_unchecked, deserialize, to_bytes, vec::ArchivedVec};

    #[test]
    fn bytes() {
        let value = Bytes::from(vec![10, 20, 40, 80]);

        let bytes = to_bytes::<Error>(&value).unwrap();
        let archived = unsafe { access_unchecked::<ArchivedVec<u8>>(&bytes) };
        assert_eq!(archived, &value);

        let deserialized =
            deserialize::<Bytes, _, Infallible>(archived, &mut ()).unwrap();
        assert_eq!(value, deserialized);
    }
}
