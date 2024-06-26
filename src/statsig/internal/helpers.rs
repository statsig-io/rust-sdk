pub trait UsizeExt {
    fn post_inc(&mut self) -> Self;
}

impl UsizeExt for usize {
    fn post_inc(&mut self) -> Self {
        let was = *self;
        *self += 1;
        was
    }
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($res: expr, $code: expr) => {
        match $res {
            Some(v) => v,
            None => return $code,
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_noop {
    ($res: expr) => {
        match $res {
            Some(v) => v,
            None => return,
        }
    };
}
