use memconstruct::HeapConstructExt;

fn main() {
    let b = Box::<[i32; 2_200_000]>::heapconstruct(|c| c.set_all(|_| 42));
    println!("{:?}", b);
}
