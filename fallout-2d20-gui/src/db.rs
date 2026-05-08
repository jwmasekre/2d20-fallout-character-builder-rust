// src/db.rs
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use anyhow::{Result};
use uuid::Uuid;
use std::str::FromStr;

use crate::character::{AmmoInv, Character, Perk, Player, SkillBlock, SpecialBlock, TagType, WeaponMods, Party, Origin, Background, Special, Skills, Trait, Apparel, Weapon, AmmoData, Gear, Consumable, RobotModule, Version, resolve_prerelease, Skill, Limbs, MutantType, CompanionType, RobotType, MeleeModifiers, Junk, BaseDR};
use crate::screens::perk_select::perk_description;
use crate::screens::background_select::{resolve_apparel_type, resolve_consumable_type, resolve_apparel_covers, resolve_mod_effect, WeaponQuality, WeaponEffect, parse_damage_type, resolve_weapon_slot};

pub struct Db {
    pub pool: SqlitePool,
    pub runtime: tokio::runtime::Runtime,
}

impl Db {
    pub fn connect(db_path: &str) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()?;

        let options = SqliteConnectOptions::from_str(db_path)?
            .create_if_missing(true)  // creates the .db file on first run
            .pragma("foreign_keys", "ON");

        let pool = runtime.block_on(async {
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(options)
                .await
        })?;

        /*
        // Run migrations on connect
        runtime.block_on(async {
            sqlx::migrate!("./migrations").run(&pool).await
        })?;
        */
        Ok(Self { pool, runtime })
    }

    pub fn block_on<F, T>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        self.runtime.block_on(fut).map_err(anyhow::Error::from)
    }
    pub fn create_player(&self, name: &str) -> Uuid {
        let uid = uuid::Uuid::now_v7();
        let id = uid.clone().to_string();
        self.block_on(async {
            sqlx::query!("INSERT INTO players (id, username) VALUES (?1, ?2)", id, name)
                .execute(&self.pool).await
        }).ok();
        uid
    }
    pub fn create_party(&self, name: &str) -> Uuid {
        let uid = uuid::Uuid::now_v7();
        let id = uid.to_string();
        self.block_on(async {
            sqlx::query!("INSERT INTO parties (id, name, ap_gm, ap_players) VALUES (?1, ?2, ?3, ?4)", id, name, 0, 6)
                .execute(&self.pool).await
        }).ok();
        uid
    }
    pub fn save_character(&self, character: &Character) -> anyhow::Result<()> {
        let id = character.id.to_string();
        let player_id = character.player.id.to_string();
        let party_id = character.party.id.to_string();
        let origin_id = character.origin.as_ref().map(|o| o.id);
        let background_id = character.background.as_ref().map(|b| b.id);
        let misc_json = serde_json::to_string(&character.misc).unwrap_or_default();
        let prerelease_str = character.version.prerelease.as_str();
        let (junk_c, junk_u, junk_r) = (character.junk.common, character.junk.uncommon, character.junk.rare);

        self.block_on(async {
            let mut tx = self.pool.begin().await?;
            //clear out any old data

            sqlx::query!("DELETE FROM character_perks WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM flagged_perks WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM perk_options WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM character_traits WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM character_apparel WHERE character_id = ?", id)
                .execute(&mut *tx).await?;
            
            sqlx::query!("DELETE FROM character_weapon_mods WHERE character_weapon_id IN (SELECT id FROM character_weapons WHERE character_id = ?)", id)
                .execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_weapons WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM character_ammo WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM character_gear WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM character_consumables WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM character_robot_modules WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            sqlx::query!("DELETE FROM party_membership WHERE character_id = ?", id)
                .execute(&mut *tx).await?;

            //create or overwrite the character
            sqlx::query!(
                r#"INSERT OR REPLACE INTO characters (
                    id, player_id, character_name, xp, origin, background,
                    luck_points, current_health, rad_points,
                    head_inj, la_inj, ra_inj, torso_inj, ll_inj, rl_inj,
                    optic_inj, body_inj, a1_inj, a2_inj, a3_inj,
                    thrust_inj, lt_inj, rt_inj, wheel_inj,
                    party_id, misc, junk_c, junk_u, junk_r, notes,
                    version_maj, version_min, version_pat,
                    prerelease, prerelease_ver
                ) VALUES (
                    ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,
                    ?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,
                    ?28,?29,?30,?31,?32,?33,?34,?35
                )"#,
                id, player_id,
                character.name,
                character.xp,
                origin_id,
                background_id,
                character.luck_points,
                character.hp,
                character.rad_points,
                character.limb_dr.head.injuries,
                character.limb_dr.arm_left.injuries,
                character.limb_dr.arm_right.injuries,
                character.limb_dr.torso.injuries,
                character.limb_dr.leg_left.injuries,
                character.limb_dr.leg_right.injuries,
                character.limb_dr.optics.injuries,
                character.limb_dr.body.injuries,
                character.limb_dr.arm_1.injuries,
                character.limb_dr.arm_2.injuries,
                character.limb_dr.arm_3.injuries,
                character.limb_dr.thruster.injuries,
                character.limb_dr.track_left.injuries,
                character.limb_dr.track_right.injuries,
                character.limb_dr.wheel.injuries,
                party_id,
                misc_json,
                junk_c,
                junk_u,
                junk_r,
                character.notes,
                character.version.major,
                character.version.minor,
                character.version.patch,
                prerelease_str,
                character.version.prerelease_ver,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT char: {e}")))?;

            sqlx::query!(
                r#"INSERT OR REPLACE INTO character_special
                (character_id, strength, perception, endurance, charisma, intelligence, agility, luck)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
                id,
                character.special.strength.value,
                character.special.perception.value,
                character.special.endurance.value,
                character.special.charisma.value,
                character.special.intelligence.value,
                character.special.agility.value,
                character.special.luck.value,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT spec: {e}")))?;
            
            fn is_gifted(stat: SpecialBlock) -> i32 {
                if stat.gifted { 1 } else { 0 }
            }
            let g_str = is_gifted(character.special.strength.clone());
            let g_per = is_gifted(character.special.perception.clone());
            let g_end = is_gifted(character.special.endurance.clone());
            let g_cha = is_gifted(character.special.charisma.clone());
            let g_int = is_gifted(character.special.intelligence.clone());
            let g_agi = is_gifted(character.special.agility.clone());
            let g_lck = is_gifted(character.special.luck.clone());
            sqlx::query!(
                r#"INSERT OR REPLACE INTO character_special_gifted
                (character_id, strength, perception, endurance, charisma, intelligence, agility, luck)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
                id,
                g_str,
                g_per,
                g_end,
                g_cha,
                g_int,
                g_agi,
                g_lck,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT gift: {e}")))?;

            sqlx::query!(
                r#"INSERT OR REPLACE INTO character_special_training
                (character_id, strength, perception, endurance, charisma, intelligence, agility, luck)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
                id,
                character.special.strength.trained,
                character.special.perception.trained,
                character.special.endurance.trained,
                character.special.charisma.trained,
                character.special.intelligence.trained,
                character.special.agility.trained,
                character.special.luck.trained,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT train: {e}")))?;

            let sk = &character.skills;
            sqlx::query!(
                r#"INSERT OR REPLACE INTO character_skills
                (character_id, athletics, barter, bigGuns, energyWeapons, explosives,
                    lockpick, medicine, meleeWeapons, pilot, repair, science,
                    smallGuns, sneak, speech, survival, throwing, unarmed)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)"#,
                id,
                sk.athletics.total,
                sk.barter.total,
                sk.big_guns.total,
                sk.energy_weapons.total,
                sk.explosives.total,
                sk.lockpick.total,
                sk.medicine.total,
                sk.melee_weapons.total,
                sk.pilot.total,
                sk.repair.total,
                sk.science.total,
                sk.small_guns.total,
                sk.sneak.total,
                sk.speech.total,
                sk.survival.total,
                sk.throwing.total,
                sk.unarmed.total,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT skill: {e}")))?;

            fn is_tagged(skill: SkillBlock) -> i32 {
                if skill.tagged == TagType::Standard { 1 } else { 0 }
            }
            let t_at = is_tagged(sk.athletics.clone());
			let t_bt = is_tagged(sk.barter.clone());
			let t_bg = is_tagged(sk.big_guns.clone());
            let t_ew = is_tagged(sk.energy_weapons.clone());
			let t_ex = is_tagged(sk.explosives.clone());
			let t_lp = is_tagged(sk.lockpick.clone());
            let t_md = is_tagged(sk.medicine.clone());
			let t_mw = is_tagged(sk.melee_weapons.clone());
			let t_pt = is_tagged(sk.pilot.clone());
            let t_rp = is_tagged(sk.repair.clone());
			let t_sc = is_tagged(sk.science.clone());
			let t_sg = is_tagged(sk.small_guns.clone());
            let t_sn = is_tagged(sk.sneak.clone());
			let t_sp = is_tagged(sk.speech.clone());
			let t_sv = is_tagged(sk.survival.clone());
            let t_th = is_tagged(sk.throwing.clone());
			let t_un = is_tagged(sk.unarmed.clone());
            fn is_trait(skill: SkillBlock) -> i32 {
                if skill.tagged == TagType::Trait { 1 } else { 0 }
            }
            let tt_at = is_trait(sk.athletics.clone());
			let tt_bt = is_trait(sk.barter.clone());
			let tt_bg = is_trait(sk.big_guns.clone());
            let tt_ew = is_trait(sk.energy_weapons.clone());
			let tt_ex = is_trait(sk.explosives.clone());
			let tt_lp = is_trait(sk.lockpick.clone());
            let tt_md = is_trait(sk.medicine.clone());
			let tt_mw = is_trait(sk.melee_weapons.clone());
			let tt_pt = is_trait(sk.pilot.clone());
            let tt_rp = is_trait(sk.repair.clone());
			let tt_sc = is_trait(sk.science.clone());
			let tt_sg = is_trait(sk.small_guns.clone());
            let tt_sn = is_trait(sk.sneak.clone());
			let tt_sp = is_trait(sk.speech.clone());
			let tt_sv = is_trait(sk.survival.clone());
            let tt_th = is_trait(sk.throwing.clone());
			let tt_un = is_trait(sk.unarmed.clone());
            fn is_perk(skill: SkillBlock) -> i32 {
                if skill.tagged == TagType::Perk { 1 } else { 0 }
            }
            let tp_at = is_perk(sk.athletics.clone());
			let tp_bt = is_perk(sk.barter.clone());
			let tp_bg = is_perk(sk.big_guns.clone());
            let tp_ew = is_perk(sk.energy_weapons.clone());
			let tp_ex = is_perk(sk.explosives.clone());
			let tp_lp = is_perk(sk.lockpick.clone());
            let tp_md = is_perk(sk.medicine.clone());
			let tp_mw = is_perk(sk.melee_weapons.clone());
			let tp_pt = is_perk(sk.pilot.clone());
            let tp_rp = is_perk(sk.repair.clone());
			let tp_sc = is_perk(sk.science.clone());
			let tp_sg = is_perk(sk.small_guns.clone());
            let tp_sn = is_perk(sk.sneak.clone());
			let tp_sp = is_perk(sk.speech.clone());
			let tp_sv = is_perk(sk.survival.clone());
            let tp_th = is_perk(sk.throwing.clone());
			let tp_un = is_perk(sk.unarmed.clone());
             sqlx::query!(
                r#"INSERT OR REPLACE INTO character_tags
                (character_id, athletics, barter, bigGuns, energyWeapons, explosives,
                    lockpick, medicine, meleeWeapons, pilot, repair, science,
                    smallGuns, sneak, speech, survival, throwing, unarmed)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)"#,
                id,
                t_at,
				t_bt,
				t_bg,
                t_ew,
				t_ex,
				t_lp,
                t_md,
				t_mw,
				t_pt,
                t_rp,
				t_sc,
				t_sg,
                t_sn,
				t_sp,
				t_sv,
                t_th,
				t_un,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT tag: {e}")))?;

            sqlx::query!(
                r#"INSERT OR REPLACE INTO character_tags_trait
                (character_id, athletics, barter, bigGuns, energyWeapons, explosives,
                    lockpick, medicine, meleeWeapons, pilot, repair, science,
                    smallGuns, sneak, speech, survival, throwing, unarmed)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)"#,
                id,
                tt_at,
				tt_bt,
				tt_bg,
                tt_ew,
				tt_ex,
				tt_lp,
                tt_md,
				tt_mw,
				tt_pt,
                tt_rp,
				tt_sc,
				tt_sg,
                tt_sn,
				tt_sp,
				tt_sv,
                tt_th,
				tt_un,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT ttag: {e}")))?;

            sqlx::query!(
                r#"INSERT OR REPLACE INTO character_tags_perk
                (character_id, athletics, barter, bigGuns, energyWeapons, explosives,
                    lockpick, medicine, meleeWeapons, pilot, repair, science,
                    smallGuns, sneak, speech, survival, throwing, unarmed)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)"#,
                id,
                tp_at,
				tp_bt,
				tp_bg,
                tp_ew,
				tp_ex,
				tp_lp,
                tp_md,
				tp_mw,
				tp_pt,
                tp_rp,
				tp_sc,
				tp_sg,
                tp_sn,
				tp_sp,
				tp_sv,
                tp_th,
				tp_un,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT ptag: {e}")))?;

            let sk_at = serde_json::to_string(&sk.athletics.skilled).unwrap_or_default();
            let sk_bt = serde_json::to_string(&sk.barter.skilled).unwrap_or_default();
            let sk_bg = serde_json::to_string(&sk.big_guns.skilled).unwrap_or_default();
            let sk_ew = serde_json::to_string(&sk.energy_weapons.skilled).unwrap_or_default();
            let sk_ex = serde_json::to_string(&sk.explosives.skilled).unwrap_or_default();
            let sk_lp = serde_json::to_string(&sk.lockpick.skilled).unwrap_or_default();
            let sk_md = serde_json::to_string(&sk.medicine.skilled).unwrap_or_default();
            let sk_mw = serde_json::to_string(&sk.melee_weapons.skilled).unwrap_or_default();
            let sk_pt = serde_json::to_string(&sk.pilot.skilled).unwrap_or_default();
            let sk_rp = serde_json::to_string(&sk.repair.skilled).unwrap_or_default();
            let sk_sc = serde_json::to_string(&sk.science.skilled).unwrap_or_default();
            let sk_sg = serde_json::to_string(&sk.small_guns.skilled).unwrap_or_default();
            let sk_sn = serde_json::to_string(&sk.sneak.skilled).unwrap_or_default();
            let sk_sp = serde_json::to_string(&sk.speech.skilled).unwrap_or_default();
            let sk_sv = serde_json::to_string(&sk.survival.skilled).unwrap_or_default();
            let sk_th = serde_json::to_string(&sk.throwing.skilled).unwrap_or_default();
            let sk_un = serde_json::to_string(&sk.unarmed.skilled).unwrap_or_default();
            sqlx::query!(
                r#"INSERT OR REPLACE INTO character_skills_skilled
                (character_id, athletics, barter, bigGuns, energyWeapons, explosives,
                    lockpick, medicine, meleeWeapons, pilot, repair, science,
                    smallGuns, sneak, speech, survival, throwing, unarmed)
                VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)"#,
                id,
                sk_at,
                sk_bt,
                sk_bg,
                sk_ew,
                sk_ex,
                sk_lp,
                sk_md,
                sk_mw,
                sk_pt,
                sk_rp,
                sk_sc,
                sk_sg,
                sk_sn,
                sk_sp,
                sk_sv,
                sk_th,
                sk_un,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT skilled: {e}")))?;

            //reinsert all the data
            for perk in &character.perks {
                sqlx::query!(
                    "INSERT INTO character_perks (character_id, perk_id, rank) VALUES (?1,?2,?3)",
                    id, perk.id, perk.ranks,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT perk: {e}")))?;
            }
            
            for perk_id in &character.flagged_perks {
                sqlx::query!(
                    "INSERT INTO flagged_perks (character_id, perk_id) VALUES (?1,?2)",
                    id, perk_id,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT pflag: {e}")))?;
            }

            let bwlk = character.perks.iter().find(|p| p.id == 12).map(|p| p.name.clone());
            let mmcf = character.perks.iter().find(|p| p.id == 110).map(|p| p.name.clone());
            if bwlk.is_some() || mmcf.is_some() {
                if mmcf.is_none() {
                    let bwlk_str = bwlk.unwrap();
                    sqlx::query!(
                        "INSERT INTO perk_options (character_id, bwlk_selection) VALUES (?1,?2)",
                        id, bwlk_str,
                    ).execute(&mut *tx).await
                        .map_err(|e| sqlx::Error::Protocol(format!("INSERT bwlk: {e}")))?;
                } else if bwlk.is_none() {
                    let mmcf_str = mmcf.unwrap();
                    sqlx::query!(
                        "INSERT INTO perk_options (character_id, mmcf_selection) VALUES (?1,?2)",
                        id, mmcf_str,
                    ).execute(&mut *tx).await
                        .map_err(|e| sqlx::Error::Protocol(format!("INSERT mmcf: {e}")))?;
                } else {
                    let bwlk_str = bwlk.unwrap();
                    let mmcf_str = mmcf.unwrap();
                    sqlx::query!(
                        "INSERT INTO perk_options (character_id, bwlk_selection, mmcf_selection) VALUES (?1,?2,?3)",
                        id, bwlk_str, mmcf_str,
                    ).execute(&mut *tx).await
                        .map_err(|e| sqlx::Error::Protocol(format!("INSERT blmc: {e}")))?;
                }
            }

            for t in &character.traits {
                sqlx::query!(
                    "INSERT INTO character_traits (character_id, trait_id) VALUES (?1,?2)",
                    id, t.id,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT trait: {e}")))?;
            }

            for a in &character.apparel {
                sqlx::query!(
                    "INSERT INTO character_apparel (apparel_id, character_id, equipped, legendary)
                    VALUES (?1,?2,?3,?4)",
                    a.id, id, a.equipped, false,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT apparel: {e}")))?;
            }

            for w in &character.weapons {
                let result = sqlx::query!(
                    "INSERT INTO character_weapons (weapon_id, character_id) VALUES (?1,?2)",
                    w.id, id,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT weap: {e}")))?;

                let cw_id = result.last_insert_rowid();
                let mods: Vec<&WeaponMods> = w.mods.iter().filter(|wm| wm.id != 0).collect();
                for m in mods {
                    sqlx::query!(
                        "INSERT INTO character_weapon_mods (character_weapon_id, mod_id) VALUES (?1,?2)",
                        cw_id, m.id,
                    ).execute(&mut *tx).await
                        .map_err(|e| sqlx::Error::Protocol(format!("INSERT wmod: cw_id={}, mod_id={:?} - {e}", cw_id, m)))?;
                }
            }

            let ammo: Vec<&AmmoInv> = character.ammo.iter().filter(|a| a.ammo.id != 0).collect();
            for a in ammo {
                sqlx::query!(
                    "INSERT INTO character_ammo (ammo_id, quantity, character_id) VALUES (?1,?2,?3)",
                    a.ammo.id, a.quantity, id,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT ammo: {:?} - {e}", a)))?;
            }

            for g in &character.gear {
                sqlx::query!(
                    "INSERT INTO character_gear (gear_id, quantity, character_id) VALUES (?1,?2,?3)",
                    g.id, g.quantity, id,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT gear: {e}")))?;
            }

            for c in &character.consumables {
                sqlx::query!(
                    "INSERT INTO character_consumables (consumable_id, quantity, character_id) VALUES (?1,?2,?3)",
                    c.id, c.quantity, id,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT consu: {e}")))?;
            }

            for m in &character.robot_modules {
                sqlx::query!(
                    "INSERT INTO character_robot_modules (module_id, character_id, equipped) VALUES (?1,?2,?3)",
                    m.id, id, m.installed,
                ).execute(&mut *tx).await
                    .map_err(|e| sqlx::Error::Protocol(format!("INSERT rmod: {e}")))?;
            }

            sqlx::query!(
                "INSERT INTO party_membership (party_id, character_id) VALUES (?1,?2)",
                party_id, id,
            ).execute(&mut *tx).await
                .map_err(|e| sqlx::Error::Protocol(format!("INSERT pmemb: {e}")))?;
            

            tx.commit().await
        })
    }
    pub fn load_character(&self, character_id: &str) -> anyhow::Result<Character> {
        self.block_on(async {

            let row = sqlx::query!(
                "SELECT * FROM characters WHERE id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load characters: {e}")))?;

            let player_row = sqlx::query!(
                "SELECT id, username FROM players WHERE id = ?", row.player_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load player: {e}")))?;
            let player = Player {
                id: Uuid::parse_str(&player_row.id.unwrap_or_default()).unwrap_or_default(),
                name: player_row.username.unwrap_or_default(),
            };

            let party_row = sqlx::query!(
                "SELECT id, name, ap_gm, ap_players FROM parties WHERE id = ?", row.party_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load party: {e}")))?;
            let party = Party {
                id: Uuid::parse_str(&party_row.id.unwrap_or_default()).unwrap_or_default(),
                name: party_row.name.unwrap_or_default(),
                ap_gm: party_row.ap_gm.unwrap_or(0) as i32,
                ap_players: party_row.ap_players.unwrap_or(6) as i32,
                max_ap: 6,
            };

            let origin = if let Some(origin_id) = row.origin {
                sqlx::query!("SELECT * FROM origins WHERE id = ?", origin_id)
                    .fetch_optional(&self.pool).await
                    .map_err(|e| sqlx::Error::Protocol(format!("load origin: {e}")))?
                    .map(|o| Origin {
                        id: o.id as i32,
                        name: o.name.unwrap_or_default(),
                        desc: o.description.unwrap_or_default(),
                        can_ghoul: o.can_ghoul == Some(1),
                    })
            } else { None };

            let background = if let Some(bg_id) = row.background {
                sqlx::query!("SELECT * FROM backgrounds WHERE id = ?", bg_id)
                    .fetch_optional(&self.pool).await
                    .map_err(|e| sqlx::Error::Protocol(format!("load background: {e}")))?
                    .map(|b| Background {
                        id: b.id as i32,
                        name: b.name.unwrap_or_default(),
                        desc: b.description.unwrap_or_default(),
                    })
            } else { None };

            let sp = sqlx::query!(
                "SELECT * FROM character_special WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load special: {e}")))?;

            let sp_gifted = sqlx::query!(
                "SELECT * FROM character_special_gifted WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load special_gifted: {e}")))?;

            let sp_training = sqlx::query!(
                "SELECT * FROM character_special_training WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load special_training: {e}")))?;

            macro_rules! build_special {
                ($field:ident, $val:expr, $gift:expr, $train:expr) => {
                    SpecialBlock {
                        value:   $val.unwrap_or(0) as i32,
                        gifted:  $gift.unwrap_or(0) != 0,
                        trained: $train.unwrap_or(0) as i32,
                        max: 10,
                    }
                };
            }

            let special = Special {
                strength: build_special!(strength, sp.strength, sp_gifted.strength, sp_training.strength),
                perception: build_special!(perception, sp.perception, sp_gifted.perception, sp_training.perception),
                endurance: build_special!(endurance, sp.endurance, sp_gifted.endurance, sp_training.endurance),
                charisma: build_special!(charisma, sp.charisma, sp_gifted.charisma, sp_training.charisma),
                intelligence: build_special!(intelligence, sp.intelligence, sp_gifted.intelligence, sp_training.intelligence),
                agility: build_special!(agility, sp.agility, sp_gifted.agility, sp_training.agility),
                luck: build_special!(luck, sp.luck, sp_gifted.luck, sp_training.luck),
            };

            let sk = sqlx::query!(
                "SELECT * FROM character_skills WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load skills: {e}")))?;

            let sk_tags = sqlx::query!(
                "SELECT * FROM character_tags WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load tags: {e}")))?;

            let sk_tags_trait = sqlx::query!(
                "SELECT * FROM character_tags_trait WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load tags_trait: {e}")))?;

            let sk_tags_perk = sqlx::query!(
                "SELECT * FROM character_tags_perk WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load tags_perk: {e}")))?;

            let sk_skilled = sqlx::query!(
                "SELECT * FROM character_skills_skilled WHERE character_id = ?", character_id
            ).fetch_one(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load skills_skilled: {e}")))?;

            macro_rules! build_skill {
                ($val:expr, $tag:expr, $ttag:expr, $ptag:expr, $skilled:expr) => {{
                    let total = $val.unwrap_or(0) as i32;
                    let tagged = if $ptag.unwrap_or(0) != 0 {
                        TagType::Perk
                    } else if $ttag.unwrap_or(0) != 0 {
                        TagType::Trait
                    } else if $tag.unwrap_or(0) != 0 {
                        TagType::Standard
                    } else {
                        TagType::None
                    };
                    let ranks = total - if tagged == TagType::None { 0 } else { 2 };
                    SkillBlock {
                        total,
                        tagged,
                        skilled: serde_json::from_str(
                            &$skilled.unwrap_or_default()
                        ).unwrap_or_default(),
                        ranks,
                        max: 6
                    }}
                };
            }

            let skills = Skills {
                athletics: build_skill!(sk.athletics, sk_tags.athletics, sk_tags_trait.athletics, sk_tags_perk.athletics, sk_skilled.athletics),
                barter: build_skill!(sk.barter, sk_tags.barter, sk_tags_trait.barter, sk_tags_perk.barter, sk_skilled.barter),
                big_guns: build_skill!(sk.bigGuns, sk_tags.bigGuns, sk_tags_trait.bigGuns, sk_tags_perk.bigGuns, sk_skilled.bigGuns),
                energy_weapons: build_skill!(sk.energyWeapons, sk_tags.energyWeapons, sk_tags_trait.energyWeapons, sk_tags_perk.energyWeapons, sk_skilled.energyWeapons),
                explosives: build_skill!(sk.explosives, sk_tags.explosives, sk_tags_trait.explosives, sk_tags_perk.explosives, sk_skilled.explosives),
                lockpick: build_skill!(sk.lockpick, sk_tags.lockpick, sk_tags_trait.lockpick, sk_tags_perk.lockpick, sk_skilled.lockpick),
                medicine: build_skill!(sk.medicine, sk_tags.medicine, sk_tags_trait.medicine, sk_tags_perk.medicine, sk_skilled.medicine),
                melee_weapons: build_skill!(sk.meleeWeapons, sk_tags.meleeWeapons, sk_tags_trait.meleeWeapons, sk_tags_perk.meleeWeapons, sk_skilled.meleeWeapons),
                pilot: build_skill!(sk.pilot, sk_tags.pilot, sk_tags_trait.pilot, sk_tags_perk.pilot, sk_skilled.pilot),
                repair: build_skill!(sk.repair, sk_tags.repair, sk_tags_trait.repair, sk_tags_perk.repair, sk_skilled.repair),
                science: build_skill!(sk.science, sk_tags.science, sk_tags_trait.science, sk_tags_perk.science, sk_skilled.science),
                small_guns: build_skill!(sk.smallGuns, sk_tags.smallGuns, sk_tags_trait.smallGuns, sk_tags_perk.smallGuns, sk_skilled.smallGuns),
                sneak: build_skill!(sk.sneak, sk_tags.sneak, sk_tags_trait.sneak, sk_tags_perk.sneak, sk_skilled.sneak),
                speech: build_skill!(sk.speech, sk_tags.speech, sk_tags_trait.speech, sk_tags_perk.speech, sk_skilled.speech),
                survival: build_skill!(sk.survival, sk_tags.survival, sk_tags_trait.survival, sk_tags_perk.survival, sk_skilled.survival),
                throwing: build_skill!(sk.throwing, sk_tags.throwing, sk_tags_trait.throwing, sk_tags_perk.throwing, sk_skilled.throwing),
                unarmed: build_skill!(sk.unarmed, sk_tags.unarmed, sk_tags_trait.unarmed, sk_tags_perk.unarmed, sk_skilled.unarmed),
            };

            let perk_opts = sqlx::query!(
                "SELECT bwlk_selection, mmcf_selection FROM perk_options WHERE character_id = ?",
                character_id
            ).fetch_optional(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load perk_options: {e}")))?;

            let perk_rows = sqlx::query!(
                r#"SELECT cp.perk_id, cp.rank, p.name, p.description
                FROM character_perks cp
                JOIN perks p ON p.id = cp.perk_id
                WHERE cp.character_id = ?"#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load perks: {e}")))?;

            let perks: Vec<Perk> = perk_rows.iter().map(|p| {
                let perk_id = p.perk_id.unwrap_or_default() as i32;
                // reattach perk option selections by perk id
                let name = if perk_id == 12 {
                    perk_opts.as_ref()
                        .and_then(|o| o.bwlk_selection.clone())
                        .unwrap_or_else(|| p.name.clone().unwrap_or_default())
                } else if perk_id == 110 {
                    perk_opts.as_ref()
                        .and_then(|o| o.mmcf_selection.clone())
                        .unwrap_or_else(|| p.name.clone().unwrap_or_default())
                } else {
                    p.name.clone().unwrap_or_default()
                };
                let desc = perk_description(p.description.clone().unwrap_or_default());
                Perk {
                    id:    perk_id,
                    name,
                    ranks: p.rank.unwrap_or(1) as i32,
                    desc,
                }
            }).collect();

            // ── flagged perks ────────────────────────────────────────────
            let flagged_perks: Vec<i32> = sqlx::query!(
                "SELECT perk_id FROM flagged_perks WHERE character_id = ?", character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load flagged_perks: {e}")))?
                .into_iter()
                .map(|r| r.perk_id.unwrap_or_default() as i32)
                .collect();

            let traits: Vec<Trait> = sqlx::query!(
                r#"SELECT ct.trait_id, t.name, t.description
                FROM character_traits ct
                JOIN traits t ON t.id = ct.trait_id
                WHERE ct.character_id = ?"#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load traits: {e}")))?
                .iter().map(|t| Trait {
                    id:   t.trait_id.unwrap_or_default() as i32,
                    name: t.name.clone().unwrap_or_default(),
                    desc: t.description.clone().unwrap_or_default(),
                }).collect();

            let mut apparel: Vec<Apparel> = vec![];
            let apparel_rows = sqlx::query!(
                r#"
                SELECT ca.equipped, a.*, a.type as atype
                FROM character_apparel ca
                JOIN apparel a ON a.id = ca.apparel_id
                WHERE ca.character_id = ?
                "#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load apparel: {e}")))?;
            for row in apparel_rows {
                let apparel_id = row.id as i32;
                let cover_list: Vec<i64> = sqlx::query!(
                    r#"SELECT
                        ac.id as cid
                    FROM apparel_covers ac
                    WHERE ac.apparel_id = ?
                    "#,
                    apparel_id
                ).fetch_all(&self.pool).await
                    .map_err(|e| sqlx::Error::Protocol(format!("load covers: {e}"))).unwrap_or_default().iter().map(|c| c.cid).collect();
                let covers = resolve_apparel_covers(cover_list);
                apparel.push(Apparel {
                    id: apparel_id,
                    name: row.name.clone().unwrap_or_default(),
                    equipped: row.equipped.unwrap_or(0) != 0,
                    apparel_type: resolve_apparel_type(row.atype.unwrap_or(0)),
                    covers,
                    effects: serde_json::from_str(&row.eff.unwrap_or_default()).unwrap_or_default(),
                    prefix: "".to_string(),
                    ph_dr: row.phys_dr.unwrap_or_default() as i32,
                    en_dr: row.enrg_dr.unwrap_or_default() as i32,
                    rd_dr: row.rads_dr.unwrap_or_default() as i32,
                    wgt: row.wgt.unwrap_or_default() as i32,
                })
            };

            let weapon_rows = sqlx::query!(
                r#"
                SELECT cw.id AS cw_id, w.*, a.name as aname, w.type as skill
                FROM character_weapons cw
                JOIN weapons w ON w.id = cw.weapon_id
                JOIN ammo a ON a.id = w.ammo
                WHERE cw.character_id = ?
                "#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load weapons: {e}")))?;

            let mut weapons: Vec<Weapon> = vec![];
            for wrow in weapon_rows {
                let weapon_id = wrow.id;
                //grab qualities
                let qual_rows = sqlx::query!(
                        r#"SELECT q.name, wq.qual_val
                        FROM weapon_quals wq
                        JOIN qualities q ON q.id = wq.qual_id
                        WHERE wq.weapon_id = ?"#,
                        weapon_id
                    ).fetch_all(&self.pool).await.unwrap_or_default();
                let mut qualities: Vec<WeaponQuality> = qual_rows.iter().map(|q| WeaponQuality {
                    name: q.name.clone().unwrap_or_default(),
                    value: q.qual_val.map(|v| v as i32),
                }).collect();

                //grab effects
                let eff_rows = sqlx::query!(
                        r#"SELECT de.name, we.effect_val
                        FROM weapon_effects we
                        JOIN dam_effects de ON de.id = we.effect_id
                        WHERE we.weapon_id = ?"#,
                        weapon_id
                    ).fetch_all(&self.pool).await.unwrap_or_default();
                let mut effects: Vec<WeaponEffect> = eff_rows.iter().map(|e| WeaponEffect {
                    name: e.name.clone().unwrap_or_default(),
                    value: e.effect_val.map(|v| v as i32),
                }).collect();

                let damage_str = wrow.dam.clone().unwrap_or_default();
                let mut damage: i32 = damage_str.trim_end_matches(|c: char| c.is_alphabetic()).parse().unwrap_or(0);
                let mut rate = wrow.rate.unwrap_or_default() as i32;
                let mut range = wrow.range.clone().unwrap_or_default();
                let damage_type_str = wrow.dtype.clone().unwrap_or("".to_string());
                let mut dam_type = parse_damage_type(&damage_type_str);
                let base_wgt = wrow.wgt.unwrap_or_default() as i32;
                let mut mod_wgt = 0;
                let name = wrow.name.clone().unwrap_or_default();
                let mut prefix = String::new();
                let ammo_name = wrow.aname.clone().unwrap_or("".to_string());

                let (spec_index, skill_index, skill) = match wrow.skill.unwrap_or(0) {
                    8 => (0,7,Skill::MeleeWeapons),
                    17 => (0,16,Skill::Unarmed),
                    12 => (5,11,Skill::SmallGuns),
                    16 => (5,15,Skill::Throwing),
                    4 => (1,3,Skill::EnergyWeapons),
                    5 => (1,4,Skill::Explosives),
                    3 => (2,2,Skill::BigGuns),
                    _  => (6,0,Skill::Athletics),
                };
                let special_i: Vec<i32> = special.special_block().iter().map(|s| s.value.clone()).collect();
                let skill_i: Vec<i32>  = skills.skill_block().iter().map(|s| s.total.clone()).collect();
                let tags: Vec<bool> = skills.skill_block().iter().map(|s| s.is_tagged()).collect();
                let spec_value = special_i[spec_index];
                let skill_total = skill_i[skill_index];
                let tag = tags[skill_index];
                let target = skill_total + spec_value;

                let mut mods: Vec<WeaponMods> = vec![];
                let mod_rows = sqlx::query!(
                    r#"
                    SELECT wm.*
                    FROM character_weapon_mods cwm
                    JOIN weapon_mods wm ON wm.id = cwm.mod_id
                    WHERE cwm.character_weapon_id = ?
                    "#,
                    weapon_id
                ).fetch_all(&self.pool).await
                    .map_err(|e| sqlx::Error::Protocol(format!("load weapon_mods: {e}")))?;

                for mrow in mod_rows {
                    mod_wgt = mrow.wgt.unwrap_or_default() as i32;
                    prefix += mrow.prefix.clone().unwrap_or_default().as_str();
                    let weapon_mod_eff = resolve_mod_effect(self, mrow.effects, &mut damage, &mut rate, &mut range, &mut effects, &mut qualities, &mut dam_type);
                    mods.push( WeaponMods {
                        slot: resolve_weapon_slot(mrow.slot.unwrap_or(0)),
                        installed: true,
                        id: mrow.id as i32,
                        name: mrow.name.unwrap_or_default(),
                        prefix: mrow.prefix.unwrap_or_default(),
                        wgt: mrow.wgt.unwrap_or(0) as i32,
                        damage_set: weapon_mod_eff.dam_set,
                        damage_chg: weapon_mod_eff.dam_add - weapon_mod_eff.dam_sub,
                        rate_set: weapon_mod_eff.rat_set,
                        rate_chg: weapon_mod_eff.rat_add - weapon_mod_eff.rat_sub,
                        ammo_set: weapon_mod_eff.ammo,
                        range_chg: weapon_mod_eff.rng_add - weapon_mod_eff.rng_sub,
                        effect_add: weapon_mod_eff.e_gain,
                        effect_rem: weapon_mod_eff.e_lose,
                        quality_add: weapon_mod_eff.q_gain,
                        quality_rem: weapon_mod_eff.q_lose,
                        slot_add: weapon_mod_eff.mods,
                        damage_type_set: Some(weapon_mod_eff.dam_type),
                        weapon_add: weapon_mod_eff.weap,
                        special_ability: weapon_mod_eff.unk,
                    })
                }

                let weap_eff_str: Vec<String> = effects.iter().map(|e| if e.value != Some(0) && e.value.is_some() { format!("{} {}", e.name, e.value.unwrap()) } else { e.name.clone() }).collect();
                let weap_qual_str: Vec<String> = qualities.iter().map(|q| if q.value != Some(0) && q.value.is_some() { format!("{} {}", q.name, q.value.unwrap()) } else { q.name.clone() }).collect();
                let weight = base_wgt + mod_wgt;

                weapons.push(Weapon {
                    id:   wrow.id as i32,
                    name,
                    prefix,
                    skill,
                    target,
                    tag,
                    damage,
                    effects: weap_eff_str,
                    dam_type,
                    rate,
                    range,
                    qualities: weap_qual_str,
                    ammo: ammo_name.clone(),
                    wgt: weight,
                    mods,
                });
            }

            let ammo: Vec<AmmoInv> = sqlx::query!(
                r#"SELECT ca.quantity, a.*
                FROM character_ammo ca
                JOIN ammo a ON a.id = ca.ammo_id
                WHERE ca.character_id = ?"#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load ammo: {e}")))?
                .iter().map(|a| AmmoInv {
                    ammo: AmmoData {
                        id:   a.id as i32,
                        name: a.name.clone().unwrap_or_default(),
                        wgt: a.wgt.unwrap_or_default() as i32,
                    },
                    quantity: a.quantity.unwrap_or(0) as i32,
                }).collect();

            let gear: Vec<Gear> = sqlx::query!(
                r#"SELECT cg.quantity, g.*
                FROM character_gear cg
                JOIN gear g ON g.id = cg.gear_id
                WHERE cg.character_id = ?"#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load gear: {e}")))?
                .iter().map(|g| Gear {
                    id:       g.id as i32,
                    name:     g.name.clone().unwrap_or_default(),
                    quantity: g.quantity.unwrap_or(0) as i32,
                    wgt: g.wgt.unwrap_or(0) as i32,
                    effect: serde_json::from_str(&g.eff.clone().unwrap_or_default()).unwrap_or_default()
                }).collect();

            let consumables: Vec<Consumable> = sqlx::query!(
                r#"SELECT cc.quantity, c.*, c.type as ctype
                FROM character_consumables cc
                JOIN consumables c ON c.id = cc.consumable_id
                JOIN consumable_types ct on c.type = ct.id
                WHERE cc.character_id = ?"#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load consumables: {e}")))?
                .iter().map(|c| Consumable {
                    id:       c.id as i32,
                    name:     c.name.clone().unwrap_or_default(),
                    quantity: c.quantity.unwrap_or(0) as i32,
                    wgt: c.wgt.unwrap_or(0) as i32,
                    consumable_type: resolve_consumable_type(c.ctype.unwrap_or(0)),
                    health: c.heals.unwrap_or(0) as i32,
                    addiction: c.addiction.unwrap_or(0) as i32,
                    duration: c.duration.clone().unwrap_or("0".to_string()),
                    rads: c.rads.unwrap_or(0) as i32,
                    effects: serde_json::from_str(&c.eff.clone().unwrap_or_default()).unwrap_or_default()
                }).collect();

            let robot_modules: Vec<RobotModule> = sqlx::query!(
                r#"SELECT crm.equipped, rm.*
                FROM character_robot_modules crm
                JOIN robot_modules rm ON rm.id = crm.module_id
                WHERE crm.character_id = ?"#,
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load robot_modules: {e}")))?
                .iter().map(|m| RobotModule {
                    id:        m.id as i32,
                    name:      m.name.clone().unwrap_or_default(),
                    installed: m.equipped.unwrap_or(0) != 0,
                    effect: serde_json::from_str(&m.eff.clone().unwrap_or_default()).unwrap_or_default(),
                    wgt: m.wgt.unwrap_or(0) as i32,
                }).collect();

            let misc: Vec<String> = serde_json::from_str(
                &row.misc.unwrap_or_default()
            ).unwrap_or_default();

            let version = Version {
                major:          row.version_maj.unwrap_or(0) as i32,
                minor:          row.version_min.unwrap_or(0) as i32,
                patch:          row.version_pat.unwrap_or(0) as i32,
                prerelease:     resolve_prerelease(&row.prerelease.unwrap_or_default()),
                prerelease_ver: row.prerelease_ver.unwrap_or(0) as i32,
            };

            let mut character = Character {
                id:              Uuid::parse_str(&row.id.unwrap_or_default()).unwrap_or_default(),
                version,
                name:            row.character_name.unwrap_or_default(),
                player,
                party,
                level:           1,
                xp:              row.xp.unwrap_or(0) as i32,
                xp_next:         100,
                origin,
                background,
                traits,
                ghoul:           false,
                mutant:          MutantType::None,
                robot:           RobotType::None,
                companion:       CompanionType::None,
                robot_hat:       None,
                special,
                luck_points:     row.luck_points.unwrap_or(0) as i32,
                luck_points_max: 0,
                rad_points:      row.rad_points.unwrap_or(0) as i32,
                skills,
                perks,
                flagged_perks,
                melee_mod:       MeleeModifiers::new(),
                defense:         0,
                initiative:      0,
                hp:              row.current_health.unwrap_or(0) as i32,
                hp_max:          0,
                base_dr: BaseDR::new(),
                poison_dr:       0,
                limb_dr: Limbs::new(),
                weapons,
                ammo,
                apparel,
                robot_modules,
                consumables,
                gear,
                junk: Junk {common: row.junk_c.unwrap_or(0) as i32, uncommon: row.junk_u.unwrap_or(0) as i32, rare: row.junk_r.unwrap_or(0) as i32},
                misc,
                carry_wgt:       0,
                carry_wgt_max:   0,
                notes:           row.notes.unwrap_or_default(),
            };
            character.full_update();
            character.limb_dr.head.injuries = row.head_inj.unwrap_or(0) as i32;
            character.limb_dr.arm_left.injuries = row.la_inj.unwrap_or(0) as i32;
            character.limb_dr.arm_right.injuries = row.ra_inj.unwrap_or(0) as i32;
            character.limb_dr.torso.injuries = row.torso_inj.unwrap_or(0) as i32;
            character.limb_dr.leg_left.injuries = row.ll_inj.unwrap_or(0) as i32;
            character.limb_dr.leg_right.injuries = row.rl_inj.unwrap_or(0) as i32;
            character.limb_dr.optics.injuries = row.optic_inj.unwrap_or(0) as i32;
            character.limb_dr.body.injuries = row.body_inj.unwrap_or(0) as i32;
            character.limb_dr.arm_1.injuries = row.a1_inj.unwrap_or(0) as i32;
            character.limb_dr.arm_2.injuries = row.a2_inj.unwrap_or(0) as i32;
            character.limb_dr.arm_3.injuries = row.a3_inj.unwrap_or(0) as i32;
            character.limb_dr.thruster.injuries = row.thrust_inj.unwrap_or(0) as i32;
            character.limb_dr.track_left.injuries = row.lt_inj.unwrap_or(0) as i32;
            character.limb_dr.track_right.injuries = row.rt_inj.unwrap_or(0) as i32;
            character.limb_dr.wheel.injuries = row.wheel_inj.unwrap_or(0) as i32;
            //need to equip apparel
            Ok(character)
        }).map_err(anyhow::Error::from)
    }
}