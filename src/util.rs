macro_rules! try_multiple_time {
    ($e:expr) => (
        {
            let mut error_timer = 0;
            let mut res = $e;
            while res.is_err() {
                ::std::thread::sleep(::std::time::Duration::from_millis(100));
                error_timer += 1;
                if error_timer > 10 {
                    break;
                }
                res = $e;
            }
            res
        }
    )
}
