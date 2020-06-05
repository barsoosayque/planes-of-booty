use heck::{CamelCase, ShoutySnakeCase};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{collections::BTreeMap as Map, fmt};
use uuid::Uuid;

#[derive(Deserialize, Default)]
pub struct SpawnGroupDef {
    #[serde(skip)]
    pub name: String,
    pub start: u32,
    pub grow: f32,
    #[serde(default)]
    pub weighted: Vec<WeightedEntityDef>,
}
#[derive(Deserialize, Default)]
pub struct WeightedEntityDef {
    pub id: String,
    pub weight: u8,
}

#[derive(Deserialize, Default)]
pub struct ArenaDef {
    #[serde(skip)]
    pub name: String,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub entities: Vec<ArenaEntityDef>,
    #[serde(default)]
    pub spawn_points: Vec<SpawnPointDef>,
}
#[derive(Deserialize, Default)]
pub struct ArenaEntityDef {
    pub id: String,
    pub pos: Point,
}
#[derive(Deserialize, Default)]
pub struct SpawnPointDef {
    pub radius: f32,
    pub pos: Point,
}
#[derive(Deserialize, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Default)]
pub struct EntityDef {
    #[serde(skip)]
    pub name: String,
    pub components: Map<String, ComponentDef>,
    #[serde(default)]
    pub shared_components: Map<String, ComponentDef>,
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
    Numf(f32),
    Numi(i32),
    Bool(bool),
    Image(String),
    Faction(String),
    CollisionGroup(String),
    Rarity(String),
    AttackPattern(String, Map<String, PartValue>),
    ConsumableBehaviour(String, Map<String, PartValue>),
    ShapeshifterForms(Vec<PartValue>),
    Item(String),
    Range(Box<PartValue>, Box<PartValue>),
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
        collision_membership: Vec<PartValue>,
        shape: Box<PartValue>,
        hitbox: Option<Box<PartValue>>,
    },
}

impl std::fmt::Display for PartValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartValue::Seq(vec) => write!(
                f,
                "vec![{}].drain(..).collect()",
                vec.iter().map(|x| format!("{}", x)).collect::<Vec<String>>().join(",")
            ),
            PartValue::Str(value) => write!(f, "\"{}\"", value),
            PartValue::Numf(value) => write!(f, "{}f32", value),
            PartValue::Numi(value) => write!(f, "{}", value),
            PartValue::Bool(value) => write!(f, "{}", value),
            PartValue::Image(path) => write!(f, "assets.get::<crate::assets::ImageAsset>(\"{}\", ctx).unwrap()", path),
            PartValue::Faction(faction) => write!(f, "component::FactionId::{}", faction.to_camel_case()),
            PartValue::Item(id) => write!(f, "crate::item::spawn_{}(world, ctx, assets).into()", id),
            PartValue::CollisionGroup(group) => {
                write!(f, "(component::CollisionGroup::{} as usize)", group.to_camel_case())
            },
            PartValue::ShapeshifterForms(..) => write!(f, "&SHAPESHIFTER_FORMS"),
            PartValue::Rarity(rarity) => write!(f, "component::Rarity::{}", rarity.to_camel_case()),
            PartValue::AttackPattern(pattern, ..) => write!(f, "&ATTACK_PATTERN_{}", pattern.to_shouty_snake_case()),
            PartValue::ConsumableBehaviour(consumable, ..) => {
                write!(f, "&CONSUMABLE_BEHAVIOUR_{}", consumable.to_shouty_snake_case())
            },
            PartValue::Range(start, end) => write!(f, "({}..={}).into()", start, end),
            PartValue::Directional { north, east, south, west } => write!(
                f,
                "component::DirOrSingle::Directional{{north:{},east:{},south:{},west:{}}}",
                north, east, south, west
            ),
            PartValue::Single { value } => write!(f, "component::DirOrSingle::Single{{value:{}}}", value),
            PartValue::Size { width, height } => write!(f, "crate::math::Size2f::new({}f32,{}f32)", width, height),
            PartValue::Vec { x, y } => write!(f, "crate::math::Vec2f::new({}f32,{}f32)", x, y),
            PartValue::Body { .. } => write!(f, "body"),
            PartValue::Box { uuid, .. } => write!(f, "box_{}.clone()", uuid.to_simple()),
            PartValue::Collide { shape, hitbox, .. } => {
                if let Some(hitbox) = hitbox {
                    write!(f, "component::PhysicColliders{{real:(collider,{}),hitbox:Some((hitbox,{}))}}", shape, hitbox)
                } else {
                    write!(f, "component::PhysicColliders{{real:(collider,{}),hitbox:None}}", shape)
                }
            },
        }
    }
}

impl PartValue {
    pub fn is_dependent(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Collide { .. }, Self::Body { .. }) => true,
            (Self::Collide { .. }, Self::Box { .. }) => true,
            (Self::Collide { .. }, Self::Faction { .. }) => true,
            _ => false,
        }
    }

    pub fn initialize(&self) -> Option<String> {
        match self {
            PartValue::ConsumableBehaviour(consumable, fields) => {
                let (var_name, struct_name) =
                    (consumable.to_shouty_snake_case(), format!("crate::item::{}", consumable.to_camel_case()));
                if fields.is_empty() {
                    Some(format!("const CONSUMABLE_BEHAVIOUR_{}: {} = {};", var_name, struct_name, struct_name))
                } else {
                    Some(format!(
                        "const CONSUMABLE_BEHAVIOUR_{}: {} = {}{{{}}};",
                        var_name,
                        struct_name,
                        struct_name,
                        fields.iter().map(|(k, v)| format!("{}:{}", k, v)).collect::<Vec<String>>().join(",")
                    ))
                }
            },
            PartValue::AttackPattern(pattern, fields) => {
                let (var_name, struct_name) =
                    (pattern.to_shouty_snake_case(), format!("crate::attack::{}", pattern.to_camel_case()));
                if fields.is_empty() {
                    Some(format!("const ATTACK_PATTERN_{}: {} = {};", var_name, struct_name, struct_name))
                } else {
                    Some(format!(
                        "const ATTACK_PATTERN_{}: {} = {}{{{}}};",
                        var_name,
                        struct_name,
                        struct_name,
                        fields.iter().map(|(k, v)| format!("{}:{}", k, v)).collect::<Vec<String>>().join(",")
                    ))
                }
            },
            PartValue::ShapeshifterForms(forms) => {
                let (mut lines, vars): (Vec<_>, Vec<_>) = forms
                    .iter()
                    .filter_map(|form| {
                        if let PartValue::Str(struct_name) = form {
                            let var_name = struct_name.to_shouty_snake_case();
                            Some((struct_name, var_name))
                        } else {
                            None
                        }
                    })
                    .map(|(struct_name, var_name)| {
                        (
                            (format!("const {}: {} = {};", var_name, struct_name, struct_name)),
                            (format!("&{}", var_name)),
                        )
                    })
                    .unzip();
                lines.sort_unstable();
                lines.dedup();

                Some(format!(
                    "{}\nconst SHAPESHIFTER_FORMS: [&'static dyn component::ShapeshifterForm; {}] = [{}];",
                    lines.join("\n"),
                    vars.len(),
                    vars.join(",")
                ))
            },
            PartValue::Body { mass, status } => Some(format!(
                "let body = world.write_resource::<resource::PhysicWorld>()\
                    .bodies.insert(nphysics2d::object::RigidBodyDesc::new()\
                        .mass({}f32).status(nphysics2d::object::BodyStatus::{}).build()\
                    );",
                mass,
                status.to_camel_case()
            )),
            PartValue::Collide { sensor, collision_membership, shape, hitbox } => {
                let collision_membership_str =
                    collision_membership.iter().map(|v| format!("{}", v)).collect::<Vec<String>>().join(",");

                let hitbox = if let Some(hitbox) = hitbox {
                    let first_shape = match &**hitbox {
                        PartValue::Single { value } => value,
                        PartValue::Directional { north, .. } => north,
                        _ => panic!("No valid shapes defined to create a hitbox"),
                    };

                    format!(
                        "let hitbox = world.write_resource::<resource::PhysicWorld>()\
                        .colliders.insert(nphysics2d::object::ColliderDesc::new({})\
                            .sensor(true)\
                            .collision_groups(\
                                nphysics2d::ncollide2d::pipeline::object::CollisionGroups::new()\
                                .with_membership(&[component::CollisionGroup::Hitbox as usize, {}])\
                            ).build(nphysics2d::object::BodyPartHandle(body, 0))\
                        );\n",
                        first_shape, collision_membership_str
                    )
                } else {
                    "".to_owned()
                };

                let collider = {
                    let first_shape = match &**shape {
                        PartValue::Single { value } => value,
                        PartValue::Directional { north, .. } => north,
                        _ => panic!("No valid shapes defined to create a collide"),
                    };
                    format!(
                        "let collider = world.write_resource::<resource::PhysicWorld>()\
                    .colliders.insert(nphysics2d::object::ColliderDesc::new({})\
                        .sensor({})\
                        .material(nphysics2d::material::MaterialHandle::new(\
                            nphysics2d::material::BasicMaterial::new(0.8, 0.8)\
                        ))\
                        .collision_groups(\
                            nphysics2d::ncollide2d::pipeline::object::CollisionGroups::new()\
                            .with_membership(&[{}])\
                        ).build(nphysics2d::object::BodyPartHandle(body, 0))\
                    );",
                        first_shape, sensor, collision_membership_str
                    )
                };
                Some(format!("{}{}", hitbox, collider))
            },
            PartValue::Box { uuid, x, y, width, height } => Some(format!(
                "let box_{} = nphysics2d::ncollide2d::shape::ShapeHandle::new(\
                        nphysics2d::ncollide2d::shape::ConvexPolygon::try_from_points(&[\
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),\
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),\
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),\
                            nphysics2d::nalgebra::Point2::new({}f32, {}f32),\
                        ]).unwrap()\
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

    pub fn finalize(&self) -> Option<String> {
        match self {
            PartValue::ShapeshifterForms(..) => Some(
                "SHAPESHIFTER_FORMS[0].on_begin(entity, &world.read_resource::<specs::LazyUpdate>(), (ctx, assets));"
                    .into(),
            ),
            PartValue::Body { .. } => Some(
                "world.write_resource::<resource::PhysicWorld>().bodies.rigid_body_mut(body).unwrap()\
                    .set_user_data(Some(Box::new(entity)));"
                    .into(),
            ),
            PartValue::Collide { hitbox, .. } => Some(format!(
                "{}{}",
                "world.write_resource::<resource::PhysicWorld>().colliders.get_mut(collider).unwrap()\
                .set_user_data(Some(Box::new(entity)));",
                if hitbox.is_some() {
                    "\nworld.write_resource::<resource::PhysicWorld>().colliders.get_mut(hitbox).unwrap()\
                .set_user_data(Some(Box::new(entity)));"
                } else {
                    ""
                }
            )),
            _ => None,
        }
    }
}

struct ComponentDefVisitor;
impl<'de> Visitor<'de> for ComponentDefVisitor {
    type Value = ComponentDef;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result { formatter.write_str("a map") }

    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
        let mut def = ComponentDef::default();
        while let Some((key, value)) = access.next_entry::<String, PartValue>()? {
            match key.as_ref() {
                "__default" => {
                    if let PartValue::Bool(value) = value {
                        def.default = value;
                    }
                },
                _ => {
                    def.parts.insert(key, value);
                },
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

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> { Ok(PartValue::Str(value.to_owned())) }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> { Ok(PartValue::Bool(value)) }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> { Ok(PartValue::Numi(value as i32)) }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> { Ok(PartValue::Numi(value as i32)) }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> { Ok(PartValue::Numf(value as f32)) }

    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
        let mut buffer = Map::<String, PartValue>::new();

        while let Some((key, value)) = access.next_entry::<String, PartValue>()? {
            match (key.as_ref(), value) {
                ("image", PartValue::Str(value)) => return Ok(PartValue::Image(value)),
                ("collision_group", PartValue::Str(value)) => return Ok(PartValue::CollisionGroup(value)),
                ("faction", PartValue::Str(value)) => return Ok(PartValue::Faction(value)),
                ("rarity", PartValue::Str(value)) => return Ok(PartValue::Rarity(value)),
                ("item", PartValue::Str(value)) => return Ok(PartValue::Item(value)),
                (key, value) => {
                    buffer.insert(key.to_owned(), value);
                },
            }
        }

        if let (Some(north), Some(east), Some(south), Some(west)) =
            (buffer.remove("north"), buffer.remove("east"), buffer.remove("south"), buffer.remove("west"))
        {
            Ok(PartValue::Directional {
                north: Box::new(north),
                south: Box::new(south),
                east: Box::new(east),
                west: Box::new(west),
            })
        } else if let Some(value) = buffer.remove("single") {
            Ok(PartValue::Single { value: Box::new(value) })
        } else if let (Some(PartValue::Numf(width)), Some(PartValue::Numf(height))) =
            (buffer.remove("width"), buffer.remove("height"))
        {
            Ok(PartValue::Size { width, height })
        } else if let (Some(PartValue::Numf(x)), Some(PartValue::Numf(y))) = (buffer.remove("x"), buffer.remove("y")) {
            Ok(PartValue::Vec { x, y })
        } else if let (Some(PartValue::Numf(mass)), Some(PartValue::Str(status))) =
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
        } else if let Some(PartValue::Str(pattern)) = buffer.remove("attack_pattern") {
            Ok(PartValue::AttackPattern(pattern, buffer))
        } else if let Some(PartValue::Str(consumable)) = buffer.remove("consumable_behaviour") {
            Ok(PartValue::ConsumableBehaviour(consumable, buffer))
        } else if let (Some(start), Some(end)) = (buffer.remove("start"), buffer.remove("end")) {
            Ok(PartValue::Range(Box::new(start), Box::new(end)))
        } else if let (Some(PartValue::Bool(sensor)), Some(shape), hitbox, Some(PartValue::Seq(collision_membership))) = (
            buffer.remove("sensor"),
            buffer.remove("shape"),
            buffer.remove("hitbox"),
            buffer.remove("collision_membership"),
        ) {
            Ok(PartValue::Collide {
                sensor,
                collision_membership,
                shape: Box::new(shape),
                hitbox: hitbox.map(|s| Box::new(s)),
            })
        } else if let Some(PartValue::Seq(forms)) = buffer.remove("shapeshifter_forms") {
            Ok(PartValue::ShapeshifterForms(forms))
        } else {
            Err(de::Error::custom(format!("No special fields defined. Here is buffer: {:?}", buffer)))
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
