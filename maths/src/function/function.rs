pub trait Function {
    fn apply(&self, x: f64, y: f64) -> f64;
    fn identity(&self) -> f64;
    fn symbol(&self) -> &'static str;
}

macro_rules! define {
    ($f:ident, $op:tt, $id:expr) => {
        pub struct $f;

        impl Function for $f {
            fn apply(&self, x: f64, y: f64) -> f64 {
                x $op y
            }

            fn identity(&self) -> f64 {
                $id
            }

            fn symbol(&self) -> &'static str {
                stringify!($op)
            }
        }
    };
}

define!(Add, +, 0.0);
define!(Mul, *, 1.0);
