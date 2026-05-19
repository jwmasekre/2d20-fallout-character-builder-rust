use serde::{Deserialize, Serialize};
use crate::character::CharacterSpecialRow;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecialStat {
    Strength,
    Perception,
    Endurance,
    Charisma,
    Intelligence,
    Agility,
    Luck,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecialArray {
    #[serde(rename = "")]
    Empty,
    Custom,
    Balanced,
    Focused,
    Specialized,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
//#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct SpecialStats {
    pub strength: i16,
    pub perception: i16,
    pub endurance: i16,
    pub charisma: i16,
    pub intelligence: i16,
    pub agility: i16,
    pub luck: i16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct SpecialGifted {
    pub strength: bool,
    pub perception: bool,
    pub endurance: bool,
    pub charisma: bool,
    pub intelligence: bool,
    pub agility: bool,
    pub luck: bool,
}

/*

REVIEW

*/

impl SpecialStats {
    pub fn total(&self) -> i16 {
        (self.strength as i16) + (self.perception as i16) + 
        (self.endurance as i16) + (self.charisma as i16) + 
        (self.intelligence as i16) + (self.agility as i16) + 
        self.luck as i16
    }
    pub fn from_row(row: CharacterSpecialRow) -> Self {
        Self {
            strength: row.strength as i16,
            perception: row.perception as i16,
            endurance: row.endurance as i16,
            charisma: row.charisma as i16,
            intelligence: row.intelligence as i16,
            agility: row.agility as i16,
            luck: row.luck as i16,
        }
    }
}