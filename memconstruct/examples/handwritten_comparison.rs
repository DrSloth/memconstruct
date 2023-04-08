use std::{marker::PhantomData, ptr};

use memconstruct::{MemConstruct, MemConstructConstructor};

#[derive(Debug)]
struct Example {
    x: i32,
    y: i32,
}

unsafe impl MemConstruct for Example {
    type Constructor = ExampleConstructor<MemConstructExamplex, MemConstructExampley>;
    type ConstructorFinishedToken = ExampleConstructor<(), ()>;
}

pub struct ExampleConstructor<T0, T1> {
    ptr: *mut Example,
    boo_scary: PhantomData<(T0, T1)>,
}

unsafe impl MemConstructConstructor
    for ExampleConstructor<MemConstructExamplex, MemConstructExampley>
{
    type Target = Example;

    unsafe fn new(ptr: *mut Self::Target) -> Self {
        Self {
            ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl<T0> ExampleConstructor<MemConstructExamplex, T0> {
    pub fn set_x(self, x: i32) -> ExampleConstructor<(), T0> {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).x).write(x);
        }
        ExampleConstructor::<(), T0> {
            ptr: self.ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl<T0> ExampleConstructor<T0, MemConstructExampley> {
    pub fn set_y(self, y: i32) -> ExampleConstructor<T0, ()> {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).y).write(y);
        }
        ExampleConstructor::<T0, ()> {
            ptr: self.ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

struct MemConstructExamplex;
struct MemConstructExampley;

fn main() {
    let b = memconstruct::construct_box::<Example, _>(|ctor| ctor.set_x(10).set_y(2));

    println!("{:?}", b);
}
