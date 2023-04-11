use memconstruct::{HeapConstructExt, MemConstruct};

#[derive(MemConstruct, Debug, PartialEq)]
struct Morello;

#[test]
fn construct_boxed_zst() {
    let f = |z| z;
    let m0 = memconstruct::construct_box::<Morello, _>(f);
    println!("{:?}", m0);
    let m1 = Box::heapconstruct(f);
    assert_eq!(m0, m1);
}

#[derive(MemConstruct, Debug, PartialEq)]
struct Forello {
    x: i32,
    hello_world: f32,
    m: [u8; 4],
}

#[test]
fn construct_boxed_struct() {
    let construct = |c: <Forello as MemConstruct>::Constructor| {
        c.set_x(10).set_hello_world(2.3).set_m([5u8; 4])
    };
    let f0 = memconstruct::construct_box::<Forello, _>(construct);
    println!("{:?}", f0);
    let f1 = Box::heapconstruct(construct);
    assert_eq!(f0, f1);
}

#[derive(MemConstruct, Debug, PartialEq)]
struct Borello(i32, f32, [u8; 4]);

#[test]
fn construct_boxed_tuple_struct() {
    let construct =
        |c: <Borello as MemConstruct>::Constructor| c.set_0(42).set_1(6.9).set_2([4u8; 4]);
    let b0 = memconstruct::construct_box::<Borello, _>(construct);
    println!("{:?}", b0);
    let b1 = Box::heapconstruct(construct);
    assert_eq!(b0, b1);
}

type Arr = [i32; 20];

#[test]
fn construct_boxed_array() {
    let construct = |c: <Arr as MemConstruct>::Constructor| c.memconstruct_all(|c| c.set(42));
    let arr0 = memconstruct::construct_box::<Arr, _>(construct);
    println!("{:?}", arr0);
    let arr1 = Box::heapconstruct(construct);
    assert_eq!(arr0, arr1);
}

#[test]
fn box_heap_construct() {
    let m = Box::<Morello>::heapconstruct(|m| m);
    assert_eq!(&*m, &Morello);
    let f =
        Box::<Forello>::heapconstruct(|f| f.set_x(10).set_hello_world(12.3).set_m([1, 9, 8, 7]));
    assert_eq!(
        &*f,
        &Forello {
            x: 10,
            hello_world: 12.3,
            m: [1, 9, 8, 7]
        }
    );
    let b = Box::<Borello>::heapconstruct(|b| b.set_0(10).set_1(12.3).set_2([1, 9, 8, 7]));
    assert_eq!(&*b, &Borello(10, 12.3, [1,9,8,7]));
    let arr = Box::<Arr>::heapconstruct(|b| b.set_all(|_| 10));
    assert_eq!(&*arr, &[10i32;20]);
}
