//! Wrapper type support and commonly used wrappers.
//!
//! Wrappers can be applied with the `#[with(...)]` attribute in the
//! [`Archive`](macro@crate::Archive) macro.

mod impls;

use core::{fmt, marker::PhantomData, ops::Deref};

use rancor::Fallible;

use crate::{Place, Portable};

// TODO: Gate unsafe wrappers behind Unsafe.

/// A variant of [`Archive`](crate::Archive) that works with wrappers.
///
/// Creating a wrapper allows users to customize how fields are archived easily
/// without changing the unarchived type.
///
/// This trait allows wrapper types to transparently change the archive
/// behaviors for struct and enum fields. When a field is serialized, it may use
/// the implementations for the wrapper type and the given field instead of the
/// implementation for the type itself.
///
/// # Example
///
/// ```
/// use rkyv::{
///     access_unchecked, deserialize,
///     rancor::{Error, Fallible, Infallible, ResultExt as _},
///     to_bytes,
///     with::{ArchiveWith, DeserializeWith, SerializeWith},
///     Archive, Archived, Deserialize, Place, Resolver, Serialize,
/// };
///
/// struct Incremented;
///
/// impl ArchiveWith<i32> for Incremented {
///     type Archived = Archived<i32>;
///     type Resolver = Resolver<i32>;
///
///     fn resolve_with(field: &i32, _: (), out: Place<Self::Archived>) {
///         let incremented = field + 1;
///         incremented.resolve((), out);
///     }
/// }
///
/// impl<S> SerializeWith<i32, S> for Incremented
/// where
///     S: Fallible + ?Sized,
///     i32: Serialize<S>,
/// {
///     fn serialize_with(
///         field: &i32,
///         serializer: &mut S,
///     ) -> Result<Self::Resolver, S::Error> {
///         let incremented = field + 1;
///         incremented.serialize(serializer)
///     }
/// }
///
/// impl<D> DeserializeWith<Archived<i32>, i32, D> for Incremented
/// where
///     D: Fallible + ?Sized,
///     Archived<i32>: Deserialize<i32, D>,
/// {
///     fn deserialize_with(
///         field: &Archived<i32>,
///         deserializer: &mut D,
///     ) -> Result<i32, D::Error> {
///         Ok(field.deserialize(deserializer)? - 1)
///     }
/// }
///
/// #[derive(Archive, Deserialize, Serialize)]
/// struct Example {
///     #[with(Incremented)]
///     a: i32,
///     // Another i32 field, but not incremented this time
///     b: i32,
/// }
///
/// let value = Example { a: 4, b: 9 };
///
/// let buf = to_bytes::<Error>(&value).unwrap();
///
/// let archived =
///     unsafe { access_unchecked::<Archived<Example>>(buf.as_ref()) };
/// // The wrapped field has been incremented
/// assert_eq!(archived.a, 5);
/// // ... and the unwrapped field has not
/// assert_eq!(archived.b, 9);
///
/// let deserialized =
///     deserialize::<Example, _, Infallible>(archived, &mut ()).always_ok();
/// // The wrapped field is back to normal
/// assert_eq!(deserialized.a, 4);
/// // ... and the unwrapped field is unchanged
/// assert_eq!(deserialized.b, 9);
/// ```
pub trait ArchiveWith<F: ?Sized> {
    /// The archived type of `Self` with `F`.
    type Archived: Portable;
    /// The resolver of a `Self` with `F`.
    type Resolver;

    /// Resolves the archived type using a reference to the field type `F`.
    fn resolve_with(
        field: &F,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    );
}

/// A variant of `Serialize` that works with `With` wrappers.
pub trait SerializeWith<F: ?Sized, S: Fallible + ?Sized>:
    ArchiveWith<F>
{
    /// Serializes the field type `F` using the given serializer.
    fn serialize_with(
        field: &F,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error>;
}

/// A variant of `Deserialize` that works with `With` wrappers.
pub trait DeserializeWith<F: ?Sized, T, D: Fallible + ?Sized> {
    /// Deserializes the field type `F` using the given deserializer.
    fn deserialize_with(field: &F, deserializer: &mut D)
        -> Result<T, D::Error>;
}

/// A wrapper to make a type immutable.
#[derive(Debug, Portable)]
#[archive(crate)]
#[repr(transparent)]
#[cfg_attr(feature = "bytecheck", derive(bytecheck::CheckBytes))]
pub struct Immutable<T: ?Sized>(T);

impl<T: ?Sized> Immutable<T> {
    /// Gets the underlying immutable value.
    pub fn value(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> Deref for Immutable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A generic wrapper that allows wrapping an `Option<T>`.
///
/// # Example
///
/// ```
/// use rkyv::{
///     with::{BoxedInline, Map},
///     Archive,
/// };
///
/// #[derive(Archive)]
/// struct Example<'a> {
///     #[with(Map<BoxedInline>)]
///     option: Option<&'a i32>,
///     #[with(Map<BoxedInline>)]
///     vec: Vec<&'a i32>,
/// }
/// ```
#[derive(Debug)]
pub struct Map<Archivable> {
    _type: PhantomData<Archivable>,
}

/// A type indicating relaxed atomic loads.
pub struct Relaxed;

/// A type indicating acquire atomic loads.
pub struct Acquire;

/// A type indicating sequentially-consistent atomic loads.
pub struct SeqCst;

/// A wrapper that archives an atomic by loading its value with a particular
/// ordering.
///
/// When serializing, the specified ordering will be used to load the value from
/// the source atomic. The underlying archived type is still a non-atomic value.
///
/// See [`AsAtomic`] for an unsafe alternative which archives as an atomic.
///
/// # Example
///
/// ```
/// # #[cfg(target_has_atomic = "32")]
/// use core::sync::atomic::AtomicU32;
///
/// use rkyv::{
///     with::{AtomicLoad, Relaxed},
///     Archive,
/// };
///
/// # #[cfg(target_has_atomic = "32")]
/// #[derive(Archive)]
/// struct Example {
///     #[with(AtomicLoad<Relaxed>)]
///     a: AtomicU32,
/// }
/// ```
#[derive(Debug)]
pub struct AtomicLoad<SO> {
    _phantom: PhantomData<SO>,
}

/// A wrapper that archives an atomic with an underlying atomic.
///
/// When serializing and deserializing, the specified ordering will be used to
/// load the value from the source atomic.
///
/// See [`AtomicLoad`] for a safe alternative.
///
/// # Safety
///
/// This wrapper is only safe to use when the backing memory for wrapped types
/// is mutable.
///
/// # Example
///
/// ```
/// # #[cfg(target_has_atomic = "32")]
/// use core::sync::atomic::AtomicU32;
///
/// use rkyv::{
///     with::{AsAtomic, Relaxed},
///     Archive,
/// };
///
/// # #[cfg(target_has_atomic = "32")]
/// #[derive(Archive)]
/// struct Example {
///     #[with(AsAtomic<Relaxed, Relaxed>)]
///     a: AtomicU32,
/// }
/// ```
#[derive(Debug)]
pub struct AsAtomic<SO, DO> {
    _phantom: PhantomData<(SO, DO)>,
}

/// A wrapper that serializes a reference inline.
///
/// References serialized with `Inline` cannot be deserialized because the
/// struct cannot own the deserialized value.
///
/// # Example
///
/// ```
/// use rkyv::{with::Inline, Archive};
///
/// #[derive(Archive)]
/// struct Example<'a> {
///     #[with(Inline)]
///     a: &'a i32,
/// }
/// ```
#[derive(Debug)]
pub struct Inline;

/// A wrapper that serializes a field into a box.
///
/// This functions similarly to [`BoxedInline`], but is for regular fields
/// instead of references.
///
/// # Example
///
/// ```
/// use rkyv::{with::Boxed, Archive};
///
/// #[derive(Archive)]
/// struct Example {
///     #[with(Boxed)]
///     a: i32,
///     #[with(Boxed)]
///     b: str,
/// }
/// ```
#[derive(Debug)]
pub struct Boxed;

/// A wrapper that serializes a reference as if it were boxed.
///
/// Unlike [`Inline`], unsized references can be serialized with `BoxedInline`.
///
/// References serialized with `BoxedInline` cannot be deserialized because the
/// struct cannot own the deserialized value.
///
/// # Example
///
/// ```
/// use rkyv::{with::BoxedInline, Archive};
///
/// #[derive(Archive)]
/// struct Example<'a> {
///     #[with(BoxedInline)]
///     a: &'a i32,
///     #[with(BoxedInline)]
///     b: &'a str,
/// }
/// ```
#[derive(Debug)]
pub struct BoxedInline;

/// A wrapper that attempts to convert a type to and from UTF-8.
///
/// Types like `OsString` and `PathBuf` aren't guaranteed to be encoded as
/// UTF-8, but they usually are anyway. Using this wrapper will archive them as
/// if they were regular `String`s.
///
/// Regular serializers don't support the custom error handling needed for this
/// type by default. To use this wrapper, a custom serializer with an error type
/// satisfying `<S as Fallible>::Error: From<AsStringError>` must be provided.
///
/// # Example
///
/// ```
/// use std::{ffi::OsString, path::PathBuf};
///
/// use rkyv::{with::AsString, Archive};
///
/// #[derive(Archive)]
/// struct Example {
///     #[with(AsString)]
///     os_string: OsString,
///     #[with(AsString)]
///     path: PathBuf,
/// }
/// ```
#[derive(Debug)]
pub struct AsString;

#[derive(Debug)]
struct InvalidStr;

impl fmt::Display for InvalidStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid UTF-8")
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for InvalidStr {}

/// A wrapper that locks a lock and serializes the value immutably.
///
/// This wrapper can panic under very specific circumstances when:
///
/// 1. `serialize_with` is called and succeeds in locking the value to serialize
///    it.
/// 2. Another thread locks the value and panics, poisoning the lock
/// 3. `resolve_with` is called and gets a poisoned value.
///
/// Unfortunately, it's not possible to work around this issue. If your code
/// absolutely must not panic under any circumstances, it's recommended that you
/// lock your values and then serialize them while locked.
///
/// Additionally, mutating the data protected by a mutex between the serialize
/// and resolve steps may cause undefined behavior in the resolve step. **Uses
/// of this wrapper should be considered unsafe** with the requirement that the
/// data not be mutated between these two steps.
///
/// Regular serializers don't support the custom error handling needed for this
/// type by default. To use this wrapper, a custom serializer with an error type
/// satisfying `<S as Fallible>::Error: From<LockError>` must be provided.
///
/// # Example
///
/// ```
/// use std::sync::Mutex;
///
/// use rkyv::{with::Lock, Archive};
///
/// #[derive(Archive)]
/// struct Example {
///     #[with(Lock)]
///     a: Mutex<i32>,
/// }
/// ```
#[derive(Debug)]
pub struct Lock;

#[derive(Debug)]
struct Poisoned;

impl fmt::Display for Poisoned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lock poisoned")
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for Poisoned {}

/// A wrapper that serializes a `Cow` as if it were owned.
///
/// # Example
///
/// ```
/// use std::borrow::Cow;
///
/// use rkyv::{with::AsOwned, Archive};
///
/// #[derive(Archive)]
/// struct Example<'a> {
///     #[with(AsOwned)]
///     a: Cow<'a, str>,
/// }
/// ```
#[derive(Debug)]
pub struct AsOwned;

/// A wrapper that serializes associative containers as a `Vec` of key-value
/// pairs.
///
/// This provides faster serialization for containers like `HashMap` and
/// `BTreeMap` by serializing the key-value pairs directly instead of building a
/// data structure in the buffer.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
///
/// use rkyv::{with::AsVec, Archive};
///
/// #[derive(Archive)]
/// struct Example {
///     #[with(AsVec)]
///     values: HashMap<String, u32>,
/// }
/// ```
#[derive(Debug)]
pub struct AsVec;

/// A wrapper that niches some type combinations.
///
/// A common type combination is `Option<Box<T>>`. By using a null pointer, the
/// archived version can save some space on-disk.
///
/// # Example
///
/// ```
/// use core::mem::size_of;
///
/// use rkyv::{with::Niche, Archive, Archived};
///
/// #[derive(Archive)]
/// struct BasicExample {
///     value: Option<Box<str>>,
/// }
///
/// #[derive(Archive)]
/// struct NichedExample {
///     #[with(Niche)]
///     value: Option<Box<str>>,
/// }
///
/// assert!(
///     size_of::<Archived<BasicExample>>()
///         > size_of::<Archived<NichedExample>>()
/// );
/// ```
#[derive(Debug)]
pub struct Niche;

/// A wrapper that converts a [`SystemTime`](::std::time::SystemTime) to a
/// [`Duration`](::std::time::Duration) since
/// [`UNIX_EPOCH`](::std::time::UNIX_EPOCH).
///
/// If the serialized time occurs before the UNIX epoch, serialization will
/// panic during `resolve`. The resulting archived time will be an
/// [`ArchivedDuration`](crate::time::ArchivedDuration) relative to the UNIX
/// epoch.
///
/// Regular serializers don't support the custom error handling needed for this
/// type by default. To use this wrapper, a custom serializer with an error type
/// satisfying `<S as Fallible>::Error: From<UnixTimestampError>` must be
/// provided.
///
/// # Example
///
/// ```
/// use rkyv::{Archive, with::UnixTimestamp};
/// use std::time::SystemTime;
///
/// #[derive(Archive)]
/// struct Example {
///     #[with(UnixTimestamp)]
///     time: SystemTime,
/// }
#[derive(Debug)]
pub struct UnixTimestamp;

/// Errors that can occur when serializing a [`UnixTimestamp`] wrapper.
#[derive(Debug)]
pub enum UnixTimestampError {
    /// The `SystemTime` occurred prior to the UNIX epoch.
    TimeBeforeUnixEpoch,
}

impl fmt::Display for UnixTimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time occurred before the UNIX epoch")
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for UnixTimestampError {}

/// A wrapper that allows serialize-unsafe types to be serialized.
///
/// Types like `Cell` and `UnsafeCell` may contain serializable types, but have
/// unsafe access semantics due to interior mutability. They may be safe to
/// serialize, but only under conditions that rkyv is unable to guarantee.
///
/// This wrapper enables serializing these types, and places the burden of
/// verifying that their access semantics are used safely on the user.
///
/// # Safety
///
/// Using this wrapper on types with interior mutability can create races
/// conditions or allow access to data in an invalid state if access semantics
/// are not followed properly. During serialization, the data must not be
/// modified.
///
/// # Example
///
/// ```
/// use core::cell::{Cell, UnsafeCell};
///
/// use rkyv::{with::Unsafe, Archive};
///
/// #[derive(Archive)]
/// struct Example {
///     #[with(Unsafe)]
///     cell: Cell<String>,
///     #[with(Unsafe)]
///     unsafe_cell: UnsafeCell<String>,
/// }
/// ```
#[derive(Debug)]
pub struct Unsafe;

/// A wrapper that skips serializing a field.
///
/// Skipped fields must implement `Default` to be deserialized.
///
/// # Example
///
/// ```
/// use rkyv::{with::Skip, Archive};
///
/// #[derive(Archive)]
/// struct Example {
///     #[with(Skip)]
///     a: u32,
/// }
/// ```
#[derive(Debug)]
pub struct Skip;

/// A wrapper that clones the contents of `Arc` and `Rc` pointers.
#[derive(Debug)]
pub struct Cloned;
