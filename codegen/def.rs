use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::collections::BTreeMap as Map;
use std::fmt;

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

pub enum PartValue {
    Seq(Vec<PartValue>),
    Str(String),
    Num(f32),
    Bool(bool),
    Image(String),
    Faction(String),
}

pub fn get_view_from<'a>(
    def: &'a EntityDef,
    component_name: &str,
    asset_part: &str,
) -> Option<(&'a PartValue, &'a PartValue, &'a PartValue)> {
    def.components.get(component_name).map(|comp| {
        (
            comp.parts.get(asset_part).expect(&format!(
                "{} field is missing for component {} in {}",
                asset_part, component_name, def.name
            )),
            comp.parts.get("width").expect(&format!(
                "width field is missing component for {} in {}",
                component_name, def.name
            )),
            comp.parts.get("height").expect(&format!(
                "height field is missing component for {} in {}",
                component_name, def.name
            )),
        )
    })
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
        }
    }
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
        if let Some((key, value)) = access.next_entry::<String, String>()? {
            match key.as_ref() {
                "image" => Ok(PartValue::Image(value.to_owned())),
                "faction" => Ok(PartValue::Faction(value.to_owned())),
                key => Err(de::Error::custom(format!("No known type {}", key))),
            }
        } else {
            Err(de::Error::custom("Empty map"))
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
