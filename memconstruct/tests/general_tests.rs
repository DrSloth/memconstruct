use memconstruct::MemConstruct;

#[derive(MemConstruct, Debug)]
struct Morello;

#[test]
fn construct_boxed_zst() {
    let morello = memconstruct::construct_box::<Morello, _>(|z| z);
    println!("{:?}", morello);
}

#[derive(MemConstruct, Debug)]
struct Forello {
    x: i32,
    hello_world: f32,
    m: [u8; 4],
}

#[test]
fn construct_boxed_struct() {
    let forello = memconstruct::construct_box::<Forello, _>(|c| {
        c.set_x(10).set_hello_world(2.3).set_m([5u8; 4])
    });
    println!("{:?}", forello);
}

#[derive(MemConstruct, Debug)]
struct Borello(i32, f32, [u8; 4]);

#[test]
fn construct_boxed_tuple_struct() {
    let borello = memconstruct::construct_box::<Borello, _>(|c| {
        c.set_field0(42).set_field1(6.9).set_field2([4u8; 4])
    });
    println!("{:?}", borello)
}
