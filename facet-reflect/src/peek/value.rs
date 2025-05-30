use core::cmp::Ordering;
use facet_core::{Def, Facet, Opaque, OpaqueConst, Shape, TypeNameOpts, ValueVTable};

use crate::{ReflectError, ScalarType};

use super::{PeekEnum, PeekList, PeekMap, PeekSmartPointer, PeekStruct};

/// A unique identifier for a peek value
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId {
    pub(crate) shape: &'static Shape,
    pub(crate) ptr: *const u8,
}

impl ValueId {
    pub(crate) fn new(shape: &'static Shape, ptr: *const u8) -> Self {
        Self { shape, ptr }
    }
}

impl core::fmt::Display for ValueId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}@{:p}", self.shape, self.ptr)
    }
}

impl core::fmt::Debug for ValueId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

/// Lets you read from a value (implements read-only [`ValueVTable`] proxies)
#[derive(Clone, Copy)]
pub struct Peek<'mem> {
    /// Underlying data
    pub(crate) data: OpaqueConst<'mem>,

    /// Shape of the value
    pub(crate) shape: &'static Shape,
}

impl<'mem> Peek<'mem> {
    /// Creates a new `PeekValue` instance for a value of type `T`.
    pub fn new<T: Facet + 'mem>(t: &'mem T) -> Self {
        Self {
            data: OpaqueConst::new(t as *const T),
            shape: T::SHAPE,
        }
    }

    /// Creates a new `PeekValue` instance without checking the type.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't check if the provided data
    /// and shape are compatible. The caller must ensure that the data is valid
    /// for the given shape.
    pub unsafe fn unchecked_new(data: OpaqueConst<'mem>, shape: &'static Shape) -> Self {
        Self { data, shape }
    }

    /// Returns the vtable
    #[inline(always)]
    fn vtable(&self) -> &'static ValueVTable {
        self.shape.vtable
    }

    /// Returns a unique identifier for this value, usable for cycle detection
    pub fn id(&self) -> ValueId {
        ValueId::new(self.shape, self.data.as_byte_ptr())
    }

    /// Returns true if the two values are pointer-equal
    #[inline]
    pub fn ptr_eq(&self, other: &Peek<'_>) -> bool {
        self.data.as_byte_ptr() == other.data.as_byte_ptr()
    }

    /// Returns true if this scalar is equal to the other scalar
    ///
    /// # Returns
    ///
    /// `false` if equality comparison is not supported for this scalar type
    #[inline]
    pub fn eq(&self, other: &Peek<'_>) -> Option<bool> {
        unsafe {
            self.shape
                .vtable
                .eq
                .map(|eq_fn| eq_fn(self.data, other.data))
        }
    }

    /// Compares this scalar with another and returns their ordering
    ///
    /// # Returns
    ///
    /// `None` if comparison is not supported for this scalar type
    #[inline]
    pub fn partial_cmp(&self, other: &Peek<'_>) -> Option<Ordering> {
        unsafe {
            self.shape
                .vtable
                .partial_ord
                .and_then(|partial_ord_fn| partial_ord_fn(self.data, other.data))
        }
    }

    /// Hashes this scalar
    ///
    /// # Returns
    ///
    /// `false` if hashing is not supported for this scalar type, `true` otherwise
    #[inline(always)]
    pub fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) -> bool {
        unsafe {
            if let Some(hash_fn) = self.shape.vtable.hash {
                let hasher_opaque = Opaque::new(hasher);
                hash_fn(self.data, hasher_opaque, |opaque, bytes| {
                    opaque.as_mut::<H>().write(bytes)
                });
                true
            } else {
                false
            }
        }
    }

    /// Returns the type name of this scalar
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a `core::fmt::Formatter`
    /// * `opts` - The `TypeNameOpts` to use for formatting
    ///
    /// # Returns
    ///
    /// The result of the type name formatting
    #[inline(always)]
    pub fn type_name(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        opts: TypeNameOpts,
    ) -> core::fmt::Result {
        (self.shape.vtable.type_name)(f, opts)
    }

    /// Returns the shape
    #[inline(always)]
    pub const fn shape(&self) -> &'static Shape {
        self.shape
    }

    /// Returns the data
    #[inline(always)]
    pub const fn data(&self) -> OpaqueConst<'mem> {
        self.data
    }

    /// Get the scalar type if set.
    pub fn scalar_type(&self) -> Option<ScalarType> {
        ScalarType::try_from_shape(self.shape)
    }

    /// Read the value from memory into a Rust value.
    ///
    /// # Panics
    ///
    /// Panics if the shape doesn't match the type `T`.
    pub fn get<T: Facet>(&self) -> Result<&T, ReflectError> {
        if self.shape != T::SHAPE {
            Err(ReflectError::WrongShape {
                expected: self.shape,
                actual: T::SHAPE,
            })
        } else {
            Ok(unsafe { self.data.get::<T>() })
        }
    }

    /// Tries to identify this value as a struct
    pub fn into_struct(self) -> Result<PeekStruct<'mem>, ReflectError> {
        if let Def::Struct(def) = self.shape.def {
            Ok(PeekStruct { value: self, def })
        } else {
            Err(ReflectError::WasNotA {
                expected: "struct",
                actual: self.shape,
            })
        }
    }

    /// Tries to identify this value as an enum
    pub fn into_enum(self) -> Result<PeekEnum<'mem>, ReflectError> {
        if let Def::Enum(def) = self.shape.def {
            Ok(PeekEnum { value: self, def })
        } else {
            Err(ReflectError::WasNotA {
                expected: "enum",
                actual: self.shape,
            })
        }
    }

    /// Tries to identify this value as a map
    pub fn into_map(self) -> Result<PeekMap<'mem>, ReflectError> {
        if let Def::Map(def) = self.shape.def {
            Ok(PeekMap { value: self, def })
        } else {
            Err(ReflectError::WasNotA {
                expected: "map",
                actual: self.shape,
            })
        }
    }

    /// Tries to identify this value as a list
    pub fn into_list(self) -> Result<PeekList<'mem>, ReflectError> {
        if let Def::List(def) = self.shape.def {
            Ok(PeekList { value: self, def })
        } else {
            Err(ReflectError::WasNotA {
                expected: "list",
                actual: self.shape,
            })
        }
    }

    /// Tries to identify this value as a smart pointer
    pub fn into_smart_pointer(self) -> Result<PeekSmartPointer<'mem>, ReflectError> {
        if let Def::SmartPointer(def) = self.shape.def {
            Ok(PeekSmartPointer { value: self, def })
        } else {
            Err(ReflectError::WasNotA {
                expected: "smart pointer",
                actual: self.shape,
            })
        }
    }

    /// Tries to identify this value as an option
    pub fn into_option(self) -> Result<super::PeekOption<'mem>, ReflectError> {
        if let Def::Option(def) = self.shape.def {
            Ok(super::PeekOption { value: self, def })
        } else {
            Err(ReflectError::WasNotA {
                expected: "option",
                actual: self.shape,
            })
        }
    }
}

impl core::fmt::Display for Peek<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(display_fn) = self.vtable().display {
            unsafe { display_fn(self.data, f) }
        } else {
            write!(f, "⟨{}⟩", self.shape)
        }
    }
}

impl core::fmt::Debug for Peek<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(debug_fn) = self.vtable().debug {
            unsafe { debug_fn(self.data, f) }
        } else {
            write!(f, "⟨{}⟩", self.shape)
        }
    }
}

impl core::cmp::PartialEq for Peek<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.shape != other.shape {
            return false;
        }
        let eq_fn = match self.shape.vtable.eq {
            Some(eq_fn) => eq_fn,
            None => return false,
        };
        unsafe { eq_fn(self.data, other.data) }
    }
}

impl core::cmp::PartialOrd for Peek<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.shape != other.shape {
            return None;
        }
        let partial_ord_fn = self.shape.vtable.partial_ord?;
        unsafe { partial_ord_fn(self.data, other.data) }
    }
}

impl core::hash::Hash for Peek<'_> {
    fn hash<H: core::hash::Hasher>(&self, hasher: &mut H) {
        if let Some(hash_fn) = self.shape.vtable.hash {
            let hasher_opaque = Opaque::new(hasher);
            unsafe {
                hash_fn(self.data, hasher_opaque, |opaque, bytes| {
                    opaque.as_mut::<H>().write(bytes)
                })
            };
        } else {
            panic!("Hashing is not supported for this shape");
        }
    }
}
