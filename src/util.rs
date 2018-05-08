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

#[derive(Serialize, Deserialize, Clone)]
pub struct ClampFunction {
    pub min_value: f32,
    pub max_value: f32,
    pub min_t: f32,
    pub max_t: f32
}

impl ClampFunction {
    pub fn compute(&self, t: f32) -> f32 {
        debug_assert!(self.min_t < self.max_t);
        if t <= self.min_t {
            self.min_value
        } else if t >= self.max_t {
            self.max_value
        } else {
            (t - self.min_t) / (self.max_t - self.min_t)
                * (self.max_value - self.min_value) + self.min_value
        }
    }
}
