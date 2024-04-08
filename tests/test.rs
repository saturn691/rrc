#[link(name= "foo", kind = "static")]
extern "C" {
    fn f() -> i32;
}

fn main() {
    let result: i32 = unsafe { f() };
    std::process::exit(result);
}