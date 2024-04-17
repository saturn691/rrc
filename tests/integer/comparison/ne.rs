#[no_mangle]
pub fn f() -> i32 {
    if (0 != 0) {
        0
    } else if (1 != 1) {
        1
    } else {
        2
    }
}