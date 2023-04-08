//! Safely construct boxes with values that are directly constructed on the heap.

use std::{
    alloc::{self, Layout},
    any::Any,
    mem, panic, ptr,
};

use super::{MemConstruct, MemConstructConstructor};

/// Safely construct a box that holds a value of `T` that is constructed directly on the heap,
/// skipping the construction on the stack.
///
/// This constructions catches panics inside the passes `construct` function and deallocates the
/// previously allocated memory, it resumes the unwind after deallocating. If the passed closure
/// are not [`UnwindSafe`](std::panic::Unwindsafe) or the type T is not [`RefUnwindSafe`] consider
/// using the [`construct_box_uncaught`] function which will leak the memory on panic.
///
/// # Panics:
///
/// This function panics if allocation of values of type T fail.
pub fn construct_box<
    T: MemConstruct + panic::RefUnwindSafe,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken + panic::UnwindSafe,
>(
    construct: F,
) -> Box<T> {
    match try_construct_box(construct) {
        Ok(b) => b,
        Err(AllocError::AllocFailure) => {
            panic!("Allocation of box failed");
        }
        Err(AllocError::Paniced(e)) => {
            panic::resume_unwind(e);
        }
    }
}

/// Safely construct a box that holds a value of `T` that is constructed directly on the heap,
/// skipping the construction on the stack.
///
/// This constructtions catches panics inside the passes `construct` function and deallocates the
/// previously allocated memory, it resumes the unwind after deallocating. If the passed closure
/// are not [`UnwindSafe`](std::panic::Unwindsafe) or the type T is not [`RefUnwindSafe`] consider
/// using the [`construct_box_uncaught`] function which will leak the memory on panic.
///
/// # Panics:
///
/// This function panics if allocation of values of type T fail.
pub fn construct_box_uncaught<
    T: MemConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    construct: F,
) -> Box<T> {
    match try_construct_box_uncaught(construct) {
        Ok(b) => b,
        Err(AllocError::AllocFailure) => {
            panic!("Allocation of box failed");
        }
        Err(AllocError::Paniced(e)) => {
            panic::resume_unwind(e);
        }
    }
}

/// Try to construct a box on the heap. This function should never panic but return appropriate
/// errors.
///
/// # Errors:
///
/// If the `construct` function panics [`memconstruct::AllocError::Paniced`] will be returned.
/// On allocation failure (global allocator returns null pointer on alloc) this function returns
/// [`memconstruct::AllocError::AllocFailure`] will be returned.
pub fn try_construct_box<
    T: MemConstruct + panic::RefUnwindSafe,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken + panic::UnwindSafe,
>(
    construct: F,
) -> Result<Box<T>, AllocError> {
    if mem::size_of::<T>() == 0usize {
        let res = panic::catch_unwind(|| {
            // SAFETY: Constructors for ZSTs MUST ignore the given pointer here because `alloc`
            // will fail for allocations of size 0
            construct(unsafe { T::Constructor::new(ptr::null_mut()) });
        });

        return match res {
            Ok(()) => Ok(T::new_boxed_zst()),
            Err(e) => Err(AllocError::Paniced(e)),
        };
    }

    let layout = Layout::new::<T>();
    let ptr = unsafe { alloc::alloc(layout) as *mut T };

    if ptr.is_null() {
        return Err(AllocError::AllocFailure);
    }

    let res = panic::catch_unwind(move || {
        unsafe { crate::init_ptr(ptr, construct) };
    });

    match res {
        Ok(_) => unsafe { Ok(Box::from_raw(ptr)) },
        Err(e) => {
            unsafe { alloc::dealloc(ptr as *mut u8, layout) };
            Err(AllocError::Paniced(e))
        }
    }
}

pub fn try_construct_box_uncaught<
    T: MemConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    construct: F,
) -> Result<Box<T>, AllocError> {
    if mem::size_of::<T>() == 0usize {
        // SAFETY: Constructors for ZSTs MUST ignore the given pointer here because `alloc`
        // will fail for allocations of size 0
        construct(unsafe { T::Constructor::new(ptr::null_mut()) });
        return Ok(T::new_boxed_zst());
    }

    let ptr = unsafe { alloc::alloc(Layout::new::<T>()) as *mut T };

    if ptr.is_null() {
        return Err(AllocError::AllocFailure);
    }

    unsafe {
        crate::init_ptr(ptr, construct);
    }
    unsafe { Ok(Box::from_raw(ptr)) }
}

/// A failure occured while trying to construct a value inside an allocation on the heap.
pub enum AllocError {
    /// The passed construction function paniced
    Paniced(Box<dyn Any + Send>),
    /// The allocation failed.
    AllocFailure,
}
