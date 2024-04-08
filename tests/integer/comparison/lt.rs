#[no_mangle]
pub fn f() -> i32 {
    if !(1 < 2) {
        1
    }
    else if (1 < 1) {
        2
    }
    else if (2 < 1) {
        3
    }
    else {
        0
    }
}