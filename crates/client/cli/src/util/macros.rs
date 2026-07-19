#[macro_export]
macro_rules! write_line {
    ($($arg:tt)*) => {
        if !$crate::constants::QUIET.get().unwrap() {
            println!($($arg)*);
        }
    };
}