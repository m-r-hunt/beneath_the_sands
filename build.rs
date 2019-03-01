#[macro_use]
extern crate serde;

use handlebars::Handlebars;
use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Property {
    name: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComponentData {
    name: String,
    #[serde(default)]
    properties: Vec<Property>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EntityData {
    name: String,
    components: Vec<ComponentData>,
}

macro_rules! module_start {
    () => {
        r"
use crate::all_components::*;
use specs::Builder;
use quicksilver::graphics::Color;

pub trait PrefabBuilder: Builder + Sized {{
"
    };
}

macro_rules! module_end {
    () => {
        r"
}}

impl<T> PrefabBuilder for T where T: Builder {{}}
"
    };
}

const TEMPLATE: &'static str = r"
fn with_{{name}}_prefab(self) -> Self {
    self
    {{#each components}}
    .with({{name}}{
        {{#each properties}}
        {{name}}: {{value}},
        {{/each}}
    })
    {{/each}}
}
";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("prefabs.rs");
    let mut outfile = std::fs::File::create(dest_path).unwrap();
    writeln!(outfile, module_start!()).unwrap();

    let mut hb = Handlebars::new();
    hb.set_strict_mode(true);
    hb.register_escape_fn(handlebars::no_escape);

    for entry in std::fs::read_dir("prefabs").unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() && entry.path().extension() == Some(OsStr::new("ron")) {
            let content = std::fs::read_to_string(entry.path()).unwrap();
            let ent_data: EntityData = ron::de::from_str(&content).unwrap();
            let result = hb.render_template(TEMPLATE, &ent_data);
            writeln!(outfile, "{}", result.unwrap()).unwrap();
        }
    }
    writeln!(outfile, module_end!()).unwrap();
}
