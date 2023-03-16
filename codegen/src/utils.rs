#![macro_use]

macro_rules! unwrap {
    ($err:expr) => {
        match $err {
            Ok(res) => res,
            Err(err) => return err.to_compile_error().into(),
        }
    };
}
