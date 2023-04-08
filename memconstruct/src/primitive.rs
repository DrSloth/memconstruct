use std::marker::PhantomData;

use crate::{MemConstruct, MemConstructConstructor};

pub struct PrimitiveI32Tok;

pub struct PrimitiveI32Constructor<Tok> {
    ptr: *mut i32,
    boo_scary: PhantomData<Tok>,
}

unsafe impl MemConstruct for i32 {
    type Constructor = PrimitiveI32Constructor<PrimitiveI32Tok>;
    type ConstructorFinishedToken = PrimitiveI32Constructor<()>;
}

unsafe impl MemConstructConstructor for PrimitiveI32Constructor<PrimitiveI32Tok> {
    type Target = i32;
    unsafe fn new(ptr: *mut Self::Target) -> Self {
        Self {
            ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl PrimitiveI32Constructor<PrimitiveI32Tok> {
    pub fn set(self, val: i32) -> PrimitiveI32Constructor<()> {
        // SAFETY: This operation is only unsafe if the rules of [`MemConstructConstructor::new`]
        // were broken which can only be done in unsafe code.
        unsafe {
            self.ptr.write(val);
        }

        PrimitiveI32Constructor {
            ptr: self.ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

