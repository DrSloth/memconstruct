/// Memconstruct Implementation on arrays
///
/// TODO write about implementation on arrays
use core::{marker::PhantomData, mem, panic::AssertUnwindSafe, ptr};

use crate::{primitive::MemconstructPrimitive, util, MemConstruct, MemConstructConstructor};

pub struct ArrayTok;

pub struct ArrayMemConstructor<Tok, T, const N: usize> {
    ptr: *mut [T; N],
    boo_scary: PhantomData<Tok>,
}

unsafe impl<T, const N: usize> MemConstruct for [T; N] {
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
    #[inline(always)]
    pub fn set_all<F: FnMut(usize) -> T>(self, mut f: F) -> ArrayMemConstructor<(), T, N> {
        if mem::needs_drop::<T>() {
            unsafe { self.init_all_with_drop(|ptr, i| *ptr = f(i)) }
        } else {
            unsafe { self.init_all_nodrop(|ptr, i| *ptr = f(i)) }
        }
    }

    #[inline(always)]
    pub fn memconstruct_all<F: FnMut(T::Constructor) -> T::ConstructorFinishedToken>(
        self,
        mut f: F,
    ) -> ArrayMemConstructor<(), T, N>
    where
        T: MemConstruct,
    {
        if mem::needs_drop::<T>() {
            unsafe { self.init_all_with_drop(|ptr, _| { f(T::Constructor::new(ptr)); }) }
        } else {
            unsafe { self.init_all_nodrop(|ptr, _| { f(T::Constructor::new(ptr)); }) }
        }
    }

    #[inline(always)]
    unsafe fn init_all_with_drop<F: FnMut(*mut T, usize)>(
        self,
        mut f: F,
    ) -> ArrayMemConstructor<(), T, N> where
    {
        let mut i = 0usize;
        let mut cur = self.ptr as *mut T;
        let res = util::catch_unwind(AssertUnwindSafe(|| {
            for _ in 0..N {
                // The pointer will be inside the allocation of the array. We will break
                // after offseting N times so the last access we do is arr[N-1]
                f(cur, i);
                // SAFETY: The type system guarantees that we have N entries in the array
                unsafe {
                    cur = cur.offset(1);
                }
                // We know we can't overflow as N can be usize::MAX at max and we only increment up
                // to N times.
                i = i.wrapping_add(1);
            }
        }));

        match res {
            Ok(_) => ArrayMemConstructor {
                ptr: self.ptr,
                boo_scary: PhantomData::default(),
            },
            Err(e) => {
                let mut cur = self.ptr as *mut T;
                for _ in 0..i {
                    // SAFETY: We only drop the values we know have been initialized
                    unsafe {
                        ptr::drop_in_place(cur);
                    }
                    // SAFETY: We drop at most i elements i is only incremented up until N so we know
                    // that i < N holds
                    unsafe {
                        cur = cur.offset(1);
                    }
                }
                util::resume_unwind(e)
            }
        }
    }

    #[inline(always)]
    unsafe fn init_all_nodrop<F: FnMut(*mut T, usize)>(
        self,
        mut f: F,
    ) -> ArrayMemConstructor<(), T, N>
    {
        let mut cur = self.ptr as *mut T;
        for i in 0..N {
            // The pointer will be inside the allocation of the array. We will break
            // after offseting N times so the last access we do is arr[N-1]
            f(cur, i);
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

    #[inline(always)]
    pub fn memset(self, byte: u8) -> ArrayMemConstructor<(), T, N>
    where
        T: MemconstructPrimitive,
    {
        match mem::size_of::<T>().checked_mul(N) {
            Some(count) => {
                // SAFETY: We write sizeof(T) * N bytes, the type systems says we have that many bytes
                unsafe {
                    self.ptr.write_bytes(byte, count);
                }
                ArrayMemConstructor {
                    ptr: self.ptr,
                    boo_scary: PhantomData::default(),
                }
            }
            None => {
                todo!("Split work across multiple memsets");
            }
        }
    }

    pub unsafe fn with_ptr(self, f: impl FnOnce(*mut [T; N])) -> ArrayMemConstructor<(), T, N> {
        f(self.ptr);
        ArrayMemConstructor {
            ptr: self.ptr,
            boo_scary: PhantomData::default(),
        }
    }
}
