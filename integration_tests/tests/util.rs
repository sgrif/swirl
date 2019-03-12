pub trait UnwrapFromDrop<T> {
    fn unwrap_from_drop(self) -> T;
}

impl<T, E> UnwrapFromDrop<T> for Result<T, E>
where
    T: Default,
    E: std::fmt::Debug,
{
    fn unwrap_from_drop(self) -> T {
        use std::thread::panicking;

        match self {
            Ok(t) => t,
            Err(e) => {
                if panicking() {
                    eprintln!("called `Result::unwrap()` on an `Err` value: {:?}", e);
                    T::default()
                } else {
                    panic!("called `Result::unwrap()` on an `Err` value: {:?}", e)
                }
            }
        }
    }
}
