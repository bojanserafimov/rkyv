#[cfg(target_has_atomic = "16")]
use {
    crate::primitive::{
        ArchivedAtomicI16, ArchivedAtomicU16, ArchivedI16, ArchivedU16,
    },
    core::sync::atomic::{AtomicI16, AtomicU16},
};
#[cfg(target_has_atomic = "32")]
use {
    crate::primitive::{
        ArchivedAtomicI32, ArchivedAtomicU32, ArchivedI32, ArchivedU32,
    },
    core::sync::atomic::{AtomicI32, AtomicU32},
};
#[cfg(target_has_atomic = "64")]
use {
    crate::primitive::{
        ArchivedAtomicI64, ArchivedAtomicU64, ArchivedI64, ArchivedU64,
    },
    core::sync::atomic::{AtomicI64, AtomicU64},
};
#[cfg(any(
    all(target_has_atomic = "16", feature = "pointer_width_16"),
    all(
        target_has_atomic = "32",
        not(any(feature = "pointer_width_16", feature = "pointer_width_64")),
    ),
    all(target_has_atomic = "64", feature = "pointer_width_64"),
))]
use {
    crate::primitive::{
        ArchivedAtomicIsize, ArchivedAtomicUsize, ArchivedIsize, ArchivedUsize,
    },
    core::sync::atomic::{AtomicIsize, AtomicUsize},
};

use crate::{
    rancor::Fallible,
    with::{
        impls::atomic::LoadOrdering, ArchiveWith, AsAtomic, AtomicLoad,
        DeserializeWith,
    },
    Place,
};

macro_rules! impl_multi_byte_atomic {
    ($atomic:ty, $archived:ty, $archived_non_atomic:ty) => {
        impl<SO: LoadOrdering> ArchiveWith<$atomic> for AtomicLoad<SO> {
            type Archived = $archived_non_atomic;
            type Resolver = ();

            fn resolve_with(
                field: &$atomic,
                _: Self::Resolver,
                out: Place<Self::Archived>,
            ) {
                out.write(<$archived_non_atomic>::from_native(
                    field.load(SO::ORDERING),
                ));
            }
        }

        impl<SO, DO> ArchiveWith<$atomic> for AsAtomic<SO, DO>
        where
            SO: LoadOrdering,
        {
            type Archived = $archived;
            type Resolver = ();

            fn resolve_with(
                field: &$atomic,
                _: Self::Resolver,
                out: Place<Self::Archived>,
            ) {
                out.write(<$archived>::new(field.load(SO::ORDERING)))
            }
        }

        impl_serialize_with_atomic!($atomic);

        impl<D, SO> DeserializeWith<$archived_non_atomic, $atomic, D>
            for AtomicLoad<SO>
        where
            D: Fallible + ?Sized,
        {
            fn deserialize_with(
                field: &$archived_non_atomic,
                _: &mut D,
            ) -> Result<$atomic, D::Error> {
                Ok(<$atomic>::new(field.to_native()))
            }
        }

        impl<D, SO, DO> DeserializeWith<$archived, $atomic, D>
            for AsAtomic<SO, DO>
        where
            D: Fallible + ?Sized,
            DO: LoadOrdering,
        {
            fn deserialize_with(
                field: &$archived,
                _: &mut D,
            ) -> Result<$atomic, D::Error> {
                Ok(<$atomic>::new(field.load(DO::ORDERING)))
            }
        }
    };
}

macro_rules! impl_multi_byte_atomics {
    ($($atomic:ty, $archived:ty, $archived_non_atomic:ty);* $(;)?) => {
        $(
            impl_multi_byte_atomic!($atomic, $archived, $archived_non_atomic);
        )*
    }
}

#[cfg(target_has_atomic = "16")]
impl_multi_byte_atomics! {
    AtomicI16, ArchivedAtomicI16, ArchivedI16;
    AtomicU16, ArchivedAtomicU16, ArchivedU16;
}
#[cfg(target_has_atomic = "32")]
impl_multi_byte_atomics! {
    AtomicI32, ArchivedAtomicI32, ArchivedI32;
    AtomicU32, ArchivedAtomicU32, ArchivedU32;
}
#[cfg(target_has_atomic = "64")]
impl_multi_byte_atomics! {
    AtomicI64, ArchivedAtomicI64, ArchivedI64;
    AtomicU64, ArchivedAtomicU64, ArchivedU64;
}

// AtomicUsize

macro_rules! impl_atomic_size_type {
    ($atomic:ty, $archived:ty, $archived_non_atomic:ty) => {
        impl<SO: LoadOrdering> ArchiveWith<$atomic> for AtomicLoad<SO> {
            type Archived = $archived_non_atomic;
            type Resolver = ();

            fn resolve_with(
                field: &$atomic,
                _: Self::Resolver,
                out: Place<Self::Archived>,
            ) {
                out.write(<$archived_non_atomic>::from_native(
                    field.load(SO::ORDERING) as _,
                ));
            }
        }

        impl<SO, DO> ArchiveWith<$atomic> for AsAtomic<SO, DO>
        where
            SO: LoadOrdering,
        {
            type Archived = $archived;
            type Resolver = ();

            fn resolve_with(
                field: &$atomic,
                _: Self::Resolver,
                out: Place<Self::Archived>,
            ) {
                out.write(<$archived>::new(field.load(SO::ORDERING) as _));
            }
        }

        impl_serialize_with_atomic!($atomic);

        impl<D, SO> DeserializeWith<$archived_non_atomic, $atomic, D>
            for AtomicLoad<SO>
        where
            D: Fallible + ?Sized,
        {
            fn deserialize_with(
                field: &$archived_non_atomic,
                _: &mut D,
            ) -> Result<$atomic, D::Error> {
                Ok(<$atomic>::new(field.to_native() as _))
            }
        }

        impl<D, SO, DO> DeserializeWith<$archived, $atomic, D>
            for AsAtomic<SO, DO>
        where
            D: Fallible + ?Sized,
            DO: LoadOrdering,
        {
            fn deserialize_with(
                field: &$archived,
                _: &mut D,
            ) -> Result<$atomic, D::Error> {
                Ok(<$atomic>::new(field.load(DO::ORDERING) as _))
            }
        }
    };
}

macro_rules! impl_atomic_size_types {
    ($($atomic:ty, $archived:ty, $archived_non_atomic:ty;)*) => {
        $(
            impl_atomic_size_type!($atomic, $archived, $archived_non_atomic);
        )*
    }
}

#[cfg(any(
    all(target_has_atomic = "16", feature = "pointer_width_16"),
    all(
        target_has_atomic = "32",
        not(any(feature = "pointer_width_16", feature = "pointer_width_64")),
    ),
    all(target_has_atomic = "64", feature = "pointer_width_64"),
))]
impl_atomic_size_types! {
    AtomicIsize, ArchivedAtomicIsize, ArchivedIsize;
    AtomicUsize, ArchivedAtomicUsize, ArchivedUsize;
}

// TODO: provide impls for rend atomics
