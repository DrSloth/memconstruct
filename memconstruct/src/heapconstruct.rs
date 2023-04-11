use alloc::alloc::{alloc as do_alloc, dealloc as do_dealloc};
use core::{
    alloc::Layout,
    mem,
    panic::AssertUnwindSafe,
    ptr,
};

use crate::{util, MemConstruct, MemConstructConstructor};

pub trait HeapConstruct<T> {
    unsafe fn try_heapconstruct_fallible_raw<E, F: FnOnce(*mut T) -> Result<(), E>>(
        construct: F,
    ) -> Result<Self, HeapConstructError<E>>
    where
        Self: Sized;
}

pub trait HeapConstructExt<T>: HeapConstruct<T>
where
    T: MemConstruct,
{
    #[inline(always)]
    fn try_heapconstruct_fallible<E, F>(construct: F) -> Result<Self, HeapConstructError<E>>
    where
        Self: Sized,
        F: FnOnce(T::Constructor) -> Result<T::ConstructorFinishedToken, E>,
    {
        unsafe {
            Self::try_heapconstruct_fallible_raw(|ptr| {
                construct(T::Constructor::new(ptr)).map(|_| ())
            })
        }
    }

    #[inline(always)]
    fn try_heapconstruct<F>(construct: F) -> Result<Self, HeapConstructError<()>>
    where
        Self: Sized,
        F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
    {
        unsafe {
            Self::try_heapconstruct_fallible_raw(|ptr| {
                construct(T::Constructor::new(ptr));
                Ok(())
            })
        }
    }

    #[inline(always)]
    fn heapconstruct_fallible<E, F>(construct: F) -> Result<Self, HeapConstructError<E>>
    where
        Self: Sized,
        F: FnOnce(T::Constructor) -> Result<T::ConstructorFinishedToken, E>,
    {
        let res = unsafe {
            Self::try_heapconstruct_fallible_raw(|ptr| {
                construct(T::Constructor::new(ptr)).map(|_| ())
            })
        };

        match res {
            Ok(val) => Ok(val),
            Err(HeapConstructError::AllocationFailure) => panic!("Allocation failed"),
            Err(e) => Err(e),
        }
    }

    #[inline(always)]
    fn heapconstruct<F>(construct: F) -> Self
    where
        Self: Sized,
        F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
    {
        let res = unsafe {
            Self::try_heapconstruct_fallible_raw(|ptr| {
                construct(T::Constructor::new(ptr));
                Ok(())
            })
        };

        match res {
            Ok(val) => val,
            Err(HeapConstructError::AllocationFailure) => panic!("Allocation failed"),
            Err(HeapConstructError::ConstructPanicked(e)) => util::resume_unwind(e),
            Err(HeapConstructError::ConstructFailed(())) => {
                unreachable!("The construction is not fallible")
            }
        }
    }
}

impl<T, S> HeapConstructExt<T> for S
where
    T: MemConstruct,
    S: HeapConstruct<T>,
{
}

impl<T> HeapConstruct<T> for Box<T>
where
    T: MemConstruct,
{
    #[inline(always)]
    unsafe fn try_heapconstruct_fallible_raw<E, F: FnOnce(*mut T) -> Result<(), E>>(
        construct: F,
    ) -> Result<Self, HeapConstructError<E>> {
        if mem::size_of::<T>() == 0usize {
            let res = util::catch_unwind(AssertUnwindSafe(|| {
                // Constructors for ZSTs MUST ignore the given pointer here because `alloc`
                // will fail for allocations of size 0
                construct(ptr::null_mut())?;
                Ok(())
            }));

            return match res {
                Ok(Ok(_)) => Ok(T::new_boxed_zst()),
                Ok(Err(e)) => Err(HeapConstructError::ConstructFailed(e)),
                Err(e) => Err(HeapConstructError::ConstructPanicked(e)),
            };
        }

        let layout = Layout::new::<T>();
        let ptr = unsafe { do_alloc(layout) as *mut T };

        if ptr.is_null() {
            return Err(HeapConstructError::AllocationFailure);
        }

        let res = util::catch_unwind(AssertUnwindSafe(|| {
            construct(ptr)?;
            Ok(())
        }));

        match res {
            Ok(Ok(_)) => unsafe { Ok(Box::from_raw(ptr)) },
            Ok(Err(e)) => {
                unsafe { do_dealloc(ptr as *mut u8, layout) };
                Err(HeapConstructError::ConstructFailed(e))
            }
            Err(e) => {
                unsafe { do_dealloc(ptr as *mut u8, layout) };
                Err(HeapConstructError::ConstructPanicked(e))
            }
        }
    }
}

pub fn construct_box<T: MemConstruct, F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken>(
    construct: F,
) -> Box<T> {
    Box::heapconstruct(construct)
}

pub enum HeapConstructError<E> {
    AllocationFailure,
    ConstructPanicked(util::UnwindError),
    ConstructFailed(E),
}
