// Ignore errors when outputing errors
// TODO: should only print if opt.quiet == false
#[macro_export]
macro_rules! error {
    ($($x:tt)*) => {
        let _ = writeln!(std::io::stderr(), $($x)*);
    }
}

pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}
