use std;
use serde::{Deserialize, Serialize};

#[derive(Eq, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Millis {
    rep: i32,
}

impl Millis {
    pub fn zero() -> Millis {
        return Millis::from_i32(0);
    }

    pub fn from_f32(v: f32) -> Millis{
        return Millis{rep: (1000.0 * v) as i32};
    }

    pub fn from_i32(v: i32) -> Millis {
        return Millis{rep: 1000 * v};
    }
}

impl std::cmp::PartialEq for Millis {
    fn eq(&self, other: &Millis) -> bool {
        self.rep == other.rep
    }
}

impl std::cmp::PartialOrd for Millis {
    fn partial_cmp(&self, other: &Millis) -> Option<std::cmp::Ordering> {
        Some(self.rep.cmp(&other.rep))
    }
}

impl std::cmp::Ord for Millis {
    fn cmp(&self, other: &Millis) -> std::cmp::Ordering {
        self.rep.cmp(&other.rep)
    }
}

impl std::ops::Add for Millis {
    type Output = Millis;

    fn add(self, other: Millis) -> Millis {
        return Millis{rep: self.rep + other.rep};
    }
}

impl std::ops::Sub for Millis {
    type Output = Millis;

    fn sub(self, other: Millis) -> Millis {
        return Millis{rep: self.rep - other.rep};
    }
}

impl std::fmt::Display for Millis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (self.rep as f32) / 1000.0)
    }
}
