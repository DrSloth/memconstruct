fn main() {
    let b = memconstruct::construct_box::<[i32; 2_200_000], _>(|c| c.memconstruct_all(|c| c.set(42)));
    println!("{:?}", b);
}
