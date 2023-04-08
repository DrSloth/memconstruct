```rust
#[derive(Debug, HeapConstruct)]
struct Example {
  x: i32,
  y: i32,
}

#[test]
fn test() {
    let b = construct_box::<Example, _>(|ctor| ctor.set_x(10).set_y(2));

    println!("{:?}", b);
}
```

```rust
#[derive(Debug)]
struct Example {
    x: i32,
    y: i32,
}

unsafe impl HeapConstruct for Example {
    type Constructor = ExampleConstructor<HeapConstructExamplex, HeapConstructExampley>;
    type ConstructorFinishedToken = ExampleConstructor<(), ()>;
}

pub struct ExampleConstructor<T0, T1> {
    ptr: *mut Example,
    boo_scary: PhantomData<(T0, T1)>,
}

unsafe impl HeapConstructConstructor
    for ExampleConstructor<HeapConstructExamplex, HeapConstructExampley>
{
    type Target = Example;

    fn new(ptr: *mut Self::Target) -> Self {
        Self {
            ptr,
            boo_scary: PhantomData::default(),
        }
    }
}

impl<T0> ExampleConstructor<HeapConstructExamplex, T0> {
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

impl<T0> ExampleConstructor<(), T0> {
    pub fn set_x(self, x: i32) -> Self {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).x).write(x);
        }
        self
    }
}

impl<T0> ExampleConstructor<T0, HeapConstructExampley> {
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

impl<T0> ExampleConstructor<T0, ()> {
    pub fn set_y(self, y: i32) -> Self {
        unsafe {
            ptr::addr_of_mut!((*self.ptr).y).write(y);
        }
        self
    }
}

struct HeapConstructExamplex;
struct HeapConstructExampley;

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

/// A failure occured while trying to construct a value inside an allocation on the heap.
pub enum AllocError {
    /// The passed construction function paniced
    Paniced(Box<dyn Any + Send>),
    /// The allocation failed.
    AllocFailure,
}

#[test]
fn test() {
    let b = construct_box::<Example, _>(|ctor| ctor.set_x(10).set_y(2));

    println!("{:?}", b);
}
```
