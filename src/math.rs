use euclid::{Point2D, UnknownUnit, Vector2D};

pub type Vec2f = Vector2D<f32, UnknownUnit>;
pub type Point2f = Point2D<f32, UnknownUnit>;

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[derive(Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Default for Direction {
    fn default() -> Self {
        Self::North
    }
}

impl Direction {
    pub fn as_vec2f(&self) -> Vec2f {
        match self {
            Self::North => (0.0, -1.0),
            Self::East => (1.0, 0.0),
            Self::South => (0.0, 1.0),
            Self::West => (-1.0, 0.0),
        }
        .into()
    }

    pub fn from_vec2f(vec: &Vec2f) -> Self {
        if vec.x.abs() > vec.y.abs() {
            if vec.x > 0.0 {
                Direction::East 
            } else {
                Direction::West
            }
        } else {
            if vec.y > 0.0 {
                Direction::South 
            } else {
                Direction::North
            }
        }
    }
}
