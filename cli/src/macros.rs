macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            }
        }
    };
}
