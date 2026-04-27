#[macro_export]
macro_rules! log_on_change {
    ($val:expr) => {{
        use std::cell::RefCell;
        thread_local! {
            static LAST: RefCell<Option<String>> = RefCell::new(None);
        }
        let current = format!("{:?}", $val);
        LAST.with(|last| {
            let mut last = last.borrow_mut();
            if last.as_deref() != Some(&current) {
                println!("[{}:{}] {} = {}", file!(), line!(), stringify!($val), current);
                *last = Some(current);
            }
        });
    }};
}