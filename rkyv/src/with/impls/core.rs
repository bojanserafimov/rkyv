use core::{
    cell::{Cell, UnsafeCell},
    hint::unreachable_unchecked,
    num::{NonZeroIsize, NonZeroUsize},
};

use munge::munge;
use rancor::Fallible;

use crate::{
    boxed::{ArchivedBox, BoxResolver},
    niche::option_nonzero::{
        ArchivedOptionNonZeroIsize, ArchivedOptionNonZeroUsize,
    },
    option::ArchivedOption,
    place::Initialized,
    primitive::{FixedNonZeroIsize, FixedNonZeroUsize},
    with::{
        ArchiveWith, Boxed, BoxedInline, DeserializeWith, Inline, Map, Niche,
        SerializeWith, Skip, Unsafe,
    },
    Archive, ArchiveUnsized, Deserialize, Place, Serialize, SerializeUnsized,
};

// Map for Options

// Copy-paste from Option's impls for the most part
impl<A, O> ArchiveWith<Option<O>> for Map<A>
where
    A: ArchiveWith<O>,
{
    type Archived = ArchivedOption<<A as ArchiveWith<O>>::Archived>;
    type Resolver = Option<<A as ArchiveWith<O>>::Resolver>;

    fn resolve_with(
        field: &Option<O>,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        match resolver {
            None => {
                let out = unsafe {
                    out.cast_unchecked::<ArchivedOptionVariantNone>()
                };
                munge!(let ArchivedOptionVariantNone(tag) = out);
                tag.write(ArchivedOptionTag::None);
            }
            Some(resolver) => {
                let out = unsafe {
                    out.cast_unchecked::<ArchivedOptionVariantSome<
                        <A as ArchiveWith<O>>::Archived,
                    >>()
                };
                munge!(let ArchivedOptionVariantSome(tag, out_value) = out);
                tag.write(ArchivedOptionTag::Some);

                let value = if let Some(value) = field.as_ref() {
                    value
                } else {
                    unsafe {
                        unreachable_unchecked();
                    }
                };

                A::resolve_with(value, resolver, out_value);
            }
        }
    }
}

impl<A, O, S> SerializeWith<Option<O>, S> for Map<A>
where
    S: Fallible + ?Sized,
    A: ArchiveWith<O> + SerializeWith<O, S>,
{
    fn serialize_with(
        field: &Option<O>,
        s: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        field
            .as_ref()
            .map(|value| A::serialize_with(value, s))
            .transpose()
    }
}

impl<A, O, D>
    DeserializeWith<
        ArchivedOption<<A as ArchiveWith<O>>::Archived>,
        Option<O>,
        D,
    > for Map<A>
where
    D: Fallible + ?Sized,
    A: ArchiveWith<O> + DeserializeWith<<A as ArchiveWith<O>>::Archived, O, D>,
{
    fn deserialize_with(
        field: &ArchivedOption<<A as ArchiveWith<O>>::Archived>,
        d: &mut D,
    ) -> Result<Option<O>, D::Error> {
        match field {
            ArchivedOption::Some(value) => {
                Ok(Some(A::deserialize_with(value, d)?))
            }
            ArchivedOption::None => Ok(None),
        }
    }
}

#[repr(u8)]
enum ArchivedOptionTag {
    None,
    Some,
}

// SAFETY: `ArchivedOptionTag` is `repr(u8)` and so is always initialized.
unsafe impl Initialized for ArchivedOptionTag {}

#[repr(C)]
struct ArchivedOptionVariantNone(ArchivedOptionTag);

#[repr(C)]
struct ArchivedOptionVariantSome<T>(ArchivedOptionTag, T);

// Inline

impl<F: Archive> ArchiveWith<&F> for Inline {
    type Archived = F::Archived;
    type Resolver = F::Resolver;

    fn resolve_with(
        field: &&F,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        field.resolve(resolver, out);
    }
}

impl<F: Serialize<S>, S: Fallible + ?Sized> SerializeWith<&F, S> for Inline {
    fn serialize_with(
        field: &&F,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        field.serialize(serializer)
    }
}

// BoxedInline

impl<F: ArchiveUnsized + ?Sized> ArchiveWith<&F> for BoxedInline {
    type Archived = ArchivedBox<F::Archived>;
    type Resolver = BoxResolver;

    fn resolve_with(
        field: &&F,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        ArchivedBox::resolve_from_ref(*field, resolver, out);
    }
}

impl<F: SerializeUnsized<S> + ?Sized, S: Fallible + ?Sized> SerializeWith<&F, S>
    for BoxedInline
{
    fn serialize_with(
        field: &&F,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedBox::serialize_from_ref(*field, serializer)
    }
}

// Boxed

impl<F: ArchiveUnsized + ?Sized> ArchiveWith<F> for Boxed {
    type Archived = ArchivedBox<F::Archived>;
    type Resolver = BoxResolver;

    fn resolve_with(
        field: &F,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        ArchivedBox::resolve_from_ref(field, resolver, out);
    }
}

impl<F: SerializeUnsized<S> + ?Sized, S: Fallible + ?Sized> SerializeWith<F, S>
    for Boxed
{
    fn serialize_with(
        field: &F,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedBox::serialize_from_ref(field, serializer)
    }
}

impl<F: Archive, D: Fallible + ?Sized>
    DeserializeWith<ArchivedBox<F::Archived>, F, D> for Boxed
where
    F::Archived: Deserialize<F, D>,
{
    fn deserialize_with(
        field: &ArchivedBox<F::Archived>,
        deserializer: &mut D,
    ) -> Result<F, D::Error> {
        field.get().deserialize(deserializer)
    }
}

// Niche

impl ArchiveWith<Option<NonZeroIsize>> for Niche {
    type Archived = ArchivedOptionNonZeroIsize;
    type Resolver = ();

    #[inline]
    fn resolve_with(
        field: &Option<NonZeroIsize>,
        _: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        let f = field.as_ref().map(|&x| x.try_into().unwrap());
        ArchivedOptionNonZeroIsize::resolve_from_option(f, out);
    }
}

impl<S: Fallible + ?Sized> SerializeWith<Option<NonZeroIsize>, S> for Niche {
    fn serialize_with(
        _: &Option<NonZeroIsize>,
        _: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible + ?Sized>
    DeserializeWith<ArchivedOptionNonZeroIsize, Option<NonZeroIsize>, D>
    for Niche
{
    fn deserialize_with(
        field: &ArchivedOptionNonZeroIsize,
        _: &mut D,
    ) -> Result<Option<NonZeroIsize>, D::Error> {
        // This conversion is necessary with archive_be and archive_le
        #[allow(clippy::useless_conversion)]
        Ok(field
            .as_ref()
            .map(|x| FixedNonZeroIsize::from(*x).try_into().unwrap()))
    }
}

impl ArchiveWith<Option<NonZeroUsize>> for Niche {
    type Archived = ArchivedOptionNonZeroUsize;
    type Resolver = ();

    #[inline]
    fn resolve_with(
        field: &Option<NonZeroUsize>,
        _: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        let f = field.as_ref().map(|&x| x.try_into().unwrap());
        ArchivedOptionNonZeroUsize::resolve_from_option(f, out);
    }
}

impl<S: Fallible + ?Sized> SerializeWith<Option<NonZeroUsize>, S> for Niche {
    fn serialize_with(
        _: &Option<NonZeroUsize>,
        _: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible + ?Sized>
    DeserializeWith<ArchivedOptionNonZeroUsize, Option<NonZeroUsize>, D>
    for Niche
{
    fn deserialize_with(
        field: &ArchivedOptionNonZeroUsize,
        _: &mut D,
    ) -> Result<Option<NonZeroUsize>, D::Error> {
        // This conversion is necessary with archive_be and archive_le
        #[allow(clippy::useless_conversion)]
        Ok(field
            .as_ref()
            .map(|x| FixedNonZeroUsize::from(*x).try_into().unwrap()))
    }
}

// Unsafe

impl<F: Archive> ArchiveWith<UnsafeCell<F>> for Unsafe {
    type Archived = UnsafeCell<F::Archived>;
    type Resolver = F::Resolver;

    fn resolve_with(
        field: &UnsafeCell<F>,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        let value = unsafe { &*field.get() };
        let out = unsafe { out.cast_unchecked() };
        F::resolve(value, resolver, out);
    }
}

impl<F: Serialize<S>, S: Fallible + ?Sized> SerializeWith<UnsafeCell<F>, S>
    for Unsafe
{
    fn serialize_with(
        field: &UnsafeCell<F>,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        unsafe { (*field.get()).serialize(serializer) }
    }
}

impl<F: Archive, D: Fallible + ?Sized>
    DeserializeWith<UnsafeCell<F::Archived>, UnsafeCell<F>, D> for Unsafe
where
    F::Archived: Deserialize<F, D>,
{
    fn deserialize_with(
        field: &UnsafeCell<F::Archived>,
        deserializer: &mut D,
    ) -> Result<UnsafeCell<F>, D::Error> {
        unsafe {
            (*field.get())
                .deserialize(deserializer)
                .map(|x| UnsafeCell::new(x))
        }
    }
}

impl<F: Archive> ArchiveWith<Cell<F>> for Unsafe {
    type Archived = Cell<F::Archived>;
    type Resolver = F::Resolver;

    fn resolve_with(
        field: &Cell<F>,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        let value = unsafe { &*field.as_ptr() };
        let out = unsafe { out.cast_unchecked() };
        F::resolve(value, resolver, out);
    }
}

impl<F: Serialize<S>, S: Fallible + ?Sized> SerializeWith<Cell<F>, S>
    for Unsafe
{
    fn serialize_with(
        field: &Cell<F>,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        unsafe { (*field.as_ptr()).serialize(serializer) }
    }
}

impl<F: Archive, D: Fallible + ?Sized>
    DeserializeWith<Cell<F::Archived>, Cell<F>, D> for Unsafe
where
    F::Archived: Deserialize<F, D>,
{
    fn deserialize_with(
        field: &Cell<F::Archived>,
        deserializer: &mut D,
    ) -> Result<Cell<F>, D::Error> {
        unsafe {
            (*field.as_ptr())
                .deserialize(deserializer)
                .map(|x| Cell::new(x))
        }
    }
}

// Skip

impl<F> ArchiveWith<F> for Skip {
    type Archived = ();
    type Resolver = ();

    fn resolve_with(_: &F, _: Self::Resolver, _: Place<Self::Archived>) {}
}

impl<F, S: Fallible + ?Sized> SerializeWith<F, S> for Skip {
    fn serialize_with(_: &F, _: &mut S) -> Result<(), S::Error> {
        Ok(())
    }
}

impl<F: Default, D: Fallible + ?Sized> DeserializeWith<(), F, D> for Skip {
    fn deserialize_with(_: &(), _: &mut D) -> Result<F, D::Error> {
        Ok(Default::default())
    }
}
