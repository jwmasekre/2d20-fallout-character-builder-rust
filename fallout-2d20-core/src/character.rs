use serde::{Deserialize, Serialize};
use crate::{
    SpecialStats, SkillStats,
};
use crate::items::{
    Weapon, DamageEffect, WeaponEffectRow, WeaponQualityRow, WeaponMod, WeaponSlot, WeaponLegendary, Ammo, ApparelSlot, ApparelType, BodyLocation, ArmorLegendary, ApparelCoverRow, RobotModule, Consumable, Gear,
};
//use crate::save::{RulesBundle, FullCharacterSave};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sourcebook {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disease {
    pub id: i16,
    pub name: String,
    pub eff: String,
    pub duration: String,
    pub sourcebook_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharAddiction {
    pub id: i16,
    pub character_id: i16,
    pub consumable_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    pub id: i16,
    pub name: String,
    pub description: String,
    pub can_ghoul: bool,
    pub sourcebook_id: i16,
}

pub struct OriginWithTraits {
    pub origin: Origin,
    pub traits: Vec<CharTrait>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPartStats {
    pub hp: i16,
    pub inj: i16,
    pub ph_dr: i16,
    pub en_dr: i16,
    pub rd_dr: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BodyPart {
    pub active: bool,
    pub stats: Option<BodyPartStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPerk {
    pub perk: i16,
    pub perk_name: String,
    pub perk_description: Vec<String>,
    pub ranks: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharTrait {
    pub trait_id: i16,
    pub trait_name: String,
    pub trait_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CharRecipeItemType {
    Apparel,
    Chems,
    Cooking,
    PArmor,
    RArmor,
    RMods,
    Weapon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharRecipe {
    pub item: i16,
    pub item_name: String,
    pub item_type: CharRecipeItemType,
    pub complexity: i16,
    pub common: i16,
    pub uncommon: i16,
    pub rare: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharBook {
    pub book: i16,
    pub book_name: String,
    pub book_perk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeleeModifiers {
    pub base: i8,
    pub unarmed: MeleeModifierDetail,
    pub sneak: MeleeModifierDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeleeModifierDetail {
    pub active: bool,
    pub modifier: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaseDr {
    pub ph_dr: i16,
    pub en_dr: i16,
    pub rd_dr: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JunkCounts {
    pub common: i16,
    pub uncommon: i16,
    pub rare: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuperMutantKind {
    #[serde(rename = "super mutant")]
    SuperMutant,
    #[serde(rename = "nightkin")]
    Nightkin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRow {
    pub id: i16,
    pub player_id: i16,
    pub character_name: String,
    pub xp: i32,
    pub origin_id: i16,
    pub luck_points: i16,
    pub current_health: i16,
    pub rad_points: i16,
    pub head_hp: i16,
    pub head_inj: i16,
    pub la_hp: i16,
    pub la_inj: i16,
    pub ra_hp: i16,
    pub ra_inj: i16,
    pub torso_hp: i16,
    pub torso_inj: i16,
    pub ll_hp: i16,
    pub ll_inj: i16,
    pub rl_hp: i16,
    pub rl_inj: i16,
    pub caps: i16,
    pub hunger: i16,
    pub thirst: i16,
    pub sleep: i16,
    pub exposure: i16,
    pub party: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSpecialRow {
    pub id: i16,
    pub character_id: i16,
    pub strength: i16,
    pub perception: i16,
    pub endurance: i16,
    pub charisma: i16,
    pub intelligence: i16,
    pub agility: i16,
    pub luck: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSkillsRow {
    pub id: i16,
    pub character_id: i16,
    pub athletics: i16,
    pub barter: i16,
    pub big_guns: i16,
    pub energy_weapons: i16,
    pub explosives: i16,
    pub lockpick: i16,
    pub medicine: i16,
    pub melee_weapons: i16,
    pub pilot: i16,
    pub repair: i16,
    pub science: i16,
    pub small_guns: i16,
    pub sneak: i16,
    pub speech: i16,
    pub survival: i16,
    pub throwing: i16,
    pub unarmed: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterTagsRow {
    pub id: i16,
    pub character_id: i16,
    pub athletics: bool,
    pub barter: bool,
    pub big_guns: bool,
    pub energy_weapons: bool,
    pub explosives: bool,
    pub lockpick: bool,
    pub medicine: bool,
    pub melee_weapons: bool,
    pub pilot: bool,
    pub repair: bool,
    pub science: bool,
    pub small_guns: bool,
    pub sneak: bool,
    pub speech: bool,
    pub survival: bool,
    pub throwing: bool,
    pub unarmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullCharacter {
    pub core: CharacterRow,
    pub level: i16,
    pub origin_name: String,
    pub origin_desc: String,
    pub ghoul: bool,
    pub super_mutant: Option<crate::SuperMutantKind>,
    pub robot: bool,
    pub special: SpecialStats,      // from character_special
    pub skills: SkillStats,       // from character_skills
    pub tags: CharacterTagsRow,           // from character_tags
    pub perks: Vec<CharPerk>,             // join character_perks + perks
    pub traits: Vec<CharTrait>,               // join character_traits + traits
    pub body: BodyState,
    pub max_hp: i16,
    pub max_rad_points: i16,
    pub luck_pts: i16,
    pub max_luck_pts: i16,
    pub base_dr: BaseDr,
    pub poison_dr: i16,
    pub defense: i16,
    pub initiative: i16,
    pub carry_weight: i16,
    pub max_carry_weight: i16,
    pub caps: i16,
    pub hunger: i16,
    pub thirst: i16,
    pub sleep: i16,
    pub exposure: i16,
    pub addictions: Vec<CharAddiction>,
    pub diseases: Vec<Disease>,
    pub weapons: Vec<CharWeapon>,
    pub apparel: Vec<CharApparel>,
    pub power_armor_frames: Vec<CharPAFrame>,
    pub robot_modules: Vec<CharRobotModule>,
    pub ammo: Vec<CharAmmoOwned>,
    pub consumables: Vec<CharConsumable>,
    pub gear: Vec<CharGear>,
    pub junk_common: i16,
    pub junk_uncommon: i16,
    pub junk_rare: i16,
    pub misc_stuff: Vec<String>,
    pub notes: Vec<String>,
}

impl FullCharacter {
    pub fn from_rows(
        core: CharacterRow,
        special_row: CharacterSpecialRow,
        skills_row: CharacterSkillsRow,
        tags_row: CharacterTagsRow,
        perks: Vec<CharPerk>,
        traits: Vec<CharTrait>,
        addictions: Vec<CharAddiction>,
        diseases: Vec<Disease>,
        weapons: Vec<CharWeapon>,
        apparel: Vec<CharApparel>,
        pa_frames: Vec<CharPAFrame>,
        robot_modules: Vec<CharRobotModule>,
        ammo: Vec<CharAmmoOwned>,
        consumables: Vec<CharConsumable>,
        gear: Vec<CharGear>,
    ) -> Self {
        let special = SpecialStats {
            strength: special_row.strength as i16,
            perception: special_row.perception as i16,
            endurance: special_row.endurance as i16,
            charisma: special_row.charisma as i16,
            intelligence: special_row.intelligence as i16,
            agility: special_row.agility as i16,
            luck: special_row.luck as i16,
        };

        let skills = SkillStats {
            athletics: crate::SkillStatBlock {
                ranks: skills_row.athletics as i16,
                tagged: tags_row.athletics,
                total: 0,
                max: 0,
            },
            barter: crate::SkillStatBlock {
                ranks: skills_row.barter as i16,
                tagged: tags_row.barter,
                total: 0,
                max: 0,
            },
            big_guns: crate::SkillStatBlock {
                ranks: skills_row.big_guns as i16,
                tagged: tags_row.big_guns,
                total: 0,
                max: 0,
            },
            energy_weapons: crate::SkillStatBlock {
                ranks: skills_row.energy_weapons as i16,
                tagged: tags_row.energy_weapons,
                total: 0,
                max: 0,
            },
            explosives: crate::SkillStatBlock {
                ranks: skills_row.explosives as i16,
                tagged: tags_row.explosives,
                total: 0,
                max: 0,
            },
            lockpick: crate::SkillStatBlock {
                ranks: skills_row.lockpick as i16,
                tagged: tags_row.lockpick,
                total: 0,
                max: 0,
            },
            medicine: crate::SkillStatBlock {
                ranks: skills_row.medicine as i16,
                tagged: tags_row.medicine,
                total: 0,
                max: 0,
            },
            melee_weapons: crate::SkillStatBlock {
                ranks: skills_row.melee_weapons as i16,
                tagged: tags_row.melee_weapons,
                total: 0,
                max: 0,
            },
            pilot: crate::SkillStatBlock {
                ranks: skills_row.pilot as i16,
                tagged: tags_row.pilot,
                total: 0,
                max: 0,
            },
            repair: crate::SkillStatBlock {
                ranks: skills_row.repair as i16,
                tagged: tags_row.repair,
                total: 0,
                max: 0,
            },
            science: crate::SkillStatBlock {
                ranks: skills_row.science as i16,
                tagged: tags_row.science,
                total: 0,
                max: 0,
            },
            small_guns: crate::SkillStatBlock {
                ranks: skills_row.small_guns as i16,
                tagged: tags_row.small_guns,
                total: 0,
                max: 0,
            },
            sneak: crate::SkillStatBlock {
                ranks: skills_row.sneak as i16,
                tagged: tags_row.sneak,
                total: 0,
                max: 0,
            },
            speech: crate::SkillStatBlock {
                ranks: skills_row.speech as i16,
                tagged: tags_row.speech,
                total: 0,
                max: 0,
            },
            survival: crate::SkillStatBlock {
                ranks: skills_row.survival as i16,
                tagged: tags_row.survival,
                total: 0,
                max: 0,
            },
            throwing: crate::SkillStatBlock {
                ranks: skills_row.throwing as i16,
                tagged: tags_row.throwing,
                total: 0,
                max: 0,
            },
            unarmed: crate::SkillStatBlock {
                ranks: skills_row.unarmed as i16,
                tagged: tags_row.unarmed,
                total: 0,
                max: 0,
            },
            ..Default::default()
        };

        let body = BodyState {
            head: BodyPart {
                active: true,
                stats: Some(BodyPartStats {
                    hp: core.head_hp,
                    inj: core.head_inj,
                    ph_dr: 0,
                    en_dr: 0,
                    rd_dr: 0,
                }),
            },
            l_arm: BodyPart {
                active: true,
                stats: Some(BodyPartStats {
                    hp: core.la_hp,
                    inj: core.la_inj,
                    ph_dr: 0,
                    en_dr: 0,
                    rd_dr: 0,
                }),
            },
            r_arm: BodyPart {
                active: true,
                stats: Some(BodyPartStats {
                    hp: core.ra_hp,
                    inj: core.ra_inj,
                    ph_dr: 0,
                    en_dr: 0,
                    rd_dr: 0,
                }),
            },
            torso: BodyPart {
                active: true,
                stats: Some(BodyPartStats {
                    hp: core.torso_hp,
                    inj: core.torso_inj,
                    ph_dr: 0,
                    en_dr: 0,
                    rd_dr: 0,
                }),
            },
            l_leg: BodyPart {
                active: true,
                stats: Some(BodyPartStats {
                    hp: core.ll_hp,
                    inj: core.ll_inj,
                    ph_dr: 0,
                    en_dr: 0,
                    rd_dr: 0,
                }),
            },
            r_leg: BodyPart {
                active: true,
                stats: Some(BodyPartStats {
                    hp: core.rl_hp,
                    inj: core.rl_inj,
                    ph_dr: 0,
                    en_dr: 0,
                    rd_dr: 0,
                }),
            },
            // robot bits defaulted for now
            optics: BodyPart { active: false, stats: None },
            arm1: BodyPart { active: false, stats: None },
            arm2: BodyPart { active: false, stats: None },
            arm3: BodyPart { active: false, stats: None },
            thruster: BodyPart { active: false, stats: None },
            wheel: BodyPart { active: false, stats: None },
        };

        FullCharacter {
            core: core.clone(),
            level: 1, // compute later from XP
            origin_name: String::new(),
            origin_desc: String::new(),

            special,
            skills,
            tags: tags_row,

            body,
            max_hp: 0,
            max_rad_points: 0,

            caps: 0,
            hunger: 0,
            thirst: 0,
            sleep: 0,
            exposure: 0,

            base_dr: BaseDr { ph_dr: 0, en_dr: 0, rd_dr: 0 },
            poison_dr: 0,
            defense: 0,
            initiative: 0,

            luck_pts: core.luck_points,
            max_luck_pts: core.luck_points,

            carry_weight: 0,
            max_carry_weight: 0,

            ghoul: false,
            super_mutant: None,
            robot: false,

            perks,
            traits,
            addictions,
            diseases,

            weapons,
            apparel,
            power_armor_frames: pa_frames,
            robot_modules,
            ammo,
            consumables,
            gear,

            junk_common: 0,
            junk_uncommon: 0,
            junk_rare: 0,
            misc_stuff: Vec::new(),
            notes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BodyState {
    pub head: BodyPart,
    pub l_arm: BodyPart,
    pub r_arm: BodyPart,
    pub l_leg: BodyPart,
    pub r_leg: BodyPart,
    pub torso: BodyPart,
    pub optics: BodyPart,
    pub arm1: BodyPart,
    pub arm2: BodyPart,
    pub arm3: BodyPart,
    pub thruster: BodyPart,
    pub wheel: BodyPart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterWeaponRow {
    pub id: i16,
    pub weapon_id: i16,
    pub character_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterWeaponModRow {
    pub id: i16,
    pub character_weapon_id: i16,
    pub mod_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterWeaponLegendaryRow {
    pub id: i16,
    pub character_weapon_id: i16,
    pub legendary_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAmmoRow {
    pub id: i16,
    pub ammo_id: i16,
    pub quantity: i16,
    pub character_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharAmmoOwned {
    pub ammo: Ammo,
    pub quantity: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharWeaponModApplied {
    pub r#mod: WeaponMod,
    pub slot: WeaponSlot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharWeapon {
    pub id: i16,                 // character_weapons.id
    pub base: Weapon,            // rules weapon
    pub skill_name: String,      // resolved from skills table if you want
    pub effects: Vec<DamageEffect>,
    pub qualities: Vec<WeaponQualityRow>,
    pub mods: Vec<CharWeaponModApplied>,
    pub legendary: Option<WeaponLegendary>,
    pub ammo: Vec<CharAmmoOwned>,
}

pub fn build_char_weapons(
    char_weapon_rows: Vec<CharacterWeaponRow>,
    char_weapon_mod_rows: Vec<CharacterWeaponModRow>,
    char_weapon_legendary_rows: Vec<CharacterWeaponLegendaryRow>,
    weapon_rows: Vec<Weapon>,
    weapon_effect_rows: Vec<WeaponEffectRow>,
    damage_effect_rows: Vec<DamageEffect>,
    weapon_qual_rows: Vec<WeaponQualityRow>,
    quality_rows: Vec<WeaponQualityRow>,
    weapon_mod_rows: Vec<WeaponMod>,
    weapon_slot_rows: Vec<WeaponSlot>,
    ammo_rows: Vec<Ammo>,
    char_ammo_rows: Vec<CharacterAmmoRow>,
) -> Vec<CharWeapon> {
    // 1. Index rules data by id for fast lookup
    let weapons: HashMap<i16, Weapon> =
        weapon_rows.into_iter().map(|w| (w.id, w)).collect();

    let damage_effects: HashMap<i16, DamageEffect> =
        damage_effect_rows.into_iter().map(|e| (e.id, e)).collect();

    let qualities: HashMap<i16, WeaponQualityRow> =
        quality_rows.into_iter().map(|q| (q.id, q)).collect();

    let weapon_mods: HashMap<i16, WeaponMod> =
        weapon_mod_rows.into_iter().map(|m| (m.id, m)).collect();

    let weapon_slots: HashMap<i16, WeaponSlot> =
        weapon_slot_rows.into_iter().map(|s| (s.id, s)).collect();

    let ammo_map: HashMap<i16, Ammo> =
        ammo_rows.into_iter().map(|a| (a.id, a)).collect();

    // 2. Group per-weapon/per-character data

    // effects by weapon_id
    let mut effects_by_weapon: HashMap<i16, Vec<DamageEffect>> = HashMap::new();
    for row in weapon_effect_rows {
        if let Some(effect) = damage_effects.get(&row.effect_id) {
            effects_by_weapon
                .entry(row.weapon_id)
                .or_default()
                .push(effect.clone());
        }
    }

    // qualities by weapon_id
    let mut qualities_by_weapon: HashMap<i16, Vec<WeaponQualityRow>> = HashMap::new();
    for row in weapon_qual_rows {
        if let Some(qual) = qualities.get(&row.qual_id) {
            qualities_by_weapon
                .entry(row.weapon_id)
                .or_default()
                .push(qual.clone());
        }
    }

    // mods by character_weapon_id
    let mut mods_by_char_weapon: HashMap<i16, Vec<CharWeaponModApplied>> =
        HashMap::new();
    for row in char_weapon_mod_rows {
        if let Some(m) = weapon_mods.get(&row.mod_id) {
            if let Some(slot) = weapon_slots.get(&m.slot_id) {
                mods_by_char_weapon
                    .entry(row.character_weapon_id)
                    .or_default()
                    .push(CharWeaponModApplied {
                        r#mod: m.clone(),
                        slot: slot.clone(),
                    });
            }
        }
    }

    // legendary by character_weapon_id
    let mut legendary_by_char_weapon: HashMap<i16, WeaponLegendary> =
        HashMap::new();
    // assume you already indexed legendary by id
    // let legendary_map: HashMap<i16, WeaponLegendary> = ...;
    // for row in char_weapon_legendary_rows { ... }

    // ammo owned by character_id, grouped by ammo_id
    let mut ammo_owned: HashMap<i16, Vec<CharAmmoOwned>> = HashMap::new();
    // group by character_id if you want global, or by weapon later; for now, per character
    for row in char_ammo_rows {
        if let Some(a) = ammo_map.get(&row.ammo_id) {
            ammo_owned
                .entry(row.character_id)
                .or_default()
                .push(CharAmmoOwned {
                    ammo: a.clone(),
                    quantity: row.quantity,
                });
        }
    }

    // 3. Build CharWeapon list
    let mut result = Vec::new();

    for cw in char_weapon_rows {
        if let Some(base_weapon) = weapons.get(&cw.weapon_id) {
            let effects = effects_by_weapon
                .get(&cw.weapon_id)
                .cloned()
                .unwrap_or_default();

            let qualities = qualities_by_weapon
                .get(&cw.weapon_id)
                .cloned()
                .unwrap_or_default();

            let mods = mods_by_char_weapon
                .get(&cw.id)
                .cloned()
                .unwrap_or_default();

            let legendary = legendary_by_char_weapon.get(&cw.id).cloned();

            let ammo_for_char = ammo_owned
                .get(&cw.character_id)
                .cloned()
                .unwrap_or_default();

            result.push(CharWeapon {
                id: cw.id,
                base: base_weapon.clone(),
                skill_name: String::new(), // you can resolve from skills table separately
                effects,
                qualities,
                mods,
                legendary,
                ammo: ammo_for_char,
            });
        }
    }

    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterApparelRow {
    pub id: i16,
    pub apparel_id: i16,
    pub character_id: i16,
    pub equipped: bool,
    pub legendary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterApparelModRow {
    pub id: i16,
    pub character_apparel_id: i16,
    pub mod_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterArmorLegendaryRow {
    pub id: i16,
    pub character_apparel_id: i16,
    pub legendary_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharApparelModApplied {
    pub r#mod: ApparelMod,
    pub slot: ApparelSlot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharApparel {
    pub id: i16,                 // character_apparel.id
    pub base: Apparel,           // rules apparel row
    pub r#type: ApparelType,     // resolved from apparel_types
    pub covers: Vec<BodyLocation>,
    pub equipped: bool,
    pub legendary_flag: bool,    // raw boolean flag
    pub legendary: Option<ArmorLegendary>,
    pub mods: Vec<CharApparelModApplied>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerArmorRecipe {
    pub id: i16,
    pub apparel_mod_id: i16,   // apparel_mod
    pub complexity: i16,       // FK -> recipe_materials.complexity
    pub rarity: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPowerArmorFrameRow {
    pub id: i16,
    pub head: i16,
    pub la: i16,
    pub ra: i16,
    pub torso: i16,
    pub ll: i16,
    pub rl: i16,
    pub character_id: i16,
    pub equipped: bool,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPowerArmorPieceRow {
    pub id: i16,
    pub piece_id: i16,        // FK -> apparel.id
    pub mods_applied: Vec<i16>,
    pub character_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPowerArmorPieceModRow {
    pub id: i16,
    pub piece_id: i16,        // FK -> character_powerarmor_pieces.id
    pub mod_id: i16,          // FK -> apparel_mods.id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPAPieceModApplied {
    pub r#mod: crate::ApparelMod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPAPiece {
    pub id: i16,                 // character_powerarmor_pieces.id
    pub base: crate::Apparel,    // the armor piece
    pub mods: Vec<CharPAPieceModApplied>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPAFrameSlots {
    pub head: Option<CharPAPiece>,
    pub left_arm: Option<CharPAPiece>,
    pub right_arm: Option<CharPAPiece>,
    pub torso: Option<CharPAPiece>,
    pub left_leg: Option<CharPAPiece>,
    pub right_leg: Option<CharPAPiece>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPAFrame {
    pub id: i16,                 // character_powerarmor_frames.id
    pub character_id: i16,
    pub equipped: bool,
    pub location: String,
    pub slots: CharPAFrameSlots,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRobotModuleRow {
    pub id: i16,
    pub module_id: i16,     // FK -> robot_modules.id
    pub character_id: i16,
    pub equipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharRobotModule {
    pub id: i16,                 // character_robot_modules.id
    pub base: RobotModule,       // rules data
    pub equipped: bool,
    // Optionally:
    // pub perks: Vec<RobotModulePerk>,
}

pub fn build_char_apparel(
    char_apparel_rows: Vec<CharacterApparelRow>,
    char_apparel_mod_rows: Vec<CharacterApparelModRow>,
    char_armor_legendary_rows: Vec<CharacterArmorLegendaryRow>,

    apparel_rows: Vec<Apparel>,
    apparel_type_rows: Vec<ApparelType>,
    apparel_cover_rows: Vec<ApparelCoverRow>,
    body_location_rows: Vec<BodyLocation>,

    apparel_mod_rows: Vec<ApparelMod>,
    apparel_slot_rows: Vec<ApparelSlot>,
    armor_legendary_rows: Vec<ArmorLegendary>,
) -> Vec<CharApparel> {
    // Index rules
    let apparel_map: HashMap<i16, Apparel> =
        apparel_rows.into_iter().map(|a| (a.id, a)).collect();

    let type_map: HashMap<i16, ApparelType> =
        apparel_type_rows.into_iter().map(|t| (t.id, t)).collect();

    let body_loc_map: HashMap<i16, BodyLocation> =
        body_location_rows.into_iter().map(|b| (b.id, b)).collect();

    let apparel_mod_map: HashMap<i16, ApparelMod> =
        apparel_mod_rows.into_iter().map(|m| (m.id, m)).collect();

    let apparel_slot_map: HashMap<i16, ApparelSlot> =
        apparel_slot_rows.into_iter().map(|s| (s.id, s)).collect();

    let legendary_map: HashMap<i16, ArmorLegendary> =
        armor_legendary_rows.into_iter().map(|l| (l.id, l)).collect();

    // Covers grouped by apparel_id
    let mut covers_by_apparel: HashMap<i16, Vec<BodyLocation>> = HashMap::new();
    for row in apparel_cover_rows {
        if let Some(loc) = body_loc_map.get(&row.location_id) {
            covers_by_apparel
                .entry(row.apparel_id)
                .or_default()
                .push(loc.clone());
        }
    }

    // Mods grouped by character_apparel_id
    let mut mods_by_char_apparel: HashMap<i16, Vec<CharApparelModApplied>> =
        HashMap::new();
    for row in char_apparel_mod_rows {
        if let Some(m) = apparel_mod_map.get(&row.mod_id) {
            if let Some(slot) = apparel_slot_map.get(&m.slot_id) {
                mods_by_char_apparel
                    .entry(row.character_apparel_id)
                    .or_default()
                    .push(CharApparelModApplied {
                        r#mod: m.clone(),
                        slot: slot.clone(),
                    });
            }
        }
    }

    // Legendary grouped by character_apparel_id
    let mut legendary_by_char_apparel: HashMap<i16, ArmorLegendary> =
        HashMap::new();
    for row in char_armor_legendary_rows {
        if let Some(leg) = legendary_map.get(&row.legendary_id) {
            legendary_by_char_apparel.insert(row.character_apparel_id, leg.clone());
        }
    }

    // Build CharApparel list
    let mut result = Vec::new();

    for ca in char_apparel_rows {
        if let Some(base) = apparel_map.get(&ca.apparel_id) {
            let covers = covers_by_apparel
                .get(&base.id)
                .cloned()
                .unwrap_or_default();

            let r#type = type_map
                .get(&base.type_id)
                .cloned()
                .unwrap_or(ApparelType { id: 0, name: "Unknown".to_string() });

            let mods = mods_by_char_apparel
                .get(&ca.id)
                .cloned()
                .unwrap_or_default();

            let legendary = legendary_by_char_apparel.get(&ca.id).cloned();

            result.push(CharApparel {
                id: ca.id,
                base: base.clone(),
                r#type,
                covers,
                equipped: ca.equipped,
                legendary_flag: ca.legendary,
                legendary,
                mods,
            });
        }
    }

    result
}

use crate::{Apparel, ApparelMod};

pub fn build_char_power_armor(
    frame_rows: Vec<CharacterPowerArmorFrameRow>,
    piece_rows: Vec<CharacterPowerArmorPieceRow>,
    piece_mod_rows: Vec<CharacterPowerArmorPieceModRow>,

    apparel_rows: Vec<Apparel>,
    apparel_mod_rows: Vec<ApparelMod>,
) -> Vec<CharPAFrame> {
    // Index base rules
    let apparel_map: HashMap<i16, Apparel> =
        apparel_rows.into_iter().map(|a| (a.id, a)).collect();

    let mod_map: HashMap<i16, ApparelMod> =
        apparel_mod_rows.into_iter().map(|m| (m.id, m)).collect();

    // Index pieces by id
    let piece_map: HashMap<i16, CharacterPowerArmorPieceRow> =
        piece_rows.into_iter().map(|p| (p.id, p)).collect();

    // Group mods by character_piece_id
    let mut mods_by_piece: HashMap<i16, Vec<CharPAPieceModApplied>> =
        HashMap::new();
    for row in piece_mod_rows {
        if let Some(m) = mod_map.get(&row.mod_id) {
            mods_by_piece
                .entry(row.piece_id)
                .or_default()
                .push(CharPAPieceModApplied { r#mod: m.clone() });
        }
    }

    // Helper to build an Option<CharPAPiece> from a piece_id
    let make_piece = |piece_id: i16,
                      piece_map: &HashMap<i16, CharacterPowerArmorPieceRow>,
                      apparel_map: &HashMap<i16, Apparel>,
                      mods_by_piece: &HashMap<i16, Vec<CharPAPieceModApplied>>|
     -> Option<CharPAPiece> {
        let row = piece_map.get(&piece_id)?;
        let base = apparel_map.get(&row.piece_id)?.clone();
        let mods = mods_by_piece.get(&row.id).cloned().unwrap_or_default();
        Some(CharPAPiece {
            id: row.id,
            base,
            mods,
        })
    };

    // Build frames
    let mut frames = Vec::new();

    for fr in frame_rows {
        let slots = CharPAFrameSlots {
            head: make_piece(fr.head, &piece_map, &apparel_map, &mods_by_piece),
            left_arm: make_piece(fr.la, &piece_map, &apparel_map, &mods_by_piece),
            right_arm: make_piece(fr.ra, &piece_map, &apparel_map, &mods_by_piece),
            torso: make_piece(fr.torso, &piece_map, &apparel_map, &mods_by_piece),
            left_leg: make_piece(fr.ll, &piece_map, &apparel_map, &mods_by_piece),
            right_leg: make_piece(fr.rl, &piece_map, &apparel_map, &mods_by_piece),
        };

        frames.push(CharPAFrame {
            id: fr.id,
            character_id: fr.character_id,
            equipped: fr.equipped,
            location: fr.location,
            slots,
        });
    }

    frames
}

pub fn build_char_robot_modules(
    char_rows: Vec<CharacterRobotModuleRow>,
    module_rows: Vec<RobotModule>,
) -> Vec<CharRobotModule> {
    let module_map: HashMap<i16, RobotModule> =
        module_rows.into_iter().map(|m| (m.id, m)).collect();

    let mut result = Vec::new();

    for row in char_rows {
        if let Some(base) = module_map.get(&row.module_id) {
            result.push(CharRobotModule {
                id: row.id,
                base: base.clone(),
                equipped: row.equipped,
            });
        }
    }

    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConsumableRow {
    pub id: i16,
    pub consumable_id: i16,
    pub quantity: i16,
    pub character_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterGearRow {
    pub id: i16,
    pub gear_id: i16,
    pub quantity: i16,
    pub character_id: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharConsumable {
    pub id: i16,                // character_consumables.id
    pub base: Consumable,       // rules data
    pub quantity: i16,
    // Optional: resolved type name if you want it handy
    // pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharGear {
    pub id: i16,                // character_gear.id
    pub base: Gear,             // rules data
    pub quantity: i16,
}

pub fn build_char_consumables(
    char_rows: Vec<CharacterConsumableRow>,
    consumable_rows: Vec<Consumable>,
) -> Vec<CharConsumable> {
    let consumable_map: HashMap<i16, Consumable> =
        consumable_rows.into_iter().map(|c| (c.id, c)).collect();

    let mut result = Vec::new();

    for row in char_rows {
        if let Some(base) = consumable_map.get(&row.consumable_id) {
            result.push(CharConsumable {
                id: row.id,
                base: base.clone(),
                quantity: row.quantity,
            });
        }
    }

    result
}

pub fn build_char_gear(
    char_rows: Vec<CharacterGearRow>,
    gear_rows: Vec<Gear>,
) -> Vec<CharGear> {
    let gear_map: HashMap<i16, Gear> =
        gear_rows.into_iter().map(|g| (g.id, g)).collect();

    let mut result = Vec::new();

    for row in char_rows {
        if let Some(base) = gear_map.get(&row.gear_id) {
            result.push(CharGear {
                id: row.id,
                base: base.clone(),
                quantity: row.quantity,
            });
        }
    }

    result
}