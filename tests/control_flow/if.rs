#[no_mangle]
pub fn f() -> i32 {
    let x: i32 = 5;
    if x > 0 {
        1
    } else {
        0
    }
}