/// Memconstruct implementation on primitives
///
/// TODO write about implementation on primitives

use std::marker::PhantomData;

use crate::{MemConstruct, MemConstructConstructor};

/// A primtive that can be constructed by using "`memset`" ([`core::ptr::write_bytes`])
pub unsafe trait MemconstructPrimitive {}

macro_rules! primitive_impl {
    ($($prim:tt)*) => {
        $(
            paste::paste! {
                unsafe impl MemconstructPrimitive for $prim {}
                
                pub struct [<Primitive $prim ConstructionToken>];

                pub struct [<Primitive $prim MemConstructor>] <Tok> {
                    ptr: *mut $prim,
                    boo_scary: PhantomData<Tok>,
                }

                unsafe impl MemConstruct for $prim {
                    type Constructor = [<Primitive $prim MemConstructor>]
                        <[<Primitive $prim ConstructionToken>]>;
                    type ConstructorFinishedToken = [<Primitive $prim MemConstructor>] <()>;
                }

                unsafe impl MemConstructConstructor for [<Primitive $prim MemConstructor>] 
                    <[<Primitive $prim ConstructionToken>]> 
                {
                    type Target = $prim;
                    
                    unsafe fn new(ptr: *mut $prim) -> Self {
                        Self {
                            ptr,
                            boo_scary: PhantomData::default(),
                        }
                    }
                }


                impl [<Primitive $prim MemConstructor>] <[<Primitive $prim ConstructionToken>]> {
                    pub fn set(self, val: $prim) -> [<Primitive $prim MemConstructor>] <()> {
                        // SAFETY: This operation is only unsafe if the rules of
                        // [`MemConstructConstructor::new`] were broken which can only be
                        // done in unsafe code.
                        unsafe {
                            self.ptr.write(val);
                        }

                        [<Primitive $prim MemConstructor>] :: <()> {
                            ptr: self.ptr,
                            boo_scary: PhantomData::default(),
                        }
                    }
                }
            }
        )*
    };
}

primitive_impl! {u8 i8 u16 i16 u32 i32 u64 i64 char f32 f64}

