#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    InPowi(i32),
    OutPowi(i32),
    InOutPowi(i32),
    InPowf(f64),
    OutPowf(f64),
    InOutPowf(f64),
}

impl Easing {
    pub fn apply(&self, mut x: f64) -> f64 {
        match self {
            Easing::Linear => x,
            Easing::InPowi(power) => x.powi(*power),
            Easing::OutPowi(power) => 1.0 - Self::InPowi(*power).apply(1.0 - x),
            Easing::InOutPowi(power) => {
                x *= 2.0;
                if x < 1.0 {
                    0.5 * Self::InPowi(*power).apply(x)
                } else {
                    x = 2.0 - x;
                    0.5 * (1.0 - Self::InPowi(*power).apply(x)) + 0.5
                }
            }
            Easing::InPowf(power) => x.powf(*power),
            Easing::OutPowf(power) => 1.0 - Self::InPowf(*power).apply(1.0 - x),
            Easing::InOutPowf(power) => {
                x *= 2.0;
                if x < 1.0 {
                    0.5 * Self::InPowf(*power).apply(x)
                } else {
                    x = 2.0 - x;
                    0.5 * (1.0 - Self::InPowf(*power).apply(x)) + 0.5
                }
            }
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}
