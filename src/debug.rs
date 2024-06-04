#[cfg(debug_assertions)]
#[macro_export]
macro_rules! _dbg {
    ($($arg:tt)*) => {
        dbg!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! _dbg {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! _eprintln {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            eprintln!($($arg)*);
        }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! _eprintln {
    ($($arg:tt)*) => {};
}
