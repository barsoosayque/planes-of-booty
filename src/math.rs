use euclid::{Angle, Point2D, Size2D, UnknownUnit, Vector2D};

pub type Anglef = Angle<f32>;
pub type Size2f = Size2D<f32, UnknownUnit>;
pub type Vec2f = Vector2D<f32, UnknownUnit>;
pub type Point2f = Point2D<f32, UnknownUnit>;
#[derive(Default, Debug)]
pub struct Circle2f {
    pub position: Point2f,
    pub radius: f32,
}
impl Circle2f {
    pub fn new(position: Point2f, radius: f32) -> Self { Self { position, radius } }

    pub fn contains(&self, p: Point2f) -> bool { p.distance_to(self.position) < self.radius }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Default for Direction {
    fn default() -> Self { Self::North }
}

impl Direction {
    // pub fn as_vec2f(&self) -> Vec2f {
    //     match self {
    //         Self::North => (0.0, -1.0),
    //         Self::East => (1.0, 0.0),
    //         Self::South => (0.0, 1.0),
    //         Self::West => (-1.0, 0.0),
    //     }
    //     .into()
    // }

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

#[macro_export]
macro_rules! directional {
    ($v:expr => $north:expr, $east:expr, $south:expr, $west:expr) => {
        match $v {
            Direction::North => $north,
            Direction::East => $east,
            Direction::South => $south,
            Direction::West => $west,
        }
    };
}
