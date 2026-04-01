use serde::{Serialize, Deserialize};
use std::{collections::HashMap, fs, path::Path};
use crate::items::{
    Weapon, DamageEffect, WeaponQualityRow, WeaponMod, WeaponSlot, WeaponLegendary, Ammo, Apparel, ApparelType, BodyLocation, ApparelMod,
    ApparelSlot, ArmorLegendary, RobotModule, Gear,
};
use crate::character::{
    CharConsumable, CharPerk, CharTrait, Disease,
    FullCharacter, SuperMutantKind
};
use serde_json;

#[derive(Debug, Clone)]
pub struct RulesBundle {
    pub weapons: HashMap<i16, Weapon>,
    pub damage_effects: HashMap<i16, DamageEffect>,
    pub weapon_qualities: HashMap<i16, WeaponQualityRow>,
    pub weapon_mods: HashMap<i16, WeaponMod>,
    pub weapon_slots: HashMap<i16, WeaponSlot>,
    pub weapon_legendaries: HashMap<i16, WeaponLegendary>,
    pub ammo: HashMap<i16, Ammo>,

    pub apparel: HashMap<i16, Apparel>,
    pub apparel_types: HashMap<i16, ApparelType>,
    pub body_locations: HashMap<i16, BodyLocation>,
    pub apparel_mods: HashMap<i16, ApparelMod>,
    pub apparel_slots: HashMap<i16, ApparelSlot>,
    pub armor_legendaries: HashMap<i16, ArmorLegendary>,

    pub robot_modules: HashMap<i16, RobotModule>,

    pub consumables: HashMap<i16, CharConsumable>,
    pub gear: HashMap<i16, Gear>,

    pub perks: HashMap<i16, CharPerk>,
    pub traits: HashMap<i16, CharTrait>,
    pub diseases: HashMap<i16, Disease>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullCharacterSave {
    pub version: u32,
    pub player: PlayerSave,
    pub character: CharacterMetaSave,
    pub resources: ResourcesSave,
    pub health: HealthSave,
    pub defense: DefenseSave,
    pub special: crate::SpecialStats,
    pub skills: crate::SkillStats,
    pub perks: Vec<PerkSave>,
    pub traits: Vec<TraitSave>,
    pub addictions: Vec<AddictionSave>,
    pub diseases: Vec<DiseaseSave>,
    pub weapons: Vec<WeaponSave>,
    pub apparel: Vec<ApparelSave>,
    pub power_armor_frames: Vec<PAFrameSave>,
    pub robot_modules: Vec<RobotModuleSave>,
    pub ammo: Vec<AmmoSave>,
    pub consumables: Vec<ConsumableSave>,
    pub gear: Vec<GearSave>,
    pub junk: JunkSave,
    pub misc_stuff: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMetaSave {
    pub id: i16,
    pub name: String,
    pub xp: u32,
    pub level: i16,
    pub origin_id: i16,
    pub origin_name: String,
    pub origin_desc: String,
    pub ghoul: bool,
    pub super_mutant: Option<SuperMutantKind>,
    pub robot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesSave {
    pub caps: u32,
    pub luck_pts: i16,
    pub max_luck_pts: i16,
    pub hunger: i16,
    pub thirst: i16,
    pub sleep: i16,
    pub exposure: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPartSave {
    pub hp: i16,
    pub inj: i16,
    pub ph_dr: i16,
    pub en_dr: i16,
    pub rd_dr: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSave {
    pub current_hp: i16,
    pub max_hp: i16,
    pub current_rads: i16,
    pub max_rads: i16,
    pub body: BodySave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodySave {
    pub head: BodyPartSave,
    pub l_arm: BodyPartSave,
    pub r_arm: BodyPartSave,
    pub torso: BodyPartSave,
    pub l_leg: BodyPartSave,
    pub r_leg: BodyPartSave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseSave {
    pub base_ph_dr: i16,
    pub base_en_dr: i16,
    pub base_rd_dr: i16,
    pub poison_dr: i16,
    pub defense: i16,
    pub initiative: i16,
    pub carry_weight: i16,
    pub max_carry_weight: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponSave {
    pub char_weapon_id: i16,
    pub weapon_id: i16,
    pub nickname: Option<String>,
    pub tagged: bool,
    pub mod_ids: Vec<i16>,          // apparel style: or (slot_id, mod_id)
    pub legendary_id: Option<i16>,
    pub ammo: Vec<AmmoSave>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmoSave {
    pub ammo_id: i16,
    pub quantity: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApparelSave {
    pub char_apparel_id: i16,
    pub apparel_id: i16,
    pub equipped: bool,
    pub legendary_id: Option<i16>,
    pub mod_ids: Vec<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PAFrameSave {
    pub frame_id: i16,
    pub equipped: bool,
    pub location: String,
    pub head_piece_id: Option<i16>,
    pub la_piece_id: Option<i16>,
    pub ra_piece_id: Option<i16>,
    pub torso_piece_id: Option<i16>,
    pub ll_piece_id: Option<i16>,
    pub rl_piece_id: Option<i16>,
    pub piece_mod_ids: Vec<(i16, i16)>, // (character_piece_id, mod_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotModuleSave {
    pub char_module_id: i16,
    pub module_id: i16,
    pub equipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableSave {
    pub char_consumable_id: i16,
    pub consumable_id: i16,
    pub quantity: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearSave {
    pub char_gear_id: i16,
    pub gear_id: i16,
    pub quantity: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerkSave {
    pub perk_id: i16,
    pub ranks: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitSave {
    pub trait_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddictionSave {
    pub consumable_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseSave {
    pub disease_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkSave {
    pub common: i16,
    pub uncommon: i16,
    pub rare: i16,
}

impl FullCharacterSave {
    pub fn into_full(self, rules: &RulesBundle) -> Result<FullCharacter, String> {
        todo!("reconstruct FullCharacter from IDs")
        /*
        // reconstruct CharacterRow, CharacterSpecialRow, CharacterSkillsRow, CharacterTagsRow
        // from the flat save data, then call FullCharacter::from_rows
        // and your inventory loaders (build_char_weapons, build_char_apparel, etc.)
        // using rules + saved IDs.

        // example: weapons
        let char_weapon_rows = self.weapons.iter().map(|w| CharacterWeaponRow {
            id: w.char_weapon_id,
            weapon_id: w.weapon_id,
            character_id: self.character.id,
        }).collect();

        // similar for mods, legendary, ammo based on IDs in WeaponSave
        // then:
        // let weapons = build_char_weapons(..., &rules.weapons, ...);

        // do same for apparel, PA, robot modules, consumables, gear

        // finally:
        Ok(FullCharacter::from_rows(
            core_row,
            special_row,
            skills_row,
            tags_row,
            perks_vec,
            traits_vec,
            addictions_vec,
            diseases_vec,
            weapons,
            apparel,
            pa_frames,
            robot_modules,
            ammo_owned,
            consumables,
            gear,
        ))
        */
    }
}

impl From<&FullCharacter> for FullCharacterSave {
    fn from(_character: &FullCharacter) -> Self {
        todo!("Extract IDs/state from FullCharacter");
    }
}

pub fn save_character_to_file(
    character: &FullCharacter,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let save = FullCharacterSave::from(character);
    let json = serde_json::to_string_pretty(&save)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_character_from_file(
    path: impl AsRef<Path>,
    rules: &RulesBundle, // your in-memory rules (weapons, apparel, perks, etc.)
) -> Result<FullCharacter, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    todo!("read file -> FullCharacterSave -> into_full(rules)")
    /* let save: FullCharacterSave = serde_json::from_str(&data)?;
    let full = save.into_full_character(rules)?;
    Ok(full) */
}
