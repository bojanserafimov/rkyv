use core::{
    alloc::{Layout, LayoutError},
    cell::{Cell, UnsafeCell},
    mem::ManuallyDrop,
    ptr::{self, addr_of_mut},
    str,
};

use ptr_meta::Pointee;
use rancor::Fallible;

use crate::{
    primitive::ArchivedUsize,
    ser::{Allocator, Writer, WriterExt as _},
    tuple::*,
    Archive, ArchivePointee, ArchiveUnsized, ArchivedMetadata,
    CopyOptimization, Deserialize, DeserializeUnsized, LayoutRaw, Place,
    Portable, Serialize, SerializeUnsized,
};

mod ops;
mod option;
mod primitive;
mod result;
mod time;

impl<T> LayoutRaw for T {
    fn layout_raw(
        _: <Self as Pointee>::Metadata,
    ) -> Result<Layout, LayoutError> {
        Ok(Layout::new::<T>())
    }
}

impl<T> LayoutRaw for [T] {
    fn layout_raw(
        metadata: <Self as Pointee>::Metadata,
    ) -> Result<Layout, LayoutError> {
        Layout::array::<T>(metadata)
    }
}

impl LayoutRaw for str {
    #[inline]
    fn layout_raw(
        metadata: <Self as Pointee>::Metadata,
    ) -> Result<Layout, LayoutError> {
        Layout::array::<u8>(metadata)
    }
}

impl<T> ArchivePointee for T {
    type ArchivedMetadata = ();

    fn pointer_metadata(
        _: &Self::ArchivedMetadata,
    ) -> <Self as Pointee>::Metadata {
    }
}

impl<T: Archive> ArchiveUnsized for T {
    type Archived = T::Archived;

    fn archived_metadata(&self) -> ArchivedMetadata<Self> {}
}

impl<T, S> SerializeUnsized<S> for T
where
    T: Serialize<S>,
    S: Fallible + Writer + ?Sized,
{
    fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
        self.serialize_and_resolve(serializer)
    }
}

impl<T: Archive, D: Fallible + ?Sized> DeserializeUnsized<T, D> for T::Archived
where
    T::Archived: Deserialize<T, D>,
{
    unsafe fn deserialize_unsized(
        &self,
        deserializer: &mut D,
        out: *mut T,
    ) -> Result<(), D::Error> {
        // SAFETY: The caller has guaranteed that `out` is non-null, properly
        // aligned, valid for writes, and allocated according to the layout of
        // the deserialized metadata (the unit type for sized types).
        unsafe {
            out.write(self.deserialize(deserializer)?);
        }
        Ok(())
    }

    fn deserialize_metadata(
        &self,
        _: &mut D,
    ) -> Result<<T as Pointee>::Metadata, D::Error> {
        Ok(())
    }
}

macro_rules! impl_tuple {
    ($name:ident, $($type:ident $index:tt),*) => {
        impl<$($type),*> Archive for ($($type,)*)
        where
            $($type: Archive,)*
        {
            type Archived = $name<$($type::Archived,)*>;
            type Resolver = ($($type::Resolver,)*);

            fn resolve(
                &self,
                resolver: Self::Resolver,
                out: Place<Self::Archived>,
            ) {
                // SAFETY: This pointer will only be used to manually project
                // to each of the fields to wrap them in a `Place` again.
                let out_ptr = unsafe { out.ptr() };
                $(
                    // SAFETY: `out_ptr` is guaranteed to be properly aligned
                    // and dereferenceable.
                    let ptr = unsafe { addr_of_mut!((*out_ptr).$index) };
                    // SAFETY:
                    // - `ptr` points to the `$index` field of `out`
                    // - `ptr` is properly aligned, dereferenceable, and all of
                    //   its bytes are initialized
                    let out_field = unsafe {
                        Place::from_field_unchecked(out, ptr)
                    };
                    self.$index.resolve(resolver.$index, out_field);
                )*
            }
        }

        impl<$($type,)* S> Serialize<S> for ($($type,)*)
        where
            $($type: Serialize<S>,)*
            S: Fallible + ?Sized,
        {
            fn serialize(
                &self,
                serializer: &mut S,
            ) -> Result<Self::Resolver, S::Error> {
                Ok((
                    $(self.$index.serialize(serializer)?,)*
                ))
            }
        }

        impl<$($type,)* D> Deserialize<($($type,)*), D>
            for $name<$($type::Archived,)*>
        where
            D: Fallible + ?Sized,
            $($type: Archive,)*
            $($type::Archived: Deserialize<$type, D>,)*
        {
            fn deserialize(
                &self,
                deserializer: &mut D,
            ) -> Result<($($type,)*), D::Error> {
                Ok((
                    $(self.$index.deserialize(deserializer)?,)*
                ))
            }
        }
    };
}

impl_tuple!(ArchivedTuple1, T0 0);
impl_tuple!(ArchivedTuple2, T0 0, T1 1);
impl_tuple!(ArchivedTuple3, T0 0, T1 1, T2 2);
impl_tuple!(ArchivedTuple4, T0 0, T1 1, T2 2, T3 3);
impl_tuple!(ArchivedTuple5, T0 0, T1 1, T2 2, T3 3, T4 4);
impl_tuple!(ArchivedTuple6, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5);
impl_tuple!(ArchivedTuple7, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6);
impl_tuple!(ArchivedTuple8, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7);
impl_tuple!(
    ArchivedTuple9, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8
);
impl_tuple!(
    ArchivedTuple10, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9
);
impl_tuple!(
    ArchivedTuple11, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9,
    T10 10
);
impl_tuple!(
    ArchivedTuple12, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9,
    T10 10, T11 11
);
impl_tuple!(
    ArchivedTuple13, T0 0, T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9,
    T10 10, T11 11, T12 12
);

impl<T: Archive, const N: usize> Archive for [T; N] {
    const COPY_OPTIMIZATION: CopyOptimization<Self> = unsafe {
        CopyOptimization::enable_if(T::COPY_OPTIMIZATION.is_enabled())
    };

    type Archived = [T::Archived; N];
    type Resolver = [T::Resolver; N];

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        for (i, (value, resolver)) in self.iter().zip(resolver).enumerate() {
            let out_i = unsafe { out.index(i) };
            value.resolve(resolver, out_i);
        }
    }
}

impl<T, S, const N: usize> Serialize<S> for [T; N]
where
    T: Serialize<S>,
    S: Fallible + ?Sized,
{
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        let mut result = core::mem::MaybeUninit::<Self::Resolver>::uninit();
        let result_ptr = result.as_mut_ptr().cast::<T::Resolver>();
        for (i, value) in self.iter().enumerate() {
            unsafe {
                result_ptr.add(i).write(value.serialize(serializer)?);
            }
        }
        unsafe { Ok(result.assume_init()) }
    }
}

impl<T, D, const N: usize> Deserialize<[T; N], D> for [T::Archived; N]
where
    T: Archive,
    T::Archived: Deserialize<T, D>,
    D: Fallible + ?Sized,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<[T; N], D::Error> {
        let mut result = core::mem::MaybeUninit::<[T; N]>::uninit();
        let result_ptr = result.as_mut_ptr().cast::<T>();
        for (i, value) in self.iter().enumerate() {
            unsafe {
                result_ptr.add(i).write(value.deserialize(deserializer)?);
            }
        }
        unsafe { Ok(result.assume_init()) }
    }
}

impl<T: Archive> ArchiveUnsized for [T] {
    type Archived = [T::Archived];

    fn archived_metadata(&self) -> ArchivedMetadata<Self> {
        ArchivedUsize::from_native(ptr_meta::metadata(self) as _)
    }
}

impl<T> ArchivePointee for [T] {
    type ArchivedMetadata = ArchivedUsize;

    fn pointer_metadata(
        archived: &Self::ArchivedMetadata,
    ) -> <Self as Pointee>::Metadata {
        archived.to_native() as usize
    }
}

impl<T, S> SerializeUnsized<S> for [T]
where
    T: Serialize<S>,
    S: Fallible + Allocator + Writer + ?Sized,
{
    fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
        if T::COPY_OPTIMIZATION.is_enabled() {
            let result = serializer.align_for::<T::Archived>()?;
            let as_bytes = unsafe {
                core::slice::from_raw_parts(
                    self.as_ptr().cast::<u8>(),
                    core::mem::size_of_val(self),
                )
            };
            serializer.write(as_bytes)?;

            Ok(result)
        } else {
            use crate::util::SerVec;

            SerVec::with_capacity(
                serializer,
                self.len(),
                |resolvers, serializer| {
                    for value in self.iter() {
                        unsafe {
                            resolvers
                                .push_unchecked(value.serialize(serializer)?);
                        }
                    }

                    let result = serializer.align_for::<T::Archived>()?;

                    for (value, resolver) in self.iter().zip(resolvers.drain())
                    {
                        unsafe {
                            serializer.resolve_aligned(value, resolver)?;
                        }
                    }

                    Ok(result)
                },
            )?
        }
    }
}

impl<T, U, D> DeserializeUnsized<[U], D> for [T]
where
    T: Deserialize<U, D>,
    D: Fallible + ?Sized,
{
    unsafe fn deserialize_unsized(
        &self,
        deserializer: &mut D,
        out: *mut [U],
    ) -> Result<(), D::Error> {
        for (i, item) in self.iter().enumerate() {
            // SAFETY: The caller has guaranteed that `out` points to a slice
            // with a length guaranteed to match the length of `self`. Since `i`
            // is less than the length of the slice, the result of the pointer
            // add is always in-bounds.
            let out_ptr = unsafe { out.cast::<U>().add(i) };
            // SAFETY: `out_ptr` points to an element of `out` and so is
            // guaranteed to be non-null, properly aligned, and valid for
            // writes.
            unsafe {
                out_ptr.write(item.deserialize(deserializer)?);
            }
        }
        Ok(())
    }

    fn deserialize_metadata(
        &self,
        _: &mut D,
    ) -> Result<<[U] as Pointee>::Metadata, D::Error> {
        Ok(ptr_meta::metadata(self))
    }
}

/// `str`

unsafe impl Portable for str {}

impl ArchiveUnsized for str {
    type Archived = str;

    #[inline]
    fn archived_metadata(&self) -> ArchivedMetadata<Self> {
        ArchivedUsize::from_native(ptr_meta::metadata(self) as _)
    }
}

impl ArchivePointee for str {
    type ArchivedMetadata = ArchivedUsize;

    #[inline]
    fn pointer_metadata(
        archived: &Self::ArchivedMetadata,
    ) -> <Self as Pointee>::Metadata {
        <[u8]>::pointer_metadata(archived)
    }
}

impl<S: Fallible + Writer + ?Sized> SerializeUnsized<S> for str {
    fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
        let result = serializer.pos();
        serializer.write(self.as_bytes())?;
        Ok(result)
    }
}

impl<D: Fallible + ?Sized> DeserializeUnsized<str, D> for str {
    unsafe fn deserialize_unsized(
        &self,
        _: &mut D,
        out: *mut str,
    ) -> Result<(), D::Error> {
        // SAFETY: The caller has guaranteed that `out` is non-null, properly
        // aligned, valid for writes, and points to memory allocated according
        // to the layout for the metadata returned from `deserialize_metadata`.
        // Therefore, `out` points to at least `self.len()` bytes.
        // `self.as_ptr()` is valid for reads and points to the bytes of `self`
        // which are also at least `self.len()` bytes.
        unsafe {
            ptr::copy_nonoverlapping(
                self.as_ptr(),
                out.cast::<u8>(),
                self.len(),
            );
        }
        Ok(())
    }

    fn deserialize_metadata(
        &self,
        _: &mut D,
    ) -> Result<<str as Pointee>::Metadata, D::Error> {
        Ok(ptr_meta::metadata(self))
    }
}

// `ManuallyDrop`

unsafe impl<T: Portable> Portable for ManuallyDrop<T> {}

impl<T: Archive> Archive for ManuallyDrop<T> {
    const COPY_OPTIMIZATION: CopyOptimization<Self> = unsafe {
        CopyOptimization::enable_if(T::COPY_OPTIMIZATION.is_enabled())
    };

    type Archived = ManuallyDrop<T::Archived>;
    type Resolver = T::Resolver;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        let out_inner = unsafe { out.cast_unchecked::<T::Archived>() };
        T::resolve(self, resolver, out_inner)
    }
}

impl<T: Serialize<S>, S: Fallible + ?Sized> Serialize<S> for ManuallyDrop<T> {
    fn serialize(
        &self,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        T::serialize(self, serializer)
    }
}

impl<T, D> Deserialize<ManuallyDrop<T>, D> for ManuallyDrop<T::Archived>
where
    T: Archive,
    T::Archived: Deserialize<T, D>,
    D: Fallible + ?Sized,
{
    fn deserialize(
        &self,
        deserializer: &mut D,
    ) -> Result<ManuallyDrop<T>, D::Error> {
        T::Archived::deserialize(self, deserializer).map(ManuallyDrop::new)
    }
}

// `Cell`

unsafe impl<T: Portable + ?Sized> Portable for Cell<T> {}

// `UnsafeCell`

unsafe impl<T: Portable + ?Sized> Portable for UnsafeCell<T> {}
