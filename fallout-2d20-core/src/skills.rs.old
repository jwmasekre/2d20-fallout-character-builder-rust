use serde::{Deserialize, Serialize};
use crate::character::{CharacterSkillsRow, CharacterTagsRow};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct SkillStatBlock {
    pub ranks: i16,
    pub tagged: bool,
    pub total: i16,
    pub max: i16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Skill {
    #[serde(rename = "Athletics")]
    Athletics,
    #[serde(rename = "Barter")]
    Barter,
    #[serde(rename = "Big Guns")]
    BigGuns,
    #[serde(rename = "Energy Weapons")]
    EnergyWeapons,
    #[serde(rename = "Explosives")]
    Explosives,
    #[serde(rename = "Lockpick")]
    Lockpick,
    #[serde(rename = "Medicine")]
    Medicine,
    #[serde(rename = "Melee Weapons")]
    MeleeWeapons,
    #[serde(rename = "Pilot")]
    Pilot,
    #[serde(rename = "Repair")]
    Repair,
    #[serde(rename = "Science")]
    Science,
    #[serde(rename = "Small Guns")]
    SmallGuns,
    #[serde(rename = "Sneak")]
    Sneak,
    #[serde(rename = "Speech")]
    Speech,
    #[serde(rename = "Survival")]
    Survival,
    #[serde(rename = "Throwing")]
    Throwing,
    #[serde(rename = "Unarmed")]
    Unarmed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillStats {
    pub athletics: SkillStatBlock,
    pub barter: SkillStatBlock,
    pub big_guns: SkillStatBlock,
    pub energy_weapons: SkillStatBlock,
    pub explosives: SkillStatBlock,
    pub lockpick: SkillStatBlock,
    pub medicine: SkillStatBlock,
    pub melee_weapons: SkillStatBlock,
    pub pilot: SkillStatBlock,
    pub repair: SkillStatBlock,
    pub science: SkillStatBlock,
    pub small_guns: SkillStatBlock,
    pub sneak: SkillStatBlock,
    pub speech: SkillStatBlock,
    pub survival: SkillStatBlock,
    pub throwing: SkillStatBlock,
    pub unarmed: SkillStatBlock,
}

impl SkillStats {
    pub fn from_rows(
        skills_row: CharacterSkillsRow, tags_row: CharacterTagsRow
    ) -> Self { SkillStats {
            athletics: SkillStatBlock {
                ranks: skills_row.athletics as i16,
                tagged: tags_row.athletics,
                total: 0,
                max: 0,
            },
            barter: SkillStatBlock {
                ranks: skills_row.barter as i16,
                tagged: tags_row.barter,
                total: 0,
                max: 0,
            },
            big_guns: SkillStatBlock {
                ranks: skills_row.big_guns as i16,
                tagged: tags_row.big_guns,
                total: 0,
                max: 0,
            },
            energy_weapons: SkillStatBlock {
                ranks: skills_row.energy_weapons as i16,
                tagged: tags_row.energy_weapons,
                total: 0,
                max: 0,
            },
            explosives: SkillStatBlock {
                ranks: skills_row.explosives as i16,
                tagged: tags_row.explosives,
                total: 0,
                max: 0,
            },
            lockpick: SkillStatBlock {
                ranks: skills_row.lockpick as i16,
                tagged: tags_row.lockpick,
                total: 0,
                max: 0,
            },
            medicine: SkillStatBlock {
                ranks: skills_row.medicine as i16,
                tagged: tags_row.medicine,
                total: 0,
                max: 0,
            },
            melee_weapons: SkillStatBlock {
                ranks: skills_row.melee_weapons as i16,
                tagged: tags_row.melee_weapons,
                total: 0,
                max: 0,
            },
            pilot: SkillStatBlock {
                ranks: skills_row.pilot as i16,
                tagged: tags_row.pilot,
                total: 0,
                max: 0,
            },
            repair: SkillStatBlock {
                ranks: skills_row.repair as i16,
                tagged: tags_row.repair,
                total: 0,
                max: 0,
            },
            science: SkillStatBlock {
                ranks: skills_row.science as i16,
                tagged: tags_row.science,
                total: 0,
                max: 0,
            },
            small_guns: SkillStatBlock {
                ranks: skills_row.small_guns as i16,
                tagged: tags_row.small_guns,
                total: 0,
                max: 0,
            },
            sneak: SkillStatBlock {
                ranks: skills_row.sneak as i16,
                tagged: tags_row.sneak,
                total: 0,
                max: 0,
            },
            speech: SkillStatBlock {
                ranks: skills_row.speech as i16,
                tagged: tags_row.speech,
                total: 0,
                max: 0,
            },
            survival: SkillStatBlock {
                ranks: skills_row.survival as i16,
                tagged: tags_row.survival,
                total: 0,
                max: 0,
            },
            throwing: SkillStatBlock {
                ranks: skills_row.throwing as i16,
                tagged: tags_row.throwing,
                total: 0,
                max: 0,
            },
            unarmed: SkillStatBlock {
                ranks: skills_row.unarmed as i16,
                tagged: tags_row.unarmed,
                total: 0,
                max: 0,
            },
            ..Default::default()
        } }
}