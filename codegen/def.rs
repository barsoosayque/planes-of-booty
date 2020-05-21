use heck::CamelCase;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::collections::BTreeMap as Map;
use std::fmt;
use uuid::Uuid;

#[derive(Deserialize, Default)]
pub struct EntityDef {
    #[serde(skip)]
    pub name: String,
    pub components: Map<String, ComponentDef>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Default)]
pub struct ComponentDef {
    pub default: bool,
    pub parts: Map<String, PartValue>,
}

#[derive(Clone, Debug)]
pub enum PartValue {
    Seq(Vec<PartValue>),
    Str(String),
    Num(f32),
    Bool(bool),
    Image(String),
    Faction(String),
    Directional {
        north: Box<PartValue>,
        east: Box<PartValue>,
        west: Box<PartValue>,
        south: Box<PartValue>,
    },
    Single {
        value: Box<PartValue>,
    },
    Size {
        width: f32,
        height: f32,
    },
    Vec {
        x: f32,
        y: f32,
    },
    Body {
        status: String,
        mass: f32,
    },
    Box {
        uuid: Uuid,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Collide {
        sensor: bool,
        shape: Box<PartValue>,
    },
}

impl std::fmt::Display for PartValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartValue::Seq(vec) => write!(
                f,
                "vec![{}].drain(..).collect()",
                vec.iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            PartValue::Str(value) => write!(f, "{}", value),
            PartValue::Num(value) => write!(f, "{}f32", value),
            PartValue::Bool(value) => write!(f, "{}", value),
            PartValue::Image(path) => write!(
                f,
                "assets.get::<crate::assets::ImageAsset>(\"{}\", ctx).unwrap()",
                path
            ),
            PartValue::Faction(faction) => write!(f, "component::FactionId::{}", faction),
            PartValue::Directional {
                north,
                east,
                south,
                west,
            } => write!(
                f,
                "component::DirOrSingle::Directional{{north:{},east:{},south:{},west:{}}}",
                north, east, south, west
            ),
            PartValue::Single { value } => {
                write!(f, "component::DirOrSingle::Single{{value:{}}}", value)
            }
            PartValue::Size { width, height } => {
                write!(f, "crate::math::Size2f::new({}f32, {}f32)", width, height)
            }
            PartValue::Vec { x, y } => write!(f, "crate::math::Vec2f::new({}f32, {}f32)", x, y),
            PartValue::Body { .. } => write!(f, "body"),
            PartValue::Box { uuid, .. } => write!(f, "box_{}.clone()", uuid.to_simple()),
            PartValue::Collide { sensor, shape } => {
                let first_shape = match &**shape {
                    PartValue::Single { value } => value,
                    PartValue::Directional { north, .. } => north,
                    _ => panic!("No valid shapes defined to create a collide"),
                };

                write!(
                    f,
                    "(world.write_resource::<resource::PhysicWorld>().colliders.insert(
                        nphysics2d::object::ColliderDesc::new({}).sensor({})
                         .build(nphysics2d::object::BodyPartHandle(body, 0))
                    ), {})",
                    first_shape, sensor, shape
                )
            }
        }
    }
}

impl PartValue {
    pub fn initialize(&self) -> Option<String> {
        match self {
            PartValue::Body { mass, status } => Some(format!(
                "let body = world.write_resource::<resource::PhysicWorld>()
                    .bodies.insert(nphysics2d::object::RigidBodyDesc::new()
                        .mass({}f32).status(nphysics2d::object::BodyStatus::{}).build()
                    );",
                mass,
                status.to_camel_case()
            )),
            PartValue::Box {
                uuid,
                x,
                y,
                width,
                height,
            } => Some(format!(
                "let box_{} = nphysics2d::ncollide2d::shape::ShapeHandle::new(
                        nphysics2d::ncollide2d::shape::ConvexPolygon::try_from_points(&[
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),
                        ]).unwrap()
                );",
                uuid.to_simple(),
                x,
                y,
                x + width,
                y,
                x + width,
                y + height,
                x,
                y + height,
            )),
            _ => None,
        }
    }
}

pub fn get_view_from<'a>(
    def: &'a EntityDef,
    component_name: &str,
    asset_part: &str,
) -> Option<(&'a PartValue, &'a PartValue)> {
    def.components.get(component_name).map(|comp| {
        (
            comp.parts
                .get(asset_part)
                .map(|part| match part {
                    PartValue::Single { value } => value.as_ref(),
                    PartValue::Directional { north, .. } => north.as_ref(),
                    _ => panic!("{} should be either single or directional", component_name),
                })
                .expect(&format!(
                    "{} field is missing for component {} in {}",
                    asset_part, component_name, def.name
                )),
            comp.parts.get("size").expect(&format!(
                "width field is missing component for {} in {}",
                component_name, def.name
            )),
        )
    })
}

struct ComponentDefVisitor;
impl<'de> Visitor<'de> for ComponentDefVisitor {
    type Value = ComponentDef;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
        let mut def = ComponentDef::default();
        while let Some((key, value)) = access.next_entry::<String, PartValue>()? {
            match key.as_ref() {
                "__default" => {
                    if let PartValue::Bool(value) = value {
                        def.default = value;
                    }
                }
                _ => {
                    def.parts.insert(key, value);
                }
            };
        }

        Ok(def)
    }
}

struct PartValueVisitor;
impl<'de> Visitor<'de> for PartValueVisitor {
    type Value = PartValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("map with one element, bool, string or number or seq of listed items")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut v: Vec<PartValue> = vec![];
        while let Some(value) = seq.next_element::<PartValue>()? {
            v.push(value)
        }
        Ok(PartValue::Seq(v))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(PartValue::Str(value.to_owned()))
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(PartValue::Bool(value))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(PartValue::Num(value as f32))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(PartValue::Num(value as f32))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        Ok(PartValue::Num(value as f32))
    }

    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
        let mut buffer = Map::<String, PartValue>::new();

        while let Some((key, value)) = access.next_entry::<String, PartValue>()? {
            match (key.as_ref(), value) {
                ("image", PartValue::Str(value)) => return Ok(PartValue::Image(value)),
                ("faction", PartValue::Str(value)) => return Ok(PartValue::Faction(value)),
                (key, value) => {
                    buffer.insert(key.to_owned(), value);
                }
            }
        }

        if let (Some(north), Some(east), Some(south), Some(west)) = (
            buffer.remove("north"),
            buffer.remove("east"),
            buffer.remove("south"),
            buffer.remove("west"),
        ) {
            Ok(PartValue::Directional {
                north: Box::new(north),
                south: Box::new(south),
                east: Box::new(east),
                west: Box::new(west),
            })
        } else if let Some(value) = buffer.remove("single") {
            Ok(PartValue::Single {
                value: Box::new(value),
            })
        } else if let (Some(PartValue::Num(width)), Some(PartValue::Num(height))) =
            (buffer.remove("width"), buffer.remove("height"))
        {
            Ok(PartValue::Size { width, height })
        } else if let (Some(PartValue::Num(x)), Some(PartValue::Num(y))) =
            (buffer.remove("x"), buffer.remove("y"))
        {
            Ok(PartValue::Vec { x, y })
        } else if let (Some(PartValue::Num(mass)), Some(PartValue::Str(status))) =
            (buffer.remove("mass"), buffer.remove("status"))
        {
            Ok(PartValue::Body { mass, status })
        } else if let (Some(PartValue::Vec { x, y }), Some(PartValue::Size { width, height })) =
            (buffer.remove("pos"), buffer.remove("size"))
        {
            let id = format!("{}-{}-{}-{}", x, y, width, height);
            Ok(PartValue::Box {
                // Generate uuid based on x-y-width-height
                uuid: Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_ref()),
                x,
                y,
                width,
                height,
            })
        } else if let (Some(PartValue::Bool(sensor)), Some(shape)) =
            (buffer.remove("sensor"), buffer.remove("shape"))
        {
            Ok(PartValue::Collide {
                sensor,
                shape: Box::new(shape),
            })
        } else {
            Err(de::Error::custom(format!(
                "No special fields defined. Here is buffer: {:?}",
                buffer
            )))
        }
    }
}

impl<'de> Deserialize<'de> for ComponentDef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ComponentDefVisitor)
    }
}

impl<'de> Deserialize<'de> for PartValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(PartValueVisitor)
    }
}
