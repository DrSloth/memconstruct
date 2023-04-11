// Most functions here should be inlined accross crates as they are basic buildings block, likely
// for very performance sensitive environments
#![allow(clippy::inline_always)]

// pub mod alloc;
pub mod array;
pub mod primitive;
pub mod heapconstruct;

mod util;

extern crate alloc;

pub use memconstruct_macros::MemConstruct;

pub use heapconstruct::{construct_box, HeapConstruct, HeapConstructExt};

use core::mem::MaybeUninit;

/// Trait implemented for types that can be safely constructed anywhere in memory.
///
/// Implementing this trait is very dangerous, you should use the 
/// [`MemConstruct`](memconstruct_macros::MemConstruct) derive macro instead.
/// 
/// If you are interested in the inner mechanisms of this take a look at the crate docs.
pub unsafe trait MemConstruct {
    type Constructor: MemConstructConstructor<Target = Self>;
    type ConstructorFinishedToken;

    #[doc(hidden)]
    fn new_boxed_zst() -> Box<Self>
    where
        Self: Sized,
    {
        unreachable!("Only zsts should implement this function")
    }
}

/// A type used to construct a heap constructable object.
///
/// This trait is unsafe to implement and shouldn't be manually implemented. A safe implementation
/// of this is generated when you derive `MemConstruct` for your type.
////
/// # Implementation for Structs:
/// For normal structs a `set` function is generated for every field, each of these `set` functions
/// has to be called exactly once, this is checked via typestate.
///
/// # Implementation for ZSTs:
/// For `ZSTs` the generated constructor has no functions and is always "ready". The construct
/// functions will still be called for ZSTs.
pub unsafe trait MemConstructConstructor {
    type Target;

    /// Create a new `MemConstructConstructor`
    ///
    /// # Safety
    ///
    /// The pointer has to be a valid, non dangling pointer. The pointer must be well
    /// aligned and non null if not documented otherwise.
    unsafe fn new(ptr: *mut Self::Target) -> Self;
}

/// Construct a value behind a pointer.
///
/// The pointer has to be non null, non dangling and well well aligned if the documentation of T or
/// T::Constructor doesn't state otherwise
///
/// # Panics
/// 
/// This function will panic if the passed `construct` function panics. The value behind `ptr` is 
/// then unspecified.
#[inline(always)]
pub unsafe fn construct_raw<
    T: MemConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    ptr: *mut T,
    construct: F,
) {
    construct(T::Constructor::new(ptr));
}

/// Safely construct a value behind a [`MaybeUninit`]
///
/// # Panics
///
/// This function panics if the passed `construct` function panics. The value inside the 
/// [`MaybeUninit`] is then unspecified and shouldn't be assumed to be initialized.
#[inline(always)]
pub fn construct_maybe_uninit<
    T: MemConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    uninit: &mut MaybeUninit<T>,
    construct: F,
) {
    unsafe { construct_raw(uninit.as_mut_ptr(), construct) }
}

