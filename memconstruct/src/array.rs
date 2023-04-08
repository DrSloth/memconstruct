use std::marker::PhantomData;

use crate::{MemConstruct, MemConstructConstructor};

pub struct ArrayTok;

pub struct ArrayMemConstructor<Tok, T, const N: usize> {
    ptr: *mut [T; N],
    boo_scary: PhantomData<Tok>,
}

unsafe impl<T, const N: usize> MemConstruct for [T; N]
where
    T: MemConstruct,
{
    type Constructor = ArrayMemConstructor<ArrayTok, T, N>;
    type ConstructorFinishedToken = ArrayMemConstructor<(), T, N>;
}

unsafe impl<T, const N: usize> MemConstructConstructor for ArrayMemConstructor<ArrayTok, T, N> {
    type Target = [T; N];

    unsafe fn new(ptr: *mut Self::Target) -> Self {
        Self {
            ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl<T, const N: usize> ArrayMemConstructor<ArrayTok, T, N> {
    pub fn memconstruct_all<F: FnMut(T::Constructor) -> T::ConstructorFinishedToken>(
        self,
        mut f: F,
    ) -> ArrayMemConstructor<(), T, N>
    where
        T: MemConstruct,
    {
        let mut cur = self.ptr as *mut T;
        for _ in 0..N {
            // SAFETY: The pointer will be inside the allocation of the array. We will break
            // after offseting N times so the last access we do is arr[N-1]
            unsafe {
                f(T::Constructor::new(cur.into()));
            }
            // SAFETY: The type system guarantees that we have N entries in the array
            unsafe {
                cur = cur.offset(1);
            }
        }

        ArrayMemConstructor {
            ptr: self.ptr,
            boo_scary: PhantomData::default(),
        }
    }

    pub unsafe fn with_ptr(self, f: impl FnOnce(*mut [T;N])) -> ArrayMemConstructor<(), T, N> {
        f(self.ptr);
        ArrayMemConstructor { ptr: self.ptr, boo_scary: PhantomData::default() }
    }
}
