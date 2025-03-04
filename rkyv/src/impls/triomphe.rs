use core::{
    alloc::LayoutError,
    mem::{forget, MaybeUninit},
};

use ptr_meta::Pointee;
use rancor::{Fallible, Source};
use triomphe::Arc;

use crate::{
    de::{Metadata, Pooling, PoolingExt, SharedPointer},
    rc::{ArchivedRc, RcResolver},
    ser::{Sharing, Writer},
    Archive, ArchiveUnsized, Deserialize, DeserializeUnsized, Place, Serialize,
    SerializeUnsized,
};

pub struct TriompheArcFlavor;

unsafe impl<T> SharedPointer<T> for Arc<T> {
    fn alloc(_: <T as Pointee>::Metadata) -> Result<*mut T, LayoutError> {
        Ok(Arc::into_raw(Arc::<MaybeUninit<T>>::new_uninit())
            .cast::<T>()
            .cast_mut())
    }

    unsafe fn from_value(ptr: *mut T) -> *mut T {
        ptr
    }

    unsafe fn drop(ptr: *mut T) {
        drop(unsafe { Arc::from_raw(ptr) })
    }
}

impl<T: ArchiveUnsized + ?Sized> Archive for Arc<T> {
    type Archived = ArchivedRc<T::Archived, TriompheArcFlavor>;
    type Resolver = RcResolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedRc::resolve_from_ref(self.as_ref(), resolver, out);
    }
}

impl<T, S> Serialize<S> for Arc<T>
where
    T: SerializeUnsized<S> + ?Sized + 'static,
    S: Writer + Sharing + Fallible + ?Sized,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedRc::<T::Archived, TriompheArcFlavor>::serialize_from_ref(
            self.as_ref(),
            serializer,
        )
    }
}

impl<T, D> Deserialize<Arc<T>, D> for ArchivedRc<T::Archived, TriompheArcFlavor>
where
    T: ArchiveUnsized + 'static,
    T::Metadata: Into<Metadata>,
    Metadata: Into<T::Metadata>,
    T::Archived: DeserializeUnsized<T, D>,
    D: Pooling + Fallible + ?Sized,
    D::Error: Source,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<Arc<T>, D::Error> {
        let raw_shared_ptr =
            deserializer.deserialize_shared::<_, Arc<T>>(self.get())?;
        let shared_ptr = unsafe { Arc::<T>::from_raw(raw_shared_ptr) };
        forget(shared_ptr.clone());
        Ok(shared_ptr)
    }
}
