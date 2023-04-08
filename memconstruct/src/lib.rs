pub mod alloc;

mod array;
mod primitive;

pub use memconstruct_macros::MemConstruct;

pub use alloc::construct_box;

use std::mem::MaybeUninit;

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

pub unsafe fn init_ptr<
    T: MemConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    ptr: *mut T,
    func: F,
) {
    func(T::Constructor::new(ptr));
}

pub fn init_maybe_uninit<
    T: MemConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    uninit: &mut MaybeUninit<T>,
    func: F,
) {
    unsafe { init_ptr(uninit.as_mut_ptr(), func) }
}

