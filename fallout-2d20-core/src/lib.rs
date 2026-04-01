//use serde::{Deserialize, Serialize};
use std::error::Error;
pub use special::*;
pub use skills::*;
pub use items::*;
pub use background::*;
pub use character::*;
pub use save::*;

pub mod special;
pub mod skills;
pub mod items;
pub mod background;
pub mod character;
pub mod save;

/*

REVIEW

*/

pub fn build_rules_bundle(/* TODO: db rows or json */) -> RulesBundle { todo!("Load from embedded JSON or SQLite") }
pub fn save_character_to_json(character: &FullCharacter) -> Result<String, Box<dyn Error>> { todo!("FullCharacterSave::from(character) + serde_json::to_string_pretty") }
pub fn load_character_from_json(json: &str, rules: &RulesBundle) -> Result<FullCharacter, Box<dyn Error>> { todo!("serde_json::from_str + into_full_character") }