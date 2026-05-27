pub mod states;
pub mod db;
pub mod character;
pub mod constants;
pub mod structs;
pub mod background_slots;

use imgui::Ui;
use serde_json;
use fancy_regex::Regex;
use std::{
    cmp::Ordering,
    path::Path,
};
use anyhow::Result;
use crate::{
    character::{
        AmmoInv,
        Apparel,
        ApparelType,
        BaseDR,
        BodyLocation,
        Character,
        Consumable,
        ConsumableType,
        DamageType,
        Gear,
        Junk,
        Perk,
        RobotModule,
        Skill,
        Weapon,
        WeaponSlot
    },
    db::{
        Db,
        EffectNameSets,
        OriginRow
    },
    states::{
        BackgroundState,
        EquipmentState,
        SheetState
    },
    structs::{
        AppConfig, ModEffect, ModEffectList, PreRelease, WeaponEffect, WeaponQuality
    }
};

pub fn build_origin_labels(origins: &[OriginRow]) -> (Vec<String>, Vec<Option<usize>>) {
    let mut labels: Vec<String> = vec![];
    let mut label_map: Vec<Option<usize>> = vec![];
    let mut current_book = String::new();

    for (i, origin) in origins.iter().enumerate() {
        if origin.sourcebook != current_book {
            current_book = origin.sourcebook.clone();
            labels.push(format!("-- {} --", current_book));
            label_map.push(None); // header — not selectable
        }
        labels.push(format!("  {}", origin.name));
        label_map.push(Some(i));
    }

    (labels, label_map)
}

pub fn perk_description(desc: String) -> Vec<String> {
    //at one point, i had the regex outside of this function, and holy shit did it just nuke performance. we only retrieve perks once, so we don't need to do this every frame lol 
    //finds everything between each #: when multiple ranks 
    let desc_reg_pattern = Regex::new(r"\d:\s+(.+?)(?=\s+\d:|$)").unwrap();//fancy-regex uses more error handling so this gets complicated
    let desc_vec: Vec<String> = desc_reg_pattern.captures_iter(&desc).filter_map(|res| { match res {
        Ok(caps) => {
            caps.get(1).map(|m| m.as_str().trim().to_string())
        }
        _ => None,
    }}).collect();
    if desc_vec.len() > 0 {desc_vec} else {vec![desc]}
}

pub fn resolve_prerelease(string: &str) -> PreRelease {
    match string {
        "-alpha" => PreRelease::Alpha,
        "-beta" => PreRelease::Beta,
        "-rc" => PreRelease::ReleaseCandidate,
        _ => PreRelease::None,
    }
}

pub fn get_staggered_bonus(val: i32) -> i32 {
    match val {
        7..9 => 1,
        9..11 => 2,
        11.. => 3,
        _ => 0,
    }
}

pub fn get_melee_str(character: &Character) -> String {
    let mut melee_string_vec: Vec<String> = vec![format!("+{}CD", character.melee_mod.melee)];
    if character.melee_mod.unarmed > 0 {
        melee_string_vec.push(format!("+{}CD unarmed", character.melee_mod.melee + character.melee_mod.unarmed))
    }
    if character.melee_mod.sneak > 0 {
        melee_string_vec.push(format!("+{}CD sneak", character.melee_mod.melee + character.melee_mod.sneak))
    }
    if character.melee_mod.unarmed > 0 && character.melee_mod.sneak > 0 {
        melee_string_vec.push(format!("+{}CD unarmed sneak", character.melee_mod.melee + character.melee_mod.sneak + character.melee_mod.unarmed))
    }
    melee_string_vec.join(", ")
}

pub fn parse_damage_type(s: &str) -> DamageType {
    match s {
        "Ph"    => DamageType::Ph,
        "En"    => DamageType::En,
        "Ph/En" => DamageType::PhEn,
        "Rad"   => DamageType::Rad,
        "En/Rad"=> DamageType::EnRad,
        "Poi"   => DamageType::Poi,
        "All"   => DamageType::All,
        _       => DamageType::None,
    }
}


//need to review
pub fn parse_mod_effect(s: &str) -> ModEffect {
    let s = s.trim();
    // Dam Type = X
    if let Some(rest) = s.strip_prefix("Dam Type =").or_else(|| s.strip_prefix("Dam Type=")) {
        let val = rest.trim();
        let dtype = if val.eq_ignore_ascii_case("Both Physical and Energy") {
            parse_damage_type("Ph/En")
        } else {
            parse_damage_type(val)
        };
        return ModEffect::SetDamageType(dtype);
    }
    // Ammo = X
    if let Some(rest) = s.strip_prefix("Ammo =").or_else(|| s.strip_prefix("Ammo=")) {
        return ModEffect::SetAmmo(rest.trim().to_string());
    }
    // Allows X Mods
    if let Some(rest) = s.strip_prefix("Allows ") {
        if let Some(mod_name) = rest.strip_suffix(" Mods") {
            return ModEffect::AllowsMods(mod_name.trim().to_string());
        }
    }
    // Add X weapon
    if let Some(rest) = s.strip_prefix("Add ") {
        if let Some(weap) = rest.strip_suffix(" weapon") {
            return ModEffect::AddWeapon(weap.trim().to_string());
        }
    }
    // Gain X / Gain X Y / Gain X(Y) / Gain X (Y)
    if let Some(rest) = s.strip_prefix("Gain ") {
        let (name, val) = parse_name_and_value(rest);
        return ModEffect::Gain(name, val);
    }
    // Lose X / Lose X Y
    if let Some(rest) = s.strip_prefix("Lose ") {
        let (name, val) = parse_name_and_value(rest);
        return ModEffect::Lose(name, val);
    }
    // +X Range / -X Range
    if let Some(rest) = s.strip_suffix(" Range") {
        if let Some(_) = rest.strip_prefix('+').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::AddRange();
        }
        if let Some(_) = rest.strip_prefix('-').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::SubRange();
        }
    }
    // +X Rate / -X Rate / X Rate
    if let Some(rest) = s.strip_suffix(" Rate") {
        if let Some(n) = rest.strip_prefix('+').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::AddRate(n);
        }
        if let Some(n) = rest.strip_prefix('-').and_then(|r| r.parse::<i32>().ok()) {
            return ModEffect::SubRate(n);
        }
        if let Ok(n) = rest.parse::<i32>() {
            return ModEffect::SetRate(n);
        }
    }
    // XCD Dam / +XCD Dam / -XCD Dam
    if let Some(rest) = s.strip_suffix(" Dam") {
        if let Some(inner) = rest.strip_prefix('+') {
            // "+3CD Dam" → extract just the number
            let n: i32 = inner.trim_end_matches(|c: char| c.is_alphabetic())
                .parse().unwrap_or(0);
            return ModEffect::AddDamage(n);
        }
        if let Some(inner) = rest.strip_prefix('-') {
            let n: i32 = inner.trim_end_matches(|c: char| c.is_alphabetic())
                .parse().unwrap_or(0);
            return ModEffect::SubDamage(n);
        }
        // "3CD Dam" or "2CD Dam" — full replacement
        return ModEffect::SetDamage(rest.trim_end_matches(|c: char| c.is_alphabetic()).parse().unwrap_or(0));
    }
    ModEffect::Unknown(s.to_string())
}

/// Parse "Name" or "Name 3" or "Name(3)" or "Name (3)"
pub fn parse_name_and_value(s: &str) -> (String, Option<i32>) {
    // Try "Name(X)" or "Name (X)"
    if let Some(paren_start) = s.rfind('(') {
        let name = s[..paren_start].trim().to_string();
        let val_str = s[paren_start+1..].trim_end_matches(')').trim();
        if let Ok(v) = val_str.parse::<i32>() {
            return (name, Some(v));
        }
    }
    // Try "Name X" where last token is a number
    if let Some(last_space) = s.rfind(' ') {
        let maybe_num = &s[last_space+1..];
        if let Ok(v) = maybe_num.parse::<i32>() {
            return (s[..last_space].trim().to_string(), Some(v));
        }
    }
    (s.trim().to_string(), None)
}
pub fn apply_gain(
    effects: &mut Vec<WeaponEffect>,
    qualities: &mut Vec<WeaponQuality>,
    name: &str,
    val: Option<i32>,
    names: &EffectNameSets,
    mod_eff: &mut ModEffectList,
) {
    if names.qual_not_eff(name).is_some() {
        if names.qual_not_eff(name).unwrap() {
            if let Some(existing) = qualities.iter_mut().find(|q| q.name.eq_ignore_ascii_case(name)) {
                match (existing.value, val) {
                    (Some(qv), Some(v)) => existing.value = Some(qv + v),
                    (None, Some(v))     => existing.value = Some(v),
                    _                   => {}
                }
            } else {
                qualities.push(WeaponQuality { name: name.to_string(), value: val });
            }
            mod_eff.q_gain.push((name.to_string(), val));
        } else {
            if let Some(existing) = effects.iter_mut().find(|e| e.name.eq_ignore_ascii_case(name)) {
                match (existing.value, val) {
                    (Some(ev), Some(v)) => existing.value = Some(ev + v),
                    (None, Some(v))     => existing.value = Some(v),
                    _                   => {}
                }
            } else {
                effects.push(WeaponEffect { name: name.to_string(), value: val });
            }
            mod_eff.e_gain.push((name.to_string(), val));
        }
    } else {
        eprintln!("[apply_gain] unknown name '{}' — not in dam_effects or qualities", name);
    }
}

pub fn apply_lose(
    effects: &mut Vec<WeaponEffect>,
    qualities: &mut Vec<WeaponQuality>,
    name: &str,
    val: Option<i32>,
    names: &EffectNameSets,
    mod_eff: &mut ModEffectList,
) {
    if names.qual_not_eff(name).is_some() {
        if names.qual_not_eff(name).unwrap() {
            if let Some(pos) = qualities.iter().position(|q| q.name.eq_ignore_ascii_case(name)) {
                match (qualities[pos].value, val) {
                    (Some(qv), Some(v)) if v >= qv => { qualities.remove(pos); }
                    (Some(qv), Some(v))             => { qualities[pos].value = Some(qv - v); }
                    _                               => { qualities.remove(pos); }
                }
            }
            mod_eff.q_lose.push((name.to_string(),val))
        } else {
            if let Some(pos) = effects.iter().position(|e| e.name.eq_ignore_ascii_case(name)) {
                match (effects[pos].value, val) {
                    (Some(ev), Some(v)) if v >= ev => { effects.remove(pos); }
                    (Some(ev), Some(v))             => { effects[pos].value = Some(ev - v); }
                    _                               => { effects.remove(pos); }
                }
            }
            mod_eff.e_lose.push((name.to_string(),val))
        }
    } else {
        eprintln!("[apply_lose] unknown name '{}' — not in dam_effects or qualities", name);
    }
}


pub fn is_derived(id: i32) -> bool {
    matches!(id, 28 | 29 | 51 | 52)
}

pub fn is_2h(weapon: &Weapon) -> bool {
    weapon.qualities.contains(&"Two-Handed".to_string())
}

pub fn sync_derived_weapons(character: &mut Character, db: &Db) {
    const UNARMED: i32 = 51;
    const ROCK: i32 = 52;
    const GUN_BASH_2H: i32 = 29;
    const GUN_BASH_1H: i32 = 28;

    let two_handed_skills = [Skill::BigGuns, Skill::SmallGuns, Skill::EnergyWeapons];
    let one_handed_skills = [Skill::SmallGuns, Skill::EnergyWeapons];

    let has_2h_gun = character.weapons.iter().any(|w| {
        !is_derived(w.id) && two_handed_skills.contains(&w.skill) && is_2h(w)
    });
    let has_1h_gun = character.weapons.iter().any(|w| {
        !is_derived(w.id) && one_handed_skills.contains(&w.skill) && !is_2h(w)
    });

    let mut desired: Vec<i32> = vec![UNARMED, ROCK];
    if has_2h_gun { desired.push(GUN_BASH_2H) }
    if has_1h_gun { desired.push(GUN_BASH_1H) }

    character.weapons.retain(|w| {
        !is_derived(w.id) || desired.contains(&w.id)
    });

    for id in desired {
        if !character.weapons.iter().any(|w| w.id == id) {
            match db.get_weapon_by_id(id, character) {
                Ok(w) => character.weapons.push(w),
                Err(e) => eprintln!("Failed to load derived weapon {}: {e}", id),
            }
        }
    }
}

pub fn resolve_weapon_slot(slot: i64) -> WeaponSlot {
    match slot {
        1 => WeaponSlot::Receiver,
        2 => WeaponSlot::Barrel,
        3 => WeaponSlot::Stock,
        4 => WeaponSlot::Grip,
        5 => WeaponSlot::Magazine,
        6 => WeaponSlot::Sights,
        7 => WeaponSlot::Muzzle,
        8 => WeaponSlot::Capacitors,
        9 => WeaponSlot::Dish,
        10 => WeaponSlot::Fuel,
        11 => WeaponSlot::Tank,
        12 => WeaponSlot::Nozzle,
        13 => WeaponSlot::Blade,
        14 => WeaponSlot::Blunt,
        15 => WeaponSlot::Frame,
        _ => WeaponSlot::None,
    }
}

    
pub fn resolve_mod_effect(name_set: EffectNameSets, eff: Option<String>, damage: &mut i32, rate: &mut i32, range: &mut String, effects: &mut Vec<WeaponEffect>, qualities: &mut Vec<WeaponQuality>, dam_type: &mut DamageType) -> ModEffectList {
    let ranges = ["R","C","M","L","X"];

    let mut weapon_mod_eff = ModEffectList::new();
    if let Some(mod_fx_json) = &eff {
        if let Ok(fx_strings) = serde_json::from_str::<Vec<String>>(mod_fx_json) {
            for fx_str in &fx_strings {
                match parse_mod_effect(fx_str) {
                    ModEffect::SetDamage(d) => {
                        *damage = d;
                        weapon_mod_eff.dam_set = d;
                    }
                    ModEffect::AddDamage(n) => {
                        // Extract leading number from e.g. "3CD", add n, reformat
                        *damage += n;
                        weapon_mod_eff.dam_add = n;
                    }
                    ModEffect::SubDamage(n) => {
                        *damage -= n;
                        if *damage < 1 { *damage = 1 }
                        weapon_mod_eff.dam_sub = n;
                    }
                    ModEffect::SetRate(r) => {
                        *rate = r;
                        weapon_mod_eff.rat_set = r;
                    }
                    ModEffect::AddRate(r) => {
                        *rate += r;
                        weapon_mod_eff.rat_add = r;
                    }
                    ModEffect::SubRate(r) => {
                        *rate = (*rate - r).max(0);
                        weapon_mod_eff.rat_sub = r;
                    }
                    ModEffect::AddRange() => {
                        let range_num = ranges.iter().position(|&r| r == range.as_str()).unwrap();
                        if range_num > 0 || range_num < 4 {
                            *range = ranges[range_num + 1].to_string();
                        }
                        weapon_mod_eff.rng_add = 1;
                    }
                    ModEffect::SubRange() => {
                        let range_num = ranges.iter().position(|&r| r == range.as_str()).unwrap();
                        if range_num > 1 || range_num < 5 {
                            *range = ranges[range_num - 1].to_string();
                        }
                        weapon_mod_eff.rng_sub = 1;
                    }
                    ModEffect::Gain(name, val) => {
                        apply_gain(effects, qualities, &name, val, &name_set, &mut weapon_mod_eff);
                    }
                    ModEffect::Lose(name, val) => {
                        apply_lose(effects, qualities, &name, val, &name_set, &mut weapon_mod_eff);
                    }
                    ModEffect::SetDamageType(dt) => {
                        *dam_type = dt.clone();
                        weapon_mod_eff.dam_type = dt;
                    }
                    ModEffect::SetAmmo(name) => {
                        // Ammo swap handled at character sheet save time
                        weapon_mod_eff.ammo = Some(name);
                    }
                    ModEffect::AllowsMods(name) => {
                        // Structural changes — handled at save time
                        weapon_mod_eff.mods = name;
                    }
                    ModEffect::AddWeapon(name) => {
                        weapon_mod_eff.weap = name;
                    }
                    ModEffect::Unknown(s) => {
                        eprintln!("Unknown mod effect: {s}");
                        weapon_mod_eff.unk = s;
                    }
                }
            }
        }
    }
    weapon_mod_eff
}


pub fn resolve_apparel_type(atype: i64) -> ApparelType {
    match atype {
        1 => ApparelType::Clothing,
        2 => ApparelType::Outfit,
        3 => ApparelType::Headgear,
        4 => ApparelType::Armor,
        5 => ApparelType::PowerArmor,
        6 => ApparelType::RobotArmor,
        _ => ApparelType::Clothing
    }
}

pub fn resolve_apparel_covers(results: Vec<i64>) -> Vec<BodyLocation> {
    let mut covers: Vec<BodyLocation> = vec![];
    for item in results {
        covers.push(match item {
            1 => BodyLocation::Head,
            2 => BodyLocation::ArmLeft,
            3 => BodyLocation::ArmRight,
            4 => BodyLocation::Torso,
            5 => BodyLocation::LegLeft,
            6 => BodyLocation::LegRight,
            7 => BodyLocation::Optics,
            8 => BodyLocation::Arm1,
            9 => BodyLocation::Arm2,
            10 => BodyLocation::Arm3,
            11 => BodyLocation::Body,
            12 => BodyLocation::Thruster,
            13 => BodyLocation::Wheel,
            _ => BodyLocation::None,
        })
    }
    covers
}


pub fn resolve_consumable_type(ctype: i64) -> ConsumableType {
    match ctype {
        1 => ConsumableType::Chem,
        2 => ConsumableType::Food,
        3 => ConsumableType::Beverage,
        4 => ConsumableType::Other,
        5 => ConsumableType::Publication,
        _ => ConsumableType::Other,
    }
}


pub fn roll_cd(roll_str: &str) -> i32 {
    let mut val = 0;
    let mut roll = 0;
    if let Some(plus_pos) = roll_str.find('+') {
        let before = &roll_str[..plus_pos];
        let after = &roll_str[plus_pos + 1..];
        if let Some(num) = before
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<i32>().ok())
            .next()
        {
            val = num;
        }
        if let Some(num) = after
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<i32>().ok())
            .next()
        {
            roll = num;
        }
    } else {
        if let Some(cd_pos) = roll_str.find('c') {
            let cd_str = &roll_str[..cd_pos];
            if let Some(num) = cd_str
                .split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<i32>().ok())
                .next()
            {
                roll = num;
            }
        } else if let Some(num) = roll_str
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<i32>().ok())
            .next()
        {
            val = num
        }
    }
    let mut result = val;
    for _ in 0..roll { 
        let cd = rand::random_range(0..6);
        result += match cd {
            0..2 => 0,
            2..5 => 1,
            5 => 2,
            _ => 0, //stupid linter bitching at me thinking i missed a possible int
        }
    }
    result
}

pub fn roll_d20(num: u32) -> i32 {
    let mut result = 0;
    for _ in 0..num {
        result += rand::random_range(0..20);
    }
    result
}

pub fn _roll_location() -> String {
    let roll = rand::random_range(0..20);
    let target = match roll {
        0..2 => "Head/Optics",
        2..8 => "Torso/Body",
        8..11 => "Left Arm/Arm 1/Securitron Body/Left Foreleg/Left Wing",
        11..14 => "Right Arm/Arm 2/Securitron Left Arm/Right Foreleg/Right Wing",
        14..17 => "Left Leg/Arm 3/Securitron Right Arm/Left Hindleg/Left Track/Legs",
        17..20 => "Right Leg/Thruster/Wheel/Right Hindleg/Right Track/Legs",
        _ => "",
    };
    format!("{} - {}", roll, target)
}

pub fn roll_trinket() -> String {
    let roll = roll_d20(1) + 1;
    match roll {
        1 => "A gold pocket watch".to_string(),
        2 => "A garbled holodisk".to_string(),
        3 => "A brightly colored bandanna".to_string(),
        4 => "A silver locket".to_string(),
        5 => "Medal".to_string(),
        6 => "Potted plant".to_string(),
        7 => "Tickets to a pre-war event".to_string(),
        8 => "Wedding ring".to_string(),
        9 => "Pre-war party invitation".to_string(),
        10 => "An engraved flip lighter".to_string(),
        11 => "Loaded casino dice".to_string(),
        12 => "Id card".to_string(),
        13 => "Cosmetics case".to_string(),
        14 => "Musical Instrument".to_string(),
        15 => "Broken eyeglasses".to_string(),
        16 => "Necklace made of junk".to_string(),
        17 => "Pages of an unfinished story".to_string(),
        18 => "Overdue library book".to_string(),
        19 => "A postcard with an address".to_string(),
        20 => "A pre-war neck-tie".to_string(),
        _ => "".to_string(),
    }
}

pub fn equip_bg_apparel(
    character: &mut Character,
    equipment: &mut EquipmentState,
    background: &mut BackgroundState,
) {
    if background.selected_index.is_none() || background.apparel_selections.is_empty() { return }

    let apparel = equipment.apparel.clone();
    let mut _armor: Vec<(usize,&Apparel)> = vec![];
    let mut outfit_dr = BaseDR::new();
    let mut outfit_pos = usize::MAX;
    let mut clothing_dr = BaseDR::new();
    let mut clothing_pos = usize::MAX;
    let headgear: Vec<(usize,&Apparel)> = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Headgear).collect();
    let mut armored_limbs: Vec<BodyLocation> = vec![];

    if character.is_robot() {
        _armor = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::RobotArmor).collect();

        if !headgear.is_empty() {
            let (_, hat) = headgear[0];
            character.robot_hat = Some(hat.clone());
        }
        //just equip the first three modules, if they even have that many
        for i in 0..character.robot_modules.len().min(3) {
            character.robot_modules[i].installed = true;
        }
    } else {
        let outfits: Vec<(usize,&Apparel)> = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Outfit).collect();
        let clothing: Vec<(usize,&Apparel)> = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Clothing).collect();
        _armor = apparel.iter().enumerate().filter(|(_,a)| a.apparel_type == ApparelType::Armor).collect();
        (outfit_dr, outfit_pos) = match outfits.len() {
            0 => { (outfit_dr, outfit_pos)},
            1 => { (
                BaseDR {
                    ph_dr: outfits[0].1.ph_dr,
                    en_dr: outfits[0].1.en_dr,
                    rd_dr: outfits[0].1.rd_dr
                }, outfits[0].0)},
            _ => {
                let best = outfits
                    .iter()
                    .max_by(|a, b| {
                        match a.1.ph_dr.cmp(&b.1.ph_dr) {
                            Ordering::Equal => a.1.en_dr.cmp(&b.1.en_dr),
                            other => other,
                        }
                    });
                let (pos, best_dr) = (best.unwrap().0.clone(), BaseDR {
                    ph_dr: best.unwrap().1.ph_dr,
                    en_dr: best.unwrap().1.en_dr,
                    rd_dr: best.unwrap().1.rd_dr,
                });
                (best_dr, pos)
            },
        };
        (clothing_dr, clothing_pos) = match clothing.len() {
            0 => { (clothing_dr, clothing_pos)},
            1 => { (
                BaseDR {
                    ph_dr: clothing[0].1.ph_dr,
                    en_dr: clothing[0].1.en_dr,
                    rd_dr: clothing[0].1.rd_dr
                }, clothing[0].0)},
            _ => {
                let best = clothing
                    .iter()
                    .max_by(|a, b| {
                        match a.1.ph_dr.cmp(&b.1.ph_dr) {
                            Ordering::Equal => a.1.en_dr.cmp(&b.1.en_dr),
                            other => other,
                        }
                    });
                let (pos, best_dr) = (best.unwrap().0.clone(), BaseDR {
                    ph_dr: best.unwrap().1.ph_dr,
                    en_dr: best.unwrap().1.en_dr,
                    rd_dr: best.unwrap().1.rd_dr,
                });
                (best_dr, pos)
            },
        };
    }
    for item in _armor.clone() {
        let covers = item.1.covers.clone();
        for loc in covers {
            if !armored_limbs.contains(&loc) { armored_limbs.push(loc) }
        }
    }
    let mut top_each: Vec<(usize, &Apparel)> = vec![];
    for (i, loc) in armored_limbs.iter().enumerate() {
        let mut loc_armor: Vec<(usize, &Apparel)> = vec![];
        for item in _armor.clone() {
            if item.1.covers.contains(&loc) { loc_armor.push(item)}
        }
        for item in loc_armor {
            if top_each.len() <= i {
                top_each.push(item.clone());
            } else if top_each[i].1.ph_dr < item.1.ph_dr {
                top_each[i] = item.clone();
            } else if top_each[i].1.ph_dr == item.1.ph_dr && top_each[i].1.en_dr < item.1.en_dr {
                top_each[i] = item.clone();
            } else if top_each[i].1.ph_dr == item.1.ph_dr && top_each[i].1.en_dr == item.1.en_dr && top_each[i].1.rd_dr == item.1.rd_dr {
                top_each[i] = item.clone();
            }
        }
    }
    let mut outfit = false;
    if outfit_pos != usize::MAX {
        outfit = true;
        for loc in apparel[outfit_pos].covers.clone() {
            let limb_pos = armored_limbs.iter().position(|l| *l == loc);
            //can't guarantee there's a limb covered by armor here, handle the unwrap correctly
            if limb_pos.is_none() { break; }
            if outfit_dr.ph_dr < top_each[limb_pos.unwrap()].1.ph_dr + clothing_dr.ph_dr {
                outfit = false;
                break;
            } else if outfit_dr.ph_dr == top_each[limb_pos.unwrap()].1.ph_dr + clothing_dr.ph_dr && outfit_dr.en_dr < top_each[limb_pos.unwrap()].1.en_dr + clothing_dr.en_dr {
                outfit = false;
                break;
            } else if outfit_dr.ph_dr == top_each[limb_pos.unwrap()].1.ph_dr + clothing_dr.ph_dr && outfit_dr.en_dr == top_each[limb_pos.unwrap()].1.en_dr + clothing_dr.en_dr && outfit_dr.rd_dr < top_each[limb_pos.unwrap()].1.rd_dr + clothing_dr.rd_dr {
                outfit = false;
                break;
            }
        }
    }
    if outfit {
        equipment.apparel[outfit_pos].equipped = true;
        for loc in equipment.apparel[outfit_pos].covers.clone() {
            match loc {
                BodyLocation::Head => {
                    character.limb_dr.head.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::ArmLeft => {
                    character.limb_dr.arm_left.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::ArmRight => {
                    character.limb_dr.arm_right.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::Torso => {
                    character.limb_dr.torso.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::LegLeft => {
                    character.limb_dr.leg_left.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                BodyLocation::LegRight => {
                    character.limb_dr.leg_right.equipped = vec![equipment.apparel[outfit_pos].clone()];
                },
                _ => {},
            }
        }
    } else {
        if clothing_pos != usize::MAX {
            equipment.apparel[clothing_pos].equipped = true;
            for loc in equipment.apparel[clothing_pos].covers.clone() {
                match loc {
                    BodyLocation::Head => {
                        character.limb_dr.head.equipped = vec![equipment.apparel[clothing_pos].clone()];
                        },
                    BodyLocation::ArmLeft => {
                        character.limb_dr.arm_left.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::ArmRight => {
                        character.limb_dr.arm_right.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::Torso => {
                        character.limb_dr.torso.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::LegLeft => {
                        character.limb_dr.leg_left.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    BodyLocation::LegRight => {
                        character.limb_dr.leg_right.equipped = vec![equipment.apparel[clothing_pos].clone()];
                    },
                    _ => {},
                }
            }
        }
        if !character.is_robot() && !headgear.is_empty() {
            //just put on the first hat idgaf rn
            character.limb_dr.head.equipped.push(equipment.apparel[headgear[0].0].clone());
        }
        for (i,loc) in armored_limbs.iter().enumerate() {
            equipment.apparel[top_each[i].0].equipped = true;
            let item = equipment.apparel[top_each[i].0].clone();
            match loc {
                BodyLocation::None => {},
                BodyLocation::Head => character.limb_dr.head.equipped.push(item),
                BodyLocation::ArmLeft => character.limb_dr.arm_left.equipped.push(item),
                BodyLocation::ArmRight => character.limb_dr.arm_right.equipped.push(item),
                BodyLocation::Torso => character.limb_dr.torso.equipped.push(item),
                BodyLocation::LegLeft => character.limb_dr.leg_left.equipped.push(item),
                BodyLocation::LegRight => character.limb_dr.leg_right.equipped.push(item),
                BodyLocation::Optics => character.limb_dr.optics.equipped.push(item),
                BodyLocation::Arm1 => character.limb_dr.arm_1.equipped.push(item),
                BodyLocation::Arm2 => character.limb_dr.arm_2.equipped.push(item),
                BodyLocation::Arm3 => character.limb_dr.arm_3.equipped.push(item),
                BodyLocation::Body => character.limb_dr.body.equipped.push(item),
                BodyLocation::Thruster => character.limb_dr.thruster.equipped.push(item),
                BodyLocation::Wheel => character.limb_dr.wheel.equipped.push(item),
            };
        }
    }
    //background.equipment_changed = false;
}

pub fn equip_apparel(character: &mut Character) {
    let equipped_apparel: Vec<&Apparel> = character.apparel.iter().filter(|a| a.equipped).collect();
    for item in equipped_apparel {
        let covered = item.covers.clone();
        for loc in covered {
            match loc {
                BodyLocation::None => {},
                BodyLocation::Head => character.limb_dr.head.equipped.push(item.clone()),
                BodyLocation::ArmLeft => character.limb_dr.arm_left.equipped.push(item.clone()),
                BodyLocation::ArmRight => character.limb_dr.arm_right.equipped.push(item.clone()),
                BodyLocation::Torso => character.limb_dr.torso.equipped.push(item.clone()),
                BodyLocation::LegLeft => character.limb_dr.leg_left.equipped.push(item.clone()),
                BodyLocation::LegRight => character.limb_dr.leg_right.equipped.push(item.clone()),
                BodyLocation::Optics => character.limb_dr.optics.equipped.push(item.clone()),
                BodyLocation::Arm1 => character.limb_dr.arm_1.equipped.push(item.clone()),
                BodyLocation::Arm2 => character.limb_dr.arm_2.equipped.push(item.clone()),
                BodyLocation::Arm3 => character.limb_dr.arm_3.equipped.push(item.clone()),
                BodyLocation::Body => character.limb_dr.body.equipped.push(item.clone()),
                BodyLocation::Thruster => character.limb_dr.thruster.equipped.push(item.clone()),
                BodyLocation::Wheel => character.limb_dr.wheel.equipped.push(item.clone()),
            }
        }
    }
    character.limb_dr.update_dr(character.base_dr.clone(), character.perk_ranks(144), character.junk.common + character.junk.uncommon + character.junk.rare, character.perk_ranks(172));
}


pub fn render_weapons(ui: &Ui, weapons: Vec<Weapon>, character: &Character, cfg: &AppConfig) {
    if weapons.is_empty() {
        ui.text_disabled("  No weapons.");
    } else {
        let table_w = ui.content_region_avail()[0];
        let table_min = 800.0 * cfg.ui_scale;
        let table_max = 1080.0 * cfg.ui_scale;
        let col_widths_min = [150.0 * cfg.ui_scale, 50.0 * cfg.ui_scale, 30.0 * cfg.ui_scale, 35.0 * cfg.ui_scale, 35.0 * cfg.ui_scale, 90.0 * cfg.ui_scale, 45.0 * cfg.ui_scale, 40.0 * cfg.ui_scale, 35.0 * cfg.ui_scale, 120.0 * cfg.ui_scale, 120.0 * cfg.ui_scale, 35.0 * cfg.ui_scale];
        let col_widths_max = [220.0 * cfg.ui_scale, 55.0 * cfg.ui_scale, 40.0 * cfg.ui_scale, 45.0 * cfg.ui_scale, 45.0 * cfg.ui_scale, 132.0 * cfg.ui_scale, 55.0 * cfg.ui_scale, 50.0 * cfg.ui_scale, 45.0 * cfg.ui_scale, 176.0 * cfg.ui_scale, 176.0 * cfg.ui_scale, 45.0 * cfg.ui_scale];
        let col_widths: [f32; 12];
        if table_w < table_min { col_widths = col_widths_min; }
        else if table_w > table_max { col_widths = col_widths_max; }
        else {
            let ratio = (table_w - table_min) / (table_max - table_min);
            col_widths = std::array::from_fn(|i| col_widths_min[i] + ((col_widths_max[i] - col_widths_min[i]) * ratio));
        }
        let headers    = ["Weapon", "Skill", "TN", "Tag", "Dmg", "Effects", "Type", "Rate", "Rng", "Qualities", "Ammo", "Wgt"];

        ui.columns(headers.len() as i32, "##weap_hdr", false);
        for (i, (hdr, cw)) in headers.iter().zip(col_widths.iter()).enumerate() {
            ui.set_column_width(i as i32, *cw);
            ui.text_disabled( hdr);
            ui.next_column();
        }
        ui.separator();

        for weapon in &weapons {
            let weapon_name = format!("{} {}",weapon.prefix, weapon.name);
            let mut effs = weapon.effects.clone();
            let mut quals = weapon.qualities.clone();
            //basher
            if character.has_perk(9) && [28,29].contains(&weapon.id) && !effs.contains(&"Vicious".to_string()) {
                effs.push("Vicious".to_string());
            }
            //big leagues
            if character.has_perk(11) && is_2h(weapon) && !is_derived(weapon.id) && weapon.skill == Skill::MeleeWeapons && !effs.contains(&"Vicious".to_string()) {
                effs.push("Vicious".to_string());
            };
            //demo expert
            if character.has_perk(26) && weapon.qualities.contains(&"Blast".to_string()) && !weapon.effects.contains(&"Vicious".to_string()) {
                effs.push("Vicious".to_string());
            };
            //licensed plumber
            if character.has_perk(160) && weapon.name.contains("Pipe") {
                let u_pos = quals.iter().position(|q| q == &"Unreliable".to_string());
                if u_pos.is_some() {
                    quals.remove(u_pos.unwrap());
                }
            }
            //piercing strike
            if character.has_perk(69) && [Skill::Unarmed, Skill::MeleeWeapons].contains(&weapon.skill) {
                let p_pos = effs.iter().position(|e| e.starts_with("Piercing"));
                if p_pos.is_some() {
                    let p_val_str = if effs[p_pos.unwrap()].contains("X") {
                        effs[p_pos.unwrap()].strip_prefix("Piercing X ")
                    } else {
                        effs[p_pos.unwrap()].strip_prefix("Piercing ")
                    };
                    let mut p_val = 0;
                    match p_val_str.unwrap().parse::<i32>() {
                        Ok(_) => p_val = p_val_str.unwrap().parse().ok().unwrap(),
                        Err(e) => eprintln!("failed to parse piercing value: {e}"),
                    };
                    effs[p_pos.unwrap()] = format!("Piercing {}", p_val + 1);
                } else {
                    effs.push("Piercing 1".to_string());
                }
            }
            //shotgun surgeon
            if character.has_perk(82) && weapon.name.ends_with("Shotgun") {
                let p_pos = effs.iter().position(|e| e.starts_with("Piercing"));
                if p_pos.is_some() {
                    let p_val: i32 = effs[p_pos.unwrap()].strip_prefix("Piercing ").unwrap().parse().ok().unwrap();
                    effs[p_pos.unwrap()] = format!("Piercing {}", p_val + 1);
                } else {
                    effs.push("Piercing 1".to_string());
                }
            }
            //incisor
            if character.has_perk(127) && weapon.skill == Skill::MeleeWeapons {
                let p_pos = effs.iter().position(|e| e.starts_with("Piercing"));
                if p_pos.is_some() {
                    let p_val: i32 = effs[p_pos.unwrap()].strip_prefix("Piercing ").unwrap().parse().ok().unwrap();
                    effs[p_pos.unwrap()] = format!("Piercing {}", p_val + character.perk_ranks(127));
                } else {
                    effs.push(format!("Piercing {}", character.perk_ranks(127)));
                }
            }
            //bow before me
            if character.has_perk(133) && weapon.name.ends_with("ow") {
                let p_pos = effs.iter().position(|e| e.starts_with("Piercing"));
                if p_pos.is_some() {
                    let p_val: i32 = effs[p_pos.unwrap()].strip_prefix("Piercing ").unwrap().parse().ok().unwrap();
                    effs[p_pos.unwrap()] = format!("Piercing {}", p_val + 1);
                } else {
                    effs.push("Piercing 1".to_string());
                }
            }
            let tag_str = if weapon.tag { "*" } else { "" };
            let skill_str = match weapon.skill {
                Skill::Athletics => "At",
                Skill::BigGuns => "BG",
                Skill::EnergyWeapons => "EW",
                Skill::Explosives => "Ex",
                Skill::MeleeWeapons => "MW",
                Skill::SmallGuns => "SG",
                Skill::Throwing => "Th",
                Skill::Unarmed => "Un",
                _ => "",
            };
            let dam_type = match weapon.dam_type {
                DamageType::All => "All",
                DamageType::En => "En",
                DamageType::EnRad => "En/Rd",
                DamageType::Ph => "Ph",
                DamageType::PhEn => "Ph/En",
                DamageType::Poi => "Poi",
                DamageType::Rad => "Rd",
                DamageType::None => "N/A",
            };
            let mut damage = weapon.damage;
            if skill_str == "MW" {
                damage += character.melee_mod.melee;
            } else if skill_str == "Un" {
                damage += character.melee_mod.melee + character.melee_mod.unarmed;
            }
            //commando
            if character.has_perk(22) && weapon.rate >= 3 && [Skill::SmallGuns, Skill::EnergyWeapons].contains(&weapon.skill) {
                damage += character.perk_ranks(22);
            }
            //gunslinger
            if character.has_perk(38) && weapon.rate <= 2 && [Skill::SmallGuns, Skill::EnergyWeapons, Skill::BigGuns].contains(&weapon.skill) && !is_2h(weapon) {
                damage += character.perk_ranks(38);
            }
            //laser cdr
            if character.has_perk(49) && weapon.skill == Skill::EnergyWeapons {
                damage += character.perk_ranks(49);
            }
            //rifleman
            if character.has_perk(76) && is_2h(weapon) && [Skill::SmallGuns, Skill::EnergyWeapons, Skill::BigGuns].contains(&weapon.skill) {
                damage += character.perk_ranks(76);
                if character.perk_ranks(76) > 1 {
                    let p_pos = effs.iter().position(|e| e.starts_with("Piercing"));
                    if p_pos.is_some() {
                        let p_val: i32 = effs[p_pos.unwrap()].strip_prefix("Piercing ").unwrap().parse().ok().unwrap();
                        effs[p_pos.unwrap()] = format!("Piercing {}", p_val + 1);
                    } else {
                        effs.push("Piercing 1".to_string());
                    }
                }
            }
            //size matters
            if character.has_perk(84) && weapon.skill == Skill::BigGuns {
                damage += character.perk_ranks(84);
            }
            //gladiator
            if character.has_perk(126) && weapon.skill == Skill::MeleeWeapons && !is_2h(weapon) {
                damage += character.perk_ranks(126);
            }
            //archer
            if character.has_perk(132) && weapon.name.ends_with("ow") {
                damage += character.perk_ranks(132);
            }
            //lock and load
            //let mut rate = if character.has_perk(109) && weapon.skill == Skill::BigGuns && weapon.rate > 0 {
            let rate = if character.has_perk(109) && weapon.skill == Skill::BigGuns && weapon.rate > 0 {
                weapon.rate + character.perk_ranks(109)
            } else { weapon.rate };
            let eff_str = effs.join(", ");
            let qual_str = quals.join(",");
            let cells: &[&str] = &[
                &weapon_name,
                skill_str,
                &weapon.target.to_string(),
                tag_str,
                &damage.to_string(),
                &eff_str,
                dam_type,
                &rate.to_string(),
                &weapon.range,
                &qual_str,
                &weapon.ammo,
                &weapon.wgt.to_string(),
            ];
            for cell in cells {
                ui.text_wrapped(cell);
                ui.next_column();
            }
        }
        ui.columns(1, "##weapon_end", false);
    }
}

pub fn measure_wrapped_item_height(ui: &Ui, name: &str, name_w: f32) -> f32 {
    let line_h = ui.text_line_height_with_spacing();
    let text_size = ui.calc_text_size_with_opts(name, false, name_w);
    // round up to nearest line and ensure at least one line
    let lines = (text_size[1] / ui.text_line_height()).ceil().max(1.0);
    lines * line_h
}

pub fn calculate_inventory_height(
    ui: &Ui,
    ammo: &[AmmoInv],
    apparel: &[Apparel],
    consumables: &[Consumable],
    modules: &[RobotModule],
    gear: &[Gear],
    junk: &Junk,
    misc: &[String],
    cfg: &AppConfig,
) -> f32 {
    let line_h = ui.text_line_height_with_spacing();
    let section_overhead = line_h * 2.0 + 8.0;
    let name_w = 150.0 * cfg.ui_scale;
    let mut h = 0.0;

    let ammo_actual: Vec<&AmmoInv> = ammo.iter().filter(|a| a.quantity > 0).collect();
    if !ammo_actual.is_empty() {
        let rows_h: f32 = ammo_actual.iter()
            .map(|a| measure_wrapped_item_height(ui, &a.ammo.name, name_w))
            .sum();
        h += section_overhead + rows_h;
    }
    if !apparel.is_empty() {
        let rows_h: f32 = apparel.iter()
            .map(|a| measure_wrapped_item_height(ui, &a.name, name_w))
            .sum();
        h += section_overhead + rows_h;
    }
    if !consumables.is_empty() {
        let rows_h: f32 = consumables.iter()
            .map(|c| measure_wrapped_item_height(ui, &c.name, name_w))
            .sum();
        h += section_overhead + rows_h;
    }
    if !modules.is_empty() {
        let rows_h: f32 = modules.iter()
            .map(|m| measure_wrapped_item_height(ui, &m.name, name_w))
            .sum();
        h += section_overhead + rows_h;
    }
    if !gear.is_empty() {
        let rows_h: f32 = gear.iter()
            .map(|g| measure_wrapped_item_height(ui, &g.name, name_w))
            .sum();
        h += section_overhead + rows_h;
    }
    if junk.common > 0 {
        h += line_h + 8.0;
    }
    let misc_actual: Vec<&String> = misc.iter().filter(|s| !s.is_empty()).collect();
    if !misc_actual.is_empty() {
        let rows_h: f32 = misc_actual.iter()
            .map(|s| measure_wrapped_item_height(ui, s, name_w))
            .sum();
        h += line_h + rows_h + 8.0;
    }

    h += 8.0 + line_h * 2.0 + line_h * 2.0;
    h.max(120.0 * cfg.ui_scale)
}

pub fn render_inventory(ui: &Ui, ammo: Vec<AmmoInv>, apparel: Vec<Apparel>, consumables: Vec<Consumable>, modules: Vec<RobotModule>, gear: Vec<Gear>, junk: Junk, misc: Vec<String>, character: &mut Character, db: &Db, cfg: &AppConfig) {
    let name_w = 150.0 * cfg.ui_scale;
    let wgt_w = 55.0 * cfg.ui_scale;
    let quan_w = 75.0 * cfg.ui_scale;
    let eq_w = 75.0 * cfg.ui_scale;

    let mut eq_wgt: i32 = character.weapons.iter().map(|w| w.wgt).sum();

    let ammo_actual: Vec<&AmmoInv> = ammo.iter().filter(|a| a.quantity > 0).collect();

    if !ammo_actual.is_empty() {
        ui.columns(3,"##eq_ammo",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Ammo");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, quan_w);
        ui.text_disabled("Quantity");
        ui.next_column();
        ui.separator();
        for item in ammo_actual.clone() {
            ui.text_wrapped(format!("{}", item.ammo.name));
            ui.next_column();
            ui.text(format!("{}", item.ammo.wgt));
            ui.next_column();
            ui.text(format!("{}", item.quantity));
            ui.next_column();
            eq_wgt += item.ammo.wgt * item.quantity;
        }
        ui.spacing();
    }
    if !apparel.is_empty() {
        ui.columns(3,"##eq_apparel",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Apparel");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, eq_w);
        ui.text_disabled("Equipped");
        ui.next_column();
        ui.separator();
        for item in apparel.clone() {
            let label = if item.equipped {
                format!("[*]##ap_{}", item.id)
            } else {
                format!("[ ]##ap_{}", item.id)
            };
            let block = can_equip(character, item.id);
            let blocked = matches!(block, EquipBlock::WouldBlock(_));
            ui.text_wrapped(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            let _d = if blocked && !item.equipped {
                Some(ui.begin_disabled(true))
            } else { None };
            if ui.button(&label) {
                toggle_apparel(character, item.id, item.db_id, db);
                //does not save yet
            }
            drop(_d);
            if blocked && ui.is_item_hovered() {
                if let EquipBlock::WouldBlock(reason) = block {
                    ui.tooltip_text(&reason);
                }
            }
            ui.next_column();
            eq_wgt += item.wgt * if item.apparel_type != ApparelType::PowerArmor { (4 - character.perk_ranks(131)) / 4 } else { 1 };
        }
        ui.spacing();
    }
    if !consumables.is_empty() {
        ui.columns(3,"##eq_consumable",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Consumable");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, quan_w);
        ui.text_disabled("Quantity");
        ui.next_column();
        ui.separator();
        for item in consumables.clone() {
            ui.text_wrapped(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            ui.text(format!("{}", item.quantity));
            ui.next_column();
            eq_wgt += item.wgt * item.quantity;
        }
        ui.spacing();
    }
    if !modules.is_empty() {
        ui.columns(3,"##eq_robomods",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Module");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, eq_w);
        ui.text_disabled("Equipped");
        ui.next_column();
        ui.separator();
        for item in modules.clone() {
            let label = if item.installed {
                format!("[*]##rm_{}", item.id)
            } else {
                format!("[ ]##rm_{}", item.id)
            };
            let installed = character.robot_modules.iter().filter(|m| m.installed).count();
            let blocked = installed >= 3;
            ui.text_wrapped(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            let _d = if blocked && !item.installed {
                Some(ui.begin_disabled(true))
            } else { None };
            if ui.button(&label) {
                toggle_module(character, item.id, item.db_id, db);
                //does not save yet
            }
            drop(_d);
            if blocked && ui.is_item_hovered() {
                ui.tooltip_text("cannot install more than 3 modules");
            }
            ui.next_column();
            eq_wgt += item.wgt;
        }
        ui.spacing();
    }
    if !gear.is_empty() {
        ui.columns(3,"##eq_gear",false);
        ui.set_column_width(0, name_w);
        ui.text_disabled("Gear");
        ui.next_column();
        ui.set_column_width(1, wgt_w);
        ui.text_disabled("Weight");
        ui.next_column();
        ui.set_column_width(2, quan_w);
        ui.text_disabled("Quantity");
        ui.next_column();
        ui.separator();
        for item in gear.clone() {
            ui.text_wrapped(format!("{}", item.name));
            ui.next_column();
            ui.text(format!("{}", item.wgt));
            ui.next_column();
            ui.text(format!("{}", item.quantity));
            ui.next_column();
            eq_wgt += item.wgt * item.quantity;
        }
        ui.spacing();
    }
    if junk.common > 0 {
        ui.columns(1,"#eq_junk", false);
        ui.text_disabled(format!("{:14}", "Junk (Common)"));
        ui.same_line();
        ui.text(format!("{}",junk.common));
        ui.spacing();
            eq_wgt += junk.common * if character.has_perk(129) { 1 } else { 2 };
    }
    let misc_actual: Vec<&String> = misc.iter().filter(|s| *s != "").collect();
    if !misc_actual.is_empty() {
        ui.columns(1,"#eq_misc", false);
        ui.text_disabled("Misc");
        for item in misc_actual.clone() {
            ui.text(format!("  {}", item));
        }
        ui.spacing();
    }
    ui.separator();

    ui.columns(2, "##eq_weight", false);
    ui.set_column_width(0,name_w);
    ui.text_disabled("Current Weight");
    ui.next_column();
    ui.set_column_width(1, wgt_w);
    ui.text(format!("{}", eq_wgt));
    ui.next_column();
    ui.text_disabled("Max Weight");
    ui.next_column();
    ui.text(format!("{}", character.carry_wgt_max));
    ui.next_column();
}


pub fn export_character(character: &Character, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(character)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn sanitize_filename(name: &str) -> String {
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let sanitized: String = name.chars().map(|c| match c {
        '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
        c if c.is_control() => '_',
        ' ' => '_',
        c => c,
    }).collect();
    let sanitized = sanitized.trim_matches('.').to_string();
    let upper = sanitized.to_uppercase();
    let base = upper.split('.').next().unwrap_or("");
    if sanitized.is_empty() || reserved_names.contains(&base) {
        "character".to_string()
    } else {
        sanitized
    }
}


pub fn toggle_module(character: &mut Character, module_id: i32, _db_id: i64, db: &Db) {
    let Some(index) = character.robot_modules.iter().position(|m| m.id == module_id) else { return };
    let installed = character.robot_modules[index].installed;
    character.robot_modules[index].installed = !installed;
    equip_apparel(character);
    match db.save_character(character) {
        Ok(_) => {},
        Err(e) => eprintln!("Failed to save character: {e}"),
    }
    //db.update_module(db_id, installed);
}

pub fn toggle_apparel(character: &mut Character, apparel_id: i32, _db_id: i64, db: &Db) {
    let Some(index) = character.apparel.iter().position(|a| a.id == apparel_id) else { return };
    let item = character.apparel[index].clone();
    if item.equipped {
        character.apparel[index].equipped = false;
    } else {
        match item.apparel_type {
            ApparelType::RobotArmor => {
                if !character.is_robot() { return; }
            }
            _ => {
                if character.is_robot() { return; }
            }
        }
        for loc in &item.covers {
            if *loc == BodyLocation::None { continue; }
            let currently_equipped: Vec<&Apparel> = character.apparel.iter()
                .filter(|a| a.equipped && a.id != apparel_id && a.covers.contains(loc))
                .collect();
            //check if the action would be blocked
            for blocking in &currently_equipped {
                match (&item.apparel_type, &blocking.apparel_type) {
                    (_, ApparelType::Outfit) => return,
                    (ApparelType::Outfit, _) => {},
                    (ApparelType::Clothing, ApparelType::Clothing) => return,
                    (ApparelType::Armor, ApparelType::Armor) => return,
                    (ApparelType::RobotArmor, ApparelType::RobotArmor) => return,
                    _ => {}
                }
            }
        }
        if item.apparel_type == ApparelType::Outfit {
            let covered = item.covers.clone();
            for a in character.apparel.iter_mut() {
                if a.id == apparel_id { continue }
                if a.covers.iter().any(|loc| covered.contains(loc)) {
                    a.equipped = false;
                }
            }
        }
        if item.apparel_type == ApparelType::Clothing {
            for a in character.apparel.iter_mut() {
                if a.id == apparel_id { continue; }
                if ApparelType::Clothing == a.apparel_type || ApparelType::Outfit == a.apparel_type {
                    a.equipped = false;
                }
            }
        }
        character.apparel[index].equipped = true;
    }
    for limb in character.limb_dr.mut_active_limbs().iter_mut() {
        limb.0.equipped.clear();
    }
    equip_apparel(character);
    match db.save_character(character) {
        Ok(_) => {},
        Err(e) => eprintln!("Failed to save character: {e}"),
    }
    //db.update_apparel(db_id, item.equipped);
}


#[derive(Debug)]
pub enum EquipBlock {
    Free,
    WouldBlock(String)
}
pub fn can_equip(character: &Character, apparel_id: i32) -> EquipBlock {
    let Some(item) = character.apparel.iter().find(|a| a.id == apparel_id) else {
        return EquipBlock::WouldBlock("item not found".into());
    };
    if item.equipped { return EquipBlock::Free }
    match item.apparel_type {
        ApparelType::RobotArmor if !character.is_robot() => return EquipBlock::WouldBlock("only robots can wear robot armor".into()),
        ApparelType::Clothing | ApparelType::Headgear | ApparelType::Outfit | ApparelType::Armor if character.is_robot() => return EquipBlock::WouldBlock("robots cannot wear clothing, outfits, or standard armor".into()),
        _ => {}
    }
    for loc in &item.covers {
        if *loc == BodyLocation::None { continue; }
        for blocking in character.apparel.iter().filter(|a| a.equipped && a.id != apparel_id && a.covers.contains(loc)) {
            match (&item.apparel_type, &blocking.apparel_type) {
                (_, ApparelType::Outfit) =>
                    return EquipBlock::WouldBlock(format!("{} covers that location", blocking.name)),
                (ApparelType::Clothing, ApparelType::Clothing) =>
                    return EquipBlock::WouldBlock(format!("{} already equipped", blocking.name)),
                (ApparelType::Armor, ApparelType::Armor) =>
                    return EquipBlock::WouldBlock(format!("{} already on that limb", blocking.name)),
                (ApparelType::RobotArmor, ApparelType::RobotArmor) =>
                    return EquipBlock::WouldBlock(format!("{} already on that limb", blocking.name)),
                _ => {}
            }
        }
    }
    EquipBlock::Free
}

pub fn apply_level_change(character: &mut Character, state: &mut SheetState, db: &Db) {
    let idx = state.skill_choice as usize;

    if state.up {
        // ── Level up ──────────────────────────────────────────────
        character.level += 1;

        // increase chosen skill rank
        let skills = character.skills.mut_skill_block();
        skills[idx].ranks += 1;
        skills[idx].update();

        // add or rank up chosen perk
        let perk_id = state.perk_choice;
        if let Some(existing) = character.perks.iter_mut().find(|p| p.id == perk_id) {
            existing.ranks += 1;
        } else if let Some(prow) = state.perks.iter().find(|p| p.id == perk_id) {
            character.perks.push(Perk {
                id: prow.id,
                name: prow.name.clone(),
                desc: prow.description.clone(),
                ranks: 1,
            });
        }
    } else {
        // ── Delevel ───────────────────────────────────────────────
        character.level -= 1;

        // reduce chosen skill rank
        let skills = character.skills.mut_skill_block();
        if skills[idx].ranks > 0 {
            skills[idx].ranks -= 1;
            skills[idx].update();
        }

        // remove or reduce chosen perk rank
        let perk_id = state.perk_choice;
        if let Some(pos) = character.perks.iter().position(|p| p.id == perk_id) {
            if character.perks[pos].ranks > 1 {
                character.perks[pos].ranks -= 1;
            } else {
                character.perks.remove(pos);
            }
        }
    }

    // reapply skill caps now that level changed
    character.skills.apply_max(&character.clone());
    match db.save_character(character) {
        Ok(_) => {},
        Err(e) => eprintln!("Failed to save character: {e}"),
    }
    state.new_character(character);

    state.skill_choice = i32::MAX;
    state.perk_choice = i32::MAX;
}

/*

REVIEW


pub fn build_rules_bundle(/* TODO: db rows or json */) -> RulesBundle { todo!("Load from embedded JSON or SQLite") }
pub fn save_character_to_json(_character: &FullCharacter) -> Result<String, Box<dyn Error>> { todo!("FullCharacterSave::from(character) + serde_json::to_string_pretty") }
pub fn load_character_from_json(_json: &str, _rules: &RulesBundle) -> Result<FullCharacter, Box<dyn Error>> { todo!("serde_json::from_str + into_full_character") }
*/