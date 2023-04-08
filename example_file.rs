use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem::MaybeUninit,
    panic, ptr,
};

pub trait HeapConstruct {
    type Constructor: HeapConstructConstructor<Target = Self>;
    type ConstructorFinishedToken;
}

pub trait HeapConstructConstructor {
    type Target;

    fn new(ptr: *mut Self::Target) -> Self;
}

#[derive(Debug)]
struct Bosono {
    x: i32,
    y: i32,
}

impl HeapConstruct for Bosono {
    type Constructor = BosonoConstructor<HeapConstructBosonox, HeapConstructBosonoy>;
    type ConstructorFinishedToken = BosonoConstructor<(), ()>;
}

struct BosonoConstructor<T0, T1> {
    ptr: *mut Bosono,
    boo_scary: PhantomData<(T0, T1)>,
}

impl HeapConstructConstructor for BosonoConstructor<HeapConstructBosonox, HeapConstructBosonoy> {
    type Target = Bosono;

    fn new(ptr: *mut Self::Target) -> Self {
        Self {
            ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl<T0> BosonoConstructor<HeapConstructBosonox, T0> {
    fn set_x(self, x: i32) -> BosonoConstructor<(), T0> {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).x).write(x);
        }
        BosonoConstructor::<(), T0> {
            ptr: self.ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl<T0> BosonoConstructor<(), T0> {
    fn set_x(self, x: i32) -> Self {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).x).write(x);
        }
        self
    }
}

impl<T0> BosonoConstructor<T0, HeapConstructBosonoy> {
    fn set_y(self, y: i32) -> BosonoConstructor<T0, ()> {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).y).write(y);
        }
        BosonoConstructor::<T0, ()> {
            ptr: self.ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl<T0> BosonoConstructor<T0, ()> {
    fn set_y(self, y: i32) -> Self {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).y).write(y);
        }
        self
    }
}

struct HeapConstructBosonox;
struct HeapConstructBosonoy;

pub unsafe fn init_ptr<
    T: HeapConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    ptr: *mut T,
    func: F,
) {
    func(T::Constructor::new(ptr));
}

pub fn init_maybe_uninit<
    T: HeapConstruct,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken,
>(
    uninit: &mut MaybeUninit<T>,
    func: F,
) {
    unsafe { init_ptr(uninit.as_mut_ptr(), func) }
}

pub fn create_box<
    T: HeapConstruct + panic::RefUnwindSafe,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken + panic::UnwindSafe,
>(
    func: F,
) -> Box<T> {
    let layout = Layout::new::<T>();
    let ptr = unsafe { alloc::alloc(layout) as *mut T };
    let res = panic::catch_unwind(move || {
        unsafe { init_ptr(ptr, func) };
    });

    match res {
        Ok(_) => unsafe { Box::from_raw(ptr) },
        Err(e) => {
            unsafe { alloc::dealloc(ptr as *mut u8, layout) };
            panic::resume_unwind(e)
        }
    }
}

pub fn create_box_uncaught<
    T: HeapConstruct + panic::RefUnwindSafe,
    F: FnOnce(T::Constructor) -> T::ConstructorFinishedToken + panic::UnwindSafe,
>(
    func: F,
) -> Box<T> {
    let ptr = unsafe { alloc::alloc(Layout::new::<T>()) as *mut T };
    unsafe { init_ptr(ptr, func); }
    unsafe { Box::from_raw(ptr) }
}

fn main() {
    let b = create_box::<Bosono, _>(|ctor| ctor.set_x(10).set_y(2));

    println!("{:?}", b);
}
