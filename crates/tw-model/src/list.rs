use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NumberingRef {
    pub numbering_id: u32,
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLevel {
    pub level: u32,
    pub format: ListMarkerFormat,
    /// Left indent of the level's text, in points.
    pub indent: f32,
    /// How far left of the text the marker hangs, in points.
    pub hanging: f32,
    pub suffix: ListSuffix,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ListMarkerFormat {
    Bullet,
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ListSuffix {
    Tab,
    Space,
    Nothing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberingDefinition {
    pub id: u32,
    pub name: String,
    pub levels: Vec<ListLevel>,
}

impl NumberingDefinition {
    pub fn bullet() -> Self {
        Self {
            id: 1,
            name: "Bullet List".into(),
            levels: vec![
                ListLevel {
                    level: 0,
                    format: ListMarkerFormat::Bullet,
                    indent: 36.0,
                    hanging: 18.0,
                    suffix: ListSuffix::Tab,
                },
                ListLevel {
                    level: 1,
                    format: ListMarkerFormat::Bullet,
                    indent: 72.0,
                    hanging: 18.0,
                    suffix: ListSuffix::Tab,
                },
            ],
        }
    }

    pub fn numbered() -> Self {
        Self {
            id: 2,
            name: "Numbered List".into(),
            levels: vec![
                ListLevel {
                    level: 0,
                    format: ListMarkerFormat::Decimal,
                    indent: 36.0,
                    hanging: 18.0,
                    suffix: ListSuffix::Tab,
                },
                ListLevel {
                    level: 1,
                    format: ListMarkerFormat::LowerAlpha,
                    indent: 72.0,
                    hanging: 18.0,
                    suffix: ListSuffix::Tab,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NumberingCatalog {
    pub definitions: HashMap<u32, NumberingDefinition>,
}

impl NumberingCatalog {
    pub fn with_defaults() -> Self {
        let mut definitions = HashMap::new();
        let bullet = NumberingDefinition::bullet();
        let numbered = NumberingDefinition::numbered();
        definitions.insert(bullet.id, bullet);
        definitions.insert(numbered.id, numbered);
        Self { definitions }
    }

    pub fn get(&self, id: u32) -> Option<&NumberingDefinition> {
        self.definitions.get(&id)
    }
}

pub fn format_list_marker(def: &NumberingDefinition, level: u32, index: u32) -> String {
    let lvl = def
        .levels
        .iter()
        .find(|l| l.level == level)
        .or_else(|| def.levels.first());

    let Some(lvl) = lvl else {
        return String::new();
    };

    match lvl.format {
        ListMarkerFormat::Bullet => "•".into(),
        ListMarkerFormat::Decimal => format!("{}.", index + 1),
        ListMarkerFormat::LowerAlpha => {
            let c = (b'a' + (index % 26) as u8) as char;
            format!("{}.", c)
        }
        ListMarkerFormat::UpperAlpha => {
            let c = (b'A' + (index % 26) as u8) as char;
            format!("{}.", c)
        }
        ListMarkerFormat::LowerRoman => format!("{}.", to_roman(index + 1).to_lowercase()),
        ListMarkerFormat::UpperRoman => format!("{}.", to_roman(index + 1)),
    }
}

fn to_roman(mut n: u32) -> String {
    const VALUES: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for &(value, symbol) in VALUES {
        while n >= value {
            result.push_str(symbol);
            n -= value;
        }
    }
    result
}
