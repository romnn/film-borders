#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
    () => {
        eprintln!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                eprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($arg:tt)*) => {{
        println!($($arg)*);
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    ($x:expr) => {
        std::convert::identity($x)
    };
    ($($arg:tt)*) => {};
}
