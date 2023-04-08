fn main() {
    let b = Box::new([42i32;2_200_000]);
    println!("{:?}", b);
}