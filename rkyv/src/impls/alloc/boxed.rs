use core::cmp;

#[cfg(not(feature = "std"))]
use ::alloc::{alloc, boxed::Box};
#[cfg(feature = "std")]
use ::std::alloc;
use rancor::{Fallible, ResultExt as _, Source};

use crate::{
    boxed::{ArchivedBox, BoxResolver},
    Archive, ArchivePointee, ArchiveUnsized, Deserialize, DeserializeUnsized,
    LayoutRaw, Place, Serialize, SerializeUnsized,
};

impl<T: ArchiveUnsized + ?Sized> Archive for Box<T> {
    type Archived = ArchivedBox<T::Archived>;
    type Resolver = BoxResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedBox::resolve_from_ref(self.as_ref(), resolver, out);
    }
}

impl<T: SerializeUnsized<S> + ?Sized, S: Fallible + ?Sized> Serialize<S>
    for Box<T>
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedBox::serialize_from_ref(self.as_ref(), serializer)
    }
}

impl<T, D> Deserialize<Box<T>, D> for ArchivedBox<T::Archived>
where
    T: ArchiveUnsized + LayoutRaw + ?Sized,
    T::Archived: DeserializeUnsized<T, D>,
    D: Fallible + ?Sized,
    D::Error: Source,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<Box<T>, D::Error> {
        let metadata = self.get().deserialize_metadata(deserializer)?;
        let layout = T::layout_raw(metadata).into_error()?;
        let data_address = if layout.size() > 0 {
            unsafe { alloc::alloc(layout) }
        } else {
            crate::polyfill::dangling(&layout).as_ptr()
        };

        let out = ptr_meta::from_raw_parts_mut(data_address.cast(), metadata);

        unsafe {
            self.get().deserialize_unsized(deserializer, out)?;
        }
        unsafe { Ok(Box::from_raw(out)) }
    }
}

impl<T: ArchivePointee + PartialEq<U> + ?Sized, U: ?Sized> PartialEq<Box<U>>
    for ArchivedBox<T>
{
    fn eq(&self, other: &Box<U>) -> bool {
        self.get().eq(other.as_ref())
    }
}

impl<T: ArchivePointee + PartialOrd<U> + ?Sized, U: ?Sized> PartialOrd<Box<U>>
    for ArchivedBox<T>
{
    fn partial_cmp(&self, other: &Box<U>) -> Option<cmp::Ordering> {
        self.get().partial_cmp(other.as_ref())
    }
}
