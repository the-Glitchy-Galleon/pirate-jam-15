use bevy::prelude::*;

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

#[derive(Reflect)]
struct Tween<T> {
    a: T,
    b: T,
}

impl<T: Clone> Clone for Tween<T> {
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
        }
    }
}

impl<T> Tween<T> {
    pub fn new(a: T, b: T) -> Self {
        Self { a, b }
    }
}
impl Tween<Vec3> {
    pub fn lerp(&self, t: f32) -> Vec3 {
        self.a.lerp(self.b, t)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Reflect)]
enum BackForth {
    Back,
    Forth,
}

#[derive(Component, Clone, Reflect)]
pub struct TweenBackAndForth {
    paths: Vec<Vec3>,
    target: usize,
    progress: f32,
    mode: BackForth,
    tween: Tween<Vec3>,
}

impl TweenBackAndForth {
    pub fn new(paths: Vec<Vec3>) -> Self {
        let tween = match paths.len() {
            0 => Tween::new(Vec3::ZERO, Vec3::ZERO),
            1 => Tween::new(paths[0], paths[0]),
            _ => Tween::new(paths[0], paths[1]),
        };
        let target = match paths.len() {
            0 => 0,
            1 => 0,
            _ => 1,
        };
        Self {
            paths,
            target,
            progress: 0.0,
            mode: BackForth::Forth,
            tween,
        }
    }

    pub fn tick(&mut self, dt: f32) -> Vec3 {
        let paths_len = self.paths.len();
        if paths_len == 0 {
            return Vec3::ZERO;
        }
        if paths_len == 1 {
            return self.paths[0];
        }

        self.progress += dt;
        if self.progress >= 1.0 {
            let old_target = self.target;
            let (mode, next_target) = match (
                self.mode,
                self.target == 0,
                self.target == self.paths.len() - 1,
            ) {
                (BackForth::Forth, _, true) => (BackForth::Back, self.target - 1),
                (BackForth::Forth, _, _) => (BackForth::Forth, self.target + 1),
                (BackForth::Back, true, _) => (BackForth::Forth, self.target + 1),
                (BackForth::Back, _, _) => (BackForth::Back, self.target - 1),
            };
            self.mode = mode;
            self.target = next_target;
            self.progress = 0.0;
            self.tween = Tween::new(self.paths[old_target], self.paths[next_target]);
            self.paths[old_target]
        } else {
            self.tween.lerp(self.progress)
        }
    }
}
