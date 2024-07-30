#[macro_export]
macro_rules! run_n {
    ($n:expr, $expr:expr) => {
        {
            seq_macro::seq!(N in 0..$n {
                (
                    #($expr,)*
                )
            })
        }
    };
}
