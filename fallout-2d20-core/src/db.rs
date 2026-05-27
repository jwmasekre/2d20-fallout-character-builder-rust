use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use anyhow::{Result};
use uuid::Uuid;
use std::{collections::HashSet, str::FromStr};

use crate::{
    background_slots::{
        ApparelSelSlot,
        ConsumableSelSlot,
        ResolvedBackground,
        RobotModuleSelSlot,
        SlotSelection,
        WeaponSelSlot,
        resolve_apparel_slots,
        resolve_consumable_slots,
        resolve_robot_module_slots,
        resolve_weapon_slots,
    }, character::{
        AmmoData, AmmoInv, Apparel, Background, BaseDR, Character, CompanionType, Consumable, ConsumableType, Gear, Junk, Limbs, MeleeModifiers, MutantType, Origin, Party, Perk, Player, RobotModule, RobotType, Skill, SkillBlock, Skills, Special, SpecialBlock, TagType, Trait, Weapon, WeaponMods
    }, equip_apparel, parse_damage_type, perk_description, resolve_apparel_covers, resolve_apparel_type, resolve_consumable_type, resolve_mod_effect, resolve_prerelease, resolve_weapon_slot, roll_cd, roll_d20, states::OriginState, structs::{Version,
        WeaponEffect,
        WeaponQuality
    }
};

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
    pub fn delete_character(&self, id: &str) -> anyhow::Result<()> {
        self.block_on(async {
            let mut tx = self.pool.begin().await?;
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
            sqlx::query!("DELETE FROM character_skills_skilled WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_tags_perk WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_tags_trait WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_tags WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_skills WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_special_training WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_special_gifted WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM character_special WHERE character_id = ?", id).execute(&mut *tx).await?;
            sqlx::query!("DELETE FROM characters WHERE id = ?", id).execute(&mut *tx).await?;
            tx.commit().await?;
            Ok(())
        })
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
                    prerelease, prerelease_ver, caps
                ) VALUES (
                    ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,
                    ?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,
                    ?28,?29,?30,?31,?32,?33,?34,?35,?36
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
                character.caps,
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
                        ac.location_id as cid
                    FROM apparel_covers ac
                    WHERE ac.apparel_id = ?
                    "#,
                    apparel_id
                ).fetch_all(&self.pool).await
                    .map_err(|e| sqlx::Error::Protocol(format!("load covers: {e}"))).unwrap_or_default().iter().map(|c| c.cid.unwrap()).collect();
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
                    db_id: row.id,
                })
            };

            let weapon_rows = sqlx::query!(
                r#"
                SELECT cw.id AS cw_id, w.*, w.type as skill
                FROM character_weapons cw
                JOIN weapons w ON w.id = cw.weapon_id
                WHERE cw.character_id = ?
                "#,
                /*
                r#"
                SELECT cw.id AS cw_id, w.*, a.name as aname, w.type as skill
                FROM character_weapons cw
                JOIN weapons w ON w.id = cw.weapon_id
                JOIN ammo a ON a.id = w.ammo
                WHERE cw.character_id = ?
                "#,
                */
                character_id
            ).fetch_all(&self.pool).await
                .map_err(|e| sqlx::Error::Protocol(format!("load weapons: {e}")))?;

            let mut weapons: Vec<Weapon> = vec![];
            for wrow in weapon_rows {
                let weapon_id = wrow.id;
                //grab ammo, if present
                let ammo_row = sqlx::query!(
                    r#"SELECT a.name as aname
                    FROM ammo a
                    JOIN weapons w on w.ammo = a.id
                    WHERE w.id = ?
                    "#,
                    weapon_id
                ).fetch_optional(&self.pool).await
                    .map_err(|e| sqlx::Error::Protocol(format!("load weapon ammo: {e}")))?;
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
                let ammo_name = if ammo_row.is_some() {ammo_row.unwrap().aname.unwrap_or("".to_string())} else { "".to_string() };

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
                    let name_set = load_mod_effect_async(&self.pool).await;
                    let weapon_mod_eff = resolve_mod_effect(name_set, mrow.effects, &mut damage, &mut rate, &mut range, &mut effects, &mut qualities, &mut dam_type);
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

                let weap_eff_str: Vec<String> = effects.iter().map(|e| if e.value != Some(0) && e.value.is_some() {
                    if e.name.contains("X") {
                        e.name.replace("X", &e.value.unwrap().to_string())
                    } else {
                        format!("{} {}", e.name, e.value.unwrap())
                    }
                } else {
                    e.name.clone()
                }).collect();
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
                r#"SELECT crm.equipped, crm.id as db_id, rm.*
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
                    db_id: m.db_id,
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
                caps: row.caps.unwrap_or(0) as i32,
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
            equip_apparel(&mut character);
            character.limb_dr.update_dr(character.base_dr.clone(), character.perk_ranks(144), character.junk.common + character.junk.uncommon + character.junk.rare, character.perk_ranks(172));
            Ok(character)
        }).map_err(anyhow::Error::from)
    }
    pub fn update_apparel(&self, db_id: i64, eq: bool) {
        let eqv = if eq { 1 } else { 0 };
        self.block_on(async {
            sqlx::query!(
                "UPDATE character_apparel
                SET equipped = ?1
                WHERE id = ?2",
                eqv, db_id,
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn update_module(&self, db_id: i64, eq: bool) {
        let eqv = if eq { 1 } else { 0 };
        self.block_on(async {
            sqlx::query!(
                "UPDATE character_robot_modules
                SET equipped = ?1
                WHERE id = ?2",
                eqv, db_id,
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn update_lp(&self, character: &Character) {
        let id: &str = &character.id.to_string();
        self.block_on(async {
            sqlx::query!(
                "UPDATE characters
                SET luck_points = ?1
                WHERE id = ?2",
                character.luck_points, id
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn update_rp(&self, character: &Character) {
        let id: &str = &character.id.to_string();
        self.block_on(async {
            sqlx::query!(
                "UPDATE characters
                SET rad_points = ?1
                WHERE id = ?2",
                character.rad_points, id
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn update_hp(&self, character: &Character) {
        let id: &str = &character.id.to_string();
        self.block_on(async {
            sqlx::query!(
                "UPDATE characters
                SET current_health = ?1
                WHERE id = ?2",
                character.hp, id
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn update_xp(&self, character: &Character) {
        let id: &str = &character.id.to_string();
        self.block_on(async {
            sqlx::query!(
                "UPDATE characters
                SET xp = ?1
                WHERE id = ?2",
                character.xp, id
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn update_caps(&self, character: &Character) {
        let id: &str = &character.id.to_string();
        self.block_on(async {
            sqlx::query!(
                "UPDATE characters
                SET caps = ?1
                WHERE id = ?2",
                character.caps, id
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn update_notes(&self, character: &Character) {
        let id: &str = &character.id.to_string();
        self.block_on(async {
            sqlx::query!(
                "UPDATE characters
                SET notes = ?1
                WHERE id = ?2",
                character.notes, id
            ).execute(&self.pool).await
        }).ok();
    }
    pub fn get_all_weapons(&self) -> anyhow::Result<Vec<(i32, String, String, String)>> {
    // returns (weapon_id, name, skill_name, range)
        self.block_on(async {
            let rows = sqlx::query!(
                r#"SELECT w.id, w.name, s.name AS skill_name, w.range
                FROM weapons w
                JOIN skills s ON s.id = w.type
                ORDER BY s.name, w.name"#
            )
            .fetch_all(&self.pool).await?;

            Ok(rows.iter().map(|r| (
                r.id as i32,
                r.name.clone().unwrap_or_default(),
                r.skill_name.clone().unwrap_or_default(),
                r.range.clone().unwrap_or_default(),
            )).collect())
        })
    }
    pub fn get_weapon_by_id(&self, weapon_id: i32, character: &Character) -> anyhow::Result<Weapon> {
        self.block_on(async {
            let row = sqlx::query!(
                r#"SELECT
                    w.id AS weapon_id, w.name AS weapon_name,
                    w.dam, w.dtype, w.rate, w.range, w.wgt,
                    s.name AS skill_name
                FROM weapons w
                JOIN skills s ON s.id = w.type
                WHERE w.id = ?"#,
                weapon_id
            )
            .fetch_one(&self.pool).await?;

            // qualities
            let qual_rows = sqlx::query!(
                r#"SELECT q.name, wq.qual_val FROM weapon_quals wq
                JOIN qualities q ON q.id = wq.qual_id
                WHERE wq.weapon_id = ?"#, weapon_id
            ).fetch_all(&self.pool).await?;

            // effects
            let eff_rows = sqlx::query!(
                r#"SELECT de.name, we.effect_val FROM weapon_effects we
                JOIN dam_effects de ON de.id = we.effect_id
                WHERE we.weapon_id = ?"#, weapon_id
            ).fetch_all(&self.pool).await?;

            let damage_str = row.dam.clone().unwrap_or_default();
            let damage: i32 = damage_str.trim_end_matches(|c: char| c.is_alphabetic()).parse().unwrap_or(0);
            let rate = row.rate.unwrap_or_default() as i32;
            let range = row.range.clone().unwrap_or_default();
            let dam_type = parse_damage_type(&row.dtype.clone().unwrap_or_default());

            let skill_name = row.skill_name.clone().unwrap_or_default();
            let special: Vec<i32> = character.special.special_block().iter().map(|s| s.value).collect();
            let skills: Vec<i32>  = character.skills.skill_block().iter().map(|s| s.total).collect();
            let tags: Vec<bool>   = character.skills.skill_block().iter().map(|s| s.is_tagged()).collect();
            let (spec_index, skill_index, skill) = match skill_name.as_str() {
                "Melee Weapons"   => (0, 7,  Skill::MeleeWeapons),
                "Unarmed"         => (0, 16, Skill::Unarmed),
                "Small Guns"      => (5, 11, Skill::SmallGuns),
                "Throwing"        => (5, 15, Skill::Throwing),
                "Energy Weapons"  => (1, 3,  Skill::EnergyWeapons),
                "Explosives"      => (1, 4,  Skill::Explosives),
                "Big Guns"        => (2, 2,  Skill::BigGuns),
                _                 => (6, 0,  Skill::Athletics),
            };
            let target = skills[skill_index] + special[spec_index];
            let tag = tags[skill_index];

            let effects: Vec<String> = eff_rows.iter().map(|e| {
                let n = e.name.clone().unwrap_or_default();
                match e.effect_val { Some(v) if v != 0 => format!("{} {}", n, v), _ => n }
            }).collect();
            let qualities: Vec<String> = qual_rows.iter().map(|q| {
                let n = q.name.clone().unwrap_or_default();
                match q.qual_val { Some(v) if v != 0 => format!("{} {}", n, v), _ => n }
            }).collect();

            Ok(Weapon {
                id: row.weapon_id as i32,
                name: row.weapon_name.clone().unwrap_or_default(),
                prefix: String::new(),
                skill,
                target,
                tag,
                damage,
                effects,
                dam_type,
                rate,
                range,
                qualities,
                ammo: String::new(),
                wgt: row.wgt.unwrap_or(0) as i32,
                mods: vec![],
            })
        })
    }
    pub fn get_apparel_by_id(&self, id: i32) -> anyhow::Result<Apparel> {
        self.block_on(async {
            let row = sqlx::query!(
                r#"SELECT
                    id, name, type as atype, phys_dr, enrg_dr, rads_dr, wgt, eff
                FROM apparel a
                WHERE a.id = ?"#,
                id
            )
            .fetch_one(&self.pool).await?;
            let cover_list: Vec<i64> = sqlx::query!(
                    "SELECT ac.location_id AS cid
                    FROM apparel_covers ac
                    WHERE ac.apparel_id = ?",
                    row.id
                ).fetch_all(&self.pool).await
                .unwrap_or_default().iter().map(|c| c.cid.unwrap()).collect();
            Ok(Apparel {
                id: row.id as i32,
                name: row.name.clone().unwrap_or_default(),
                prefix: String::new(),
                apparel_type: resolve_apparel_type(row.atype.unwrap_or(0)),
                ph_dr: row.phys_dr.unwrap_or(0) as i32,
                en_dr: row.enrg_dr.unwrap_or(0) as i32,
                rd_dr: row.rads_dr.unwrap_or(0) as i32,
                wgt: row.wgt.unwrap_or(0) as i32,
                effects: serde_json::from_str(&row.eff.clone().unwrap_or_default()).ok().unwrap(),
                covers: resolve_apparel_covers(cover_list),
                equipped: false,
                db_id: 0,
            })
        })
    }
    pub fn get_all_apparel(&self) -> anyhow::Result<Vec<Apparel>> {
        self.block_on(async {
            let rows = sqlx::query!(
                r#"SELECT a.id, a.name, a.phys_dr AS ph_dr, a.enrg_dr AS en_dr,
                        a.rads_dr AS rd_dr, a.wgt, a.eff AS effs, a.type AS a_type
                FROM apparel a ORDER BY a.type, a.name"#
            ).fetch_all(&self.pool).await?;

            let mut result = vec![];
            for row in rows {
                let apparel_id = row.id as i32;
                let cover_list: Vec<i64> = sqlx::query!(
                    "SELECT ac.location_id AS cid FROM apparel_covers ac WHERE ac.apparel_id = ?",
                    apparel_id
                ).fetch_all(&self.pool).await
                .unwrap_or_default().iter().map(|c| c.cid.unwrap()).collect();

                result.push(Apparel {
                    id: apparel_id,
                    name: row.name.clone().unwrap_or_default(),
                    prefix: String::new(),
                    apparel_type: resolve_apparel_type(row.a_type.unwrap_or(0)),
                    ph_dr: row.ph_dr.unwrap_or(0) as i32,
                    en_dr: row.en_dr.unwrap_or(0) as i32,
                    rd_dr: row.rd_dr.unwrap_or(0) as i32,
                    wgt: row.wgt.unwrap_or(0) as i32,
                    effects: vec![row.effs.unwrap_or_default()],
                    covers: resolve_apparel_covers(cover_list),
                    equipped: false,
                    db_id: 0,
                });
            }
            Ok(result)
        })
    }

    pub fn get_all_ammo(&self) -> anyhow::Result<Vec<AmmoData>> {
        self.block_on(async {
            let rows = sqlx::query!(
                "SELECT id, name, wgt FROM ammo ORDER BY name"
            ).fetch_all(&self.pool).await?;
            Ok(rows.iter().map(|r| AmmoData {
                id: r.id as i32,
                name: r.name.clone().unwrap_or_default(),
                wgt: r.wgt.unwrap_or(0) as i32,
            }).collect())
        })
    }

    pub fn get_all_consumables(&self) -> anyhow::Result<Vec<Consumable>> {
        self.block_on(async {
            let rows = sqlx::query!(
                r#"SELECT c.id, c.name, c.type AS c_type, c.heals AS health,
                        c.eff AS effs, c.rads, c.wgt, c.duration, c.addiction
                FROM consumables c ORDER BY c.type, c.name"#
            ).fetch_all(&self.pool).await?;
            Ok(rows.iter().map(|r| Consumable {
                id: r.id as i32,
                name: r.name.clone().unwrap_or_default(),
                consumable_type: resolve_consumable_type(r.c_type.unwrap_or(0)),
                health: r.health.unwrap_or(0) as i32,
                effects: vec![r.effs.clone().unwrap_or_default()],
                rads: r.rads.unwrap_or(0) as i32,
                wgt: r.wgt.unwrap_or(0) as i32,
                duration: r.duration.clone().unwrap_or_default(),
                addiction: r.addiction.unwrap_or(0) as i32,
                quantity: 1,
            }).collect())
        })
    }

    pub fn get_all_robot_modules(&self) -> anyhow::Result<Vec<RobotModule>> {
        self.block_on(async {
            let rows = sqlx::query!(
                "SELECT id, name, eff AS effs, wgt FROM robot_modules ORDER BY name"
            ).fetch_all(&self.pool).await?;
            Ok(rows.iter().map(|r| RobotModule {
                id: r.id as i32,
                name: r.name.clone().unwrap_or_default(),
                installed: false,
                effect: vec![r.effs.clone().unwrap_or_default()],
                wgt: r.wgt.unwrap_or(0) as i32,
                db_id: 0,
            }).collect())
        })
    }

    pub fn get_all_gear(&self) -> anyhow::Result<Vec<Gear>> {
        self.block_on(async {
            let rows = sqlx::query!(
                "SELECT id, name, eff AS effs, wgt FROM gear ORDER BY name"
            ).fetch_all(&self.pool).await?;
            Ok(rows.iter().map(|r| Gear {
                id: r.id as i32,
                name: r.name.clone().unwrap_or_default(),
                effect: vec![r.effs.clone().unwrap_or_default()],
                wgt: r.wgt.unwrap_or(0) as i32,
                quantity: 1,
            }).collect())
        })
    }
    pub fn roll_ammo_core(&self) -> Vec<AmmoInv> {
        let roll_value = roll_d20(2) + 2;
        let mut result: Vec<AmmoInv> = vec![];
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT a.id, a.name, a.roll_quantity, a.wgt
                FROM core_ammo_loot cal
                JOIN ammo a ON cal.ammo_id = a.id
                WHERE cal.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => result = row.iter().map(|r| {
                let quantity = roll_cd(&r.roll_quantity.as_ref().unwrap_or(&"1".to_string()));
                AmmoInv {
                    quantity,
                    ammo: AmmoData {
                        id: r.id as i32,
                        name: r.name.clone().unwrap_or_default(),
                        wgt: r.wgt.unwrap_or(0) as i32,
                    }
                }
            }).collect(),
            Err(e) => eprintln!("error retrieving ammo loot roll: {e}"),
        }
        result
    }
    pub fn roll_armor_core(&self) -> Vec<Apparel> {
        //if you're calling this, you should be rolling against limbs as well to resolve what specific armor you found; this will return every part that particular set of armor covers, but you should only be getting one limb
        let roll_value = roll_d20(2) + 2;
        let mut result: Vec<Apparel> = vec![];
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT a.id, a.name, a.type as a_type, a.phys_dr, a.enrg_dr, a.rads_dr, a.wgt, a.eff
                FROM core_armor_loot cal
                JOIN apparel a ON cal.apparel_id = a.id
                WHERE cal.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => {
                result = row.iter().map(|r| {
                    let cover_list: Vec<i64> = self.block_on(async {
                        sqlx::query!(
                            r#"SELECT
                                ac.location_id AS cid
                            FROM apparel_covers ac
                            WHERE ac.apparel_id = ?
                            "#,
                            r.id
                        ).fetch_all(&self.pool).await
                    }).unwrap_or_default().iter().map(|c| c.cid.unwrap()).collect();
                    Apparel {
                        id: r.id as i32,
                        name: r.name.clone().unwrap_or_default(),
                        prefix: "".to_string(),
                        apparel_type: resolve_apparel_type(r.a_type.unwrap_or(0)),
                        ph_dr: r.phys_dr.unwrap_or(0) as i32,
                        en_dr: r.enrg_dr.unwrap_or(0) as i32,
                        rd_dr: r.rads_dr.unwrap_or(0) as i32,
                        wgt: r.wgt.unwrap_or(0) as i32,
                        effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                        covers: resolve_apparel_covers(cover_list),
                        equipped: false,
                        db_id: 0,
                    }
                }).collect()
            },
            Err(e) => eprintln!("error retrieving armor loot roll: {e}"),
        }
        result
    }
    pub fn roll_bevs_core(&self) -> Vec<Consumable> {
        let roll_value = roll_d20(2) + 2;
        let mut result: Vec<Consumable> = vec![];
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT c.id, c.name, c.heals, c.eff, c.rads, c.wgt, c.duration, c.addiction
                FROM core_beverage_loot cbl
                JOIN consumables c ON cbl.consumable_id = c.id
                WHERE cbl.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => result = row.iter().map(|r| {
                Consumable {
                    id: r.id as i32,
                    name: r.name.clone().unwrap_or_default(),
                    consumable_type: ConsumableType::Beverage,
                    health: r.heals.unwrap_or(0) as i32,
                    effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                    rads: r.rads.unwrap_or(0) as i32,
                    wgt: r.wgt.unwrap_or(0) as i32,
                    duration: r.duration.clone().unwrap_or("0".to_string()),
                    addiction: r.addiction.unwrap_or(0) as i32,
                    quantity: 1,
                }
            }).collect(),
            Err(e) => eprintln!("error retrieving beverage loot roll: {e}"),
        }
        result
    }
    pub fn roll_chem_core(&self) -> Vec<Consumable> {
        let roll_value = roll_d20(2) + 2;
        let mut result: Vec<Consumable> = vec![];
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT c.id, c.name, c.heals, c.eff, c.rads, c.wgt, c.duration, c.addiction
                FROM core_chem_loot ccl
                JOIN consumables c ON ccl.consumable_id = c.id
                WHERE ccl.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => result = row.iter().map(|r| {
                Consumable {
                    id: r.id as i32,
                    name: r.name.clone().unwrap_or_default(),
                    consumable_type: ConsumableType::Chem,
                    health: r.heals.unwrap_or(0) as i32,
                    effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                    rads: r.rads.unwrap_or(0) as i32,
                    wgt: r.wgt.unwrap_or(0) as i32,
                    duration: r.duration.clone().unwrap_or("0".to_string()),
                    addiction: r.addiction.unwrap_or(0) as i32,
                    quantity: 1,
                }
            }).collect(),
            Err(e) => eprintln!("error retrieving chem loot roll: {e}"),
        }
        result
    }
    pub fn roll_food_core(&self) -> Vec<Consumable> {
        let roll_value = roll_d20(2) + 2;
        let mut result: Vec<Consumable> = vec![];
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT c.id, c.name, c.heals, c.eff, c.rads, c.wgt, c.duration, c.addiction
                FROM core_food_loot cfl
                JOIN consumables c ON cfl.consumable_id = c.id
                WHERE cfl.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => result = row.iter().map(|r| {
                Consumable {
                    id: r.id as i32,
                    name: r.name.clone().unwrap_or_default(),
                    consumable_type: ConsumableType::Food,
                    health: r.heals.unwrap_or(0) as i32,
                    effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                    rads: r.rads.unwrap_or(0) as i32,
                    wgt: r.wgt.unwrap_or(0) as i32,
                    duration: r.duration.clone().unwrap_or("0".to_string()),
                    addiction: r.addiction.unwrap_or(0) as i32,
                    quantity: 1,
                }
            }).collect(),
            Err(e) => eprintln!("error retrieving food loot roll: {e}"),
        }
        result
    }
    pub fn roll_forage_core(&self) -> Vec<Consumable> {
        let roll_value = roll_d20(1) + 1;
        let mut result: Vec<Consumable> = vec![];
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT c.id, c.name, c.heals, c.eff, c.rads, c.wgt, c.duration, c.addiction
                FROM core_foraging_loot cfl
                JOIN consumables c ON cfl.consumable_id = c.id
                WHERE cfl.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => result = row.iter().map(|r| {
                Consumable {
                    id: r.id as i32,
                    name: r.name.clone().unwrap_or_default(),
                    consumable_type: ConsumableType::Food,
                    health: r.heals.unwrap_or(0) as i32,
                    effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                    rads: r.rads.unwrap_or(0) as i32,
                    wgt: r.wgt.unwrap_or(0) as i32,
                    duration: r.duration.clone().unwrap_or("0".to_string()),
                    addiction: r.addiction.unwrap_or(0) as i32,
                    quantity: 1,
                }
            }).collect(),
            Err(e) => eprintln!("error retrieving forage loot roll: {e}"),
        }
        result
    }
    pub fn roll_nuka_core(&self) -> (Vec<Consumable>,i32) {
        let roll_value = roll_d20(1) + 1;
        let mut result: Vec<Consumable> = vec![];
        match roll_value {
            9..13 => { return (result,roll_cd("1+2CD") * 2)},
            13..21 => {},
            _ => { return (result,0)},
        }
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT c.id, c.name, c.heals, c.eff, c.rads, c.wgt, c.duration, c.addiction
                FROM core_beverage_loot cbl
                JOIN consumables c ON cbl.consumable_id = c.id
                WHERE cbl.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => result = row.iter().map(|r| {
                Consumable {
                    id: r.id as i32,
                    name: r.name.clone().unwrap_or_default(),
                    consumable_type: ConsumableType::Beverage,
                    health: r.heals.unwrap_or(0) as i32,
                    effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                    rads: r.rads.unwrap_or(0) as i32,
                    wgt: r.wgt.unwrap_or(0) as i32,
                    duration: r.duration.clone().unwrap_or("0".to_string()),
                    addiction: r.addiction.unwrap_or(0) as i32,
                    quantity: 1,
                }
            }).collect(),
            Err(e) => eprintln!("error retrieving beverage loot roll: {e}"),
        }
        (result,0)
    }
    pub fn roll_pubs_core(&self) -> Vec<Consumable> {
        let roll_value = roll_d20(2) + 2;
        let mut result: Vec<Consumable> = vec![];
        let rows = self.block_on(async {
            sqlx::query!(
                "SELECT c.id, c.name, c.heals, c.eff, c.rads, c.wgt, c.duration, c.addiction
                FROM core_beverage_loot cbl
                JOIN consumables c ON cbl.consumable_id = c.id
                WHERE cbl.roll_value = ?
                ", roll_value
            ).fetch_all(&self.pool).await
        });
        match rows {
            Ok(row) => result = row.iter().map(|r| {
                Consumable {
                    id: r.id as i32,
                    name: r.name.clone().unwrap_or_default(),
                    consumable_type: ConsumableType::Publication,
                    health: r.heals.unwrap_or(0) as i32,
                    effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                    rads: r.rads.unwrap_or(0) as i32,
                    wgt: r.wgt.unwrap_or(0) as i32,
                    duration: r.duration.clone().unwrap_or("0".to_string()),
                    addiction: r.addiction.unwrap_or(0) as i32,
                    quantity: 1,
                }
            }).collect(),
            Err(e) => eprintln!("error retrieving beverage loot roll: {e}"),
        }
        result
    }
    pub fn roll_random_core(&self) -> (Vec<Consumable>,Vec<Gear>,Vec<String>,Vec<(bool,i32)>,Vec<RobotModule>) {
        let roll_value = roll_d20(3) + 3;
        let mut result = (vec![],vec![],vec![],vec![],vec![]);
        match roll_value {
            38 | 46 | 58 | 59 => {
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT c.id, c.name, c.heals, c.eff, c.rads, c.wgt, c.duration, c.addiction
                        FROM core_random_loot_consumables crlc
                        JOIN consumables c ON crlc.consumable_id = c.id
                        WHERE crlc.roll_value = ?
                        ", roll_value
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.0 = row.iter().map(|r| {
                        Consumable {
                            id: r.id as i32,
                            name: r.name.clone().unwrap_or_default(),
                            consumable_type: ConsumableType::Other,
                            health: r.heals.unwrap_or(0) as i32,
                            effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                            rads: r.rads.unwrap_or(0) as i32,
                            wgt: r.wgt.unwrap_or(0) as i32,
                            duration: r.duration.clone().unwrap_or("0".to_string()),
                            addiction: r.addiction.unwrap_or(0) as i32,
                            quantity: 1,
                        }
                    }).collect(),
                    Err(e) => eprintln!("error retrieving random consumable loot roll: {e}"),
                }
            },
            13..=14 | 20..=23 | 27 | 30 | 33..=35 | 37 | 39..=41 | 45 | 47..=48 | 55..=57 => {
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT g.id, g.name, g.eff, g.wgt, crlg.quantity
                        FROM core_random_loot_gear crlg
                        JOIN gear g ON crlg.gear_id = g.id
                        WHERE crlg.roll_value = ?
                        ", roll_value
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.1 = row.iter().map(|r| {
                        Gear {
                            id: r.id as i32,
                            name: r.name.clone().unwrap_or_default(),
                            effect: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                            wgt: r.wgt.unwrap_or(0) as i32,
                            quantity: if r.quantity.is_some() { roll_cd(&r.quantity.clone().unwrap()) } else { 1 },
                        }
                    }).collect(),
                    Err(e) => eprintln!("error retrieving random gear loot roll: {e}"),
                }
            },
            15 | 18 | 36 | 44 | 52..=54 => {
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT item
                        FROM core_random_loot_misc crlm
                        WHERE crlm.roll_value = ?
                        ", roll_value
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.2 = row.iter().map(|r| r.item.clone().unwrap_or("".to_string())).collect(),
                    Err(e) => eprintln!("error retrieving random misc loot roll: {e}"),
                }
            },
            5..=9 | 16..=17 | 24..=25 | 28..=29 | 31..=32 => {
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT prewar, d20s
                        FROM core_random_loot_money crlm
                        WHERE crlm.roll_value = ?
                        ", roll_value
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.3 = row.iter().map(|r| {
                        (r.prewar.unwrap_or(0) != 0, roll_d20(r.d20s.unwrap_or(0) as u32))
                    }).collect(),
                    Err(e) => eprintln!("error retrieving random misc money roll: {e}"),
                }

            },
            3..=4 | 10..=12 | 19 | 26 | 42..=43 | 49..=51 | 60 => {
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT m.id, m.name, m.eff, m.wgt
                        FROM core_random_loot_robot_modules crlr
                        JOIN robot_modules m ON crlr.robot_module_id = m.id
                        WHERE crlr.roll_value = ?
                        ", roll_value
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.4 = row.iter().map(|r| {
                        RobotModule {
                            id: r.id as i32,
                            name: r.name.clone().unwrap_or_default(),
                            effect: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                            wgt: r.wgt.unwrap_or(0) as i32,
                            installed: false,
                            db_id: 0,
                        }
                    }).collect(),
                    Err(e) => eprintln!("error retrieving random robot module loot roll: {e}"),
                }
            },
            _ => {}
        }
        result
    }
    pub fn roll_random_outcast(&self, character: &Character) -> (Vec<Gear>,Vec<Consumable>,Vec<Weapon>,Vec<Apparel>,String,Vec<RobotModule>) {
        let mut result = (vec![],vec![],vec![],vec![],"".to_string(),vec![]);
        let roll = roll_d20(1) + 1;
        match roll {
            1..=2 | 6 | 13 | 20 => {
                let gid = match roll {
                    1 => 4,
                    2 => 8,
                    6 => 17,
                    13 => 2,
                    20 => 16,
                    _ => 0,
                };
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT id, name, eff, wgt
                        FROM gear g
                        WHERE g.id = ?
                        ", gid
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.0 = row.iter().map(|r| {
                        Gear {
                            id: r.id as i32,
                            name: r.name.clone().unwrap_or_default(),
                            effect: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                            wgt: r.wgt.unwrap_or(0) as i32,
                            quantity: 1,
                        }
                    }).collect(),
                    Err(e) => eprintln!("error retrieving outcast gear roll: {e}"),
                }
            },
            3..=4 | 9 | 16 => {
                let cid = match roll {
                    3 => 233,
                    4 => 159,
                    9 => 185,
                    16 => 182,
                    _ => 0,
                };
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT id, name, heals, eff, rads, wgt, duration, addiction, type as ctype
                        FROM consumables c
                        WHERE c.id = ?
                        ", cid
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.1 = row.iter().map(|r| {
                        Consumable {
                            id: r.id as i32,
                            name: r.name.clone().unwrap_or_default(),
                            consumable_type: resolve_consumable_type(r.ctype.unwrap_or(0)),
                            health: r.heals.unwrap_or(0) as i32,
                            effects: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                            rads: r.rads.unwrap_or(0) as i32,
                            wgt: r.wgt.unwrap_or(0) as i32,
                            duration: r.duration.clone().unwrap_or("0".to_string()),
                            addiction: r.addiction.unwrap_or(0) as i32,
                            quantity: 1,
                        }
                    }).collect(),
                    Err(e) => eprintln!("error retrieving outcast consumable roll: {e}"),
                }
            },
            5 | 14..=15 | 17..=18 => {
                let wid = match roll {
                    5 => 66,
                    14 => 33,
                    15 => 17,
                    17 => 9,
                    18 => 56,
                    _ => 0
                };
                let weap = self.get_weapon_by_id(wid, character);
                match weap {
                    Ok(weapon) => result.2 = vec![weapon],
                    Err(e) => eprintln!("error retrieving outcast weapon roll: {e}"),
                }
            },
            7 | 10..=11 | 19 => {
                let aid = match roll {
                    7 => 90,
                    10 => 93,
                    11 => 91,
                    19 => 95,
                    _ => 0,
                };
                let app = self.get_apparel_by_id(aid);
                match app {
                    Ok(apparel) => result.3 = vec![apparel],
                    Err(e) => eprintln!("error retrieving outcast apparel roll: {e}"),
                }
            },
            8 => result.4 = "A map to an old world survivalist cache".to_string(),
            12 => {
                let rows = self.block_on(async {
                    sqlx::query!(
                        "SELECT id, name, eff, wgt
                        FROM robot_modules m
                        WHERE m.id = 11
                        "
                    ).fetch_all(&self.pool).await
                });
                match rows {
                    Ok(row) => result.5 = row.iter().map(|r| {
                        RobotModule {
                            id: r.id as i32,
                            name: r.name.clone().unwrap_or_default(),
                            effect: serde_json::from_str(&r.eff.clone().unwrap_or_default()).ok().unwrap(),
                            wgt: r.wgt.unwrap_or(0) as i32,
                            installed: false,
                            db_id: 0,
                        }
                    }).collect(),
                    Err(e) => eprintln!("error retrieving outcast robot module roll: {e}"),
                }
            }
            _ => {}
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct OriginRow {
    pub id: i32,
    pub name: String,
    pub sourcebook: String,
    pub description: String,
    pub can_ghoul: bool,
}

#[derive(Debug, Clone)]
pub struct TraitRow {
    pub id: i32,
    pub _origin_id: i32,
    pub name: String,
    pub description: String,
    pub is_ghoul_trait: bool,
}

pub fn load_origins(db: &Db) -> Vec<OriginRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT o.id, o.name, o.description, o.can_ghoul,
                s.name AS sourcebook
            FROM origins o
            JOIN sourcebooks s ON s.id = o.sourcebook_id
            ORDER BY s.id, o.name
            "#
        ).fetch_all(&db.pool).await
    });

    match result {
        Ok(rows) => rows.into_iter().map(|r| OriginRow {
            id: r.id as i32,
            name: r.name.unwrap_or_default(),
            sourcebook: r.sourcebook.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            can_ghoul: r.can_ghoul.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load origins: {e}"); vec![] }
    }
}

pub fn load_traits(db: &Db, origin_id: i32, state: &mut OriginState) -> Vec<TraitRow> {
    //let origin_id = origin.id as i64;
    let result =
        db.block_on(async {
            sqlx::query!(
                r#"
                SELECT t.id, ot.origin_id, t.name, t.description,
                    ot.is_ghoul_trait
                FROM origin_traits ot
                JOIN traits t ON t.id = ot.trait_id
                WHERE ot.origin_id = ?
                ORDER BY ot.is_ghoul_trait, t.name
                "#,
                origin_id
            ).fetch_all(&db.pool).await
        });
    
    state.origin_trait_count = result.iter().count().min(2) as i32;
    match result {
        Ok(rows) => rows.into_iter().map(|r| TraitRow {
            id: r.id as i32,
            _origin_id: r.origin_id.unwrap_or_default() as i32,
            name: r.name.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            is_ghoul_trait: r.is_ghoul_trait.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load traits: {e}"); vec![] }
    }
}

pub fn load_ghoul_traits(db: &Db, _state: &mut OriginState) -> Vec<TraitRow> {
    let result =
        db.block_on(async {
            sqlx::query!(
                r#"
                SELECT t.id, ot.origin_id, t.name, t.description,
                    ot.is_ghoul_trait
                FROM origin_traits ot
                JOIN traits t ON t.id = ot.trait_id
                WHERE ot.origin_id = 2
                ORDER BY ot.is_ghoul_trait, t.name
                "#,
            ).fetch_all(&db.pool).await
        });
    
    //state.origin_trait_count = 1;
    match result {
        Ok(rows) => rows.into_iter().map(|r| TraitRow {
            id: r.id as i32,
            _origin_id: r.origin_id.unwrap_or_default() as i32,
            name: r.name.unwrap_or_default(),
            description: r.description.unwrap_or_default(),
            is_ghoul_trait: r.is_ghoul_trait.unwrap_or(0) != 0,
        }).collect(),
        Err(e) => { eprintln!("Failed to load traits: {e}"); vec![] }
    }
}

#[derive(Debug, Clone)]
pub struct PerkRow {
    pub id: i32,
    pub name: String,
    pub description: Vec<String>,
    pub level_req: i32,
    pub ranks: i32,
    pub rank_range: i32,
    pub reqs: Vec<String>,
    pub limits: Vec<String>,
    pub sourcebook: String,
}

pub fn load_perks(db: &Db) -> Vec<PerkRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"
            SELECT p.id, p.name, p.description, p.ranks, p.rank_range, p.level_req, p.reqs, p.limits, s.name AS sourcebook
            FROM perks p
            JOIN sourcebooks s ON s.id = p.sourcebook_id
            ORDER BY s.id, p.name
            "#
        ).fetch_all(&db.pool).await
    });
    match result {
        Ok(rows) => rows.into_iter().map(|r| {
            let reqs: Vec<String> = r.reqs
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            let limits: Vec<String> = r.limits
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            PerkRow {
                id: r.id as i32,
                name: r.name.unwrap_or_default(),
                sourcebook: r.sourcebook.unwrap_or_default(),
                description: perk_description(r.description.unwrap_or_default()),
                level_req: r.level_req.unwrap_or_default() as i32,
                ranks: r.ranks.unwrap_or_default() as i32,
                rank_range: r.rank_range.unwrap_or_default() as i32,
                limits,
                reqs,
            }
        }).collect(),
        Err(e) => { eprintln!("Failed to load perks: {e}"); vec![] }
    }
}

//db structs
#[derive(Debug, Clone)]
pub struct BackgroundRow {
    pub id: i32,
    pub origin_id: i32,
    pub name: String,
    pub desc: String,
    pub caps: i32,
    pub misc: String,
    pub trinket: i32,
    pub food: i32,
    pub forage: i32,
    pub bev: i32,
    pub chem: i32,
    pub ammo: i32,
    pub aid: i32,
    pub odd: i32,
    pub outcast: i32,
    pub junk: i32,
}

#[derive(Debug, Clone)]
pub struct WeaponRow {
    pub id: i32,
    pub _bg_id: i32,
    pub weapon_id: i32,
    pub weapon_name: String,
    pub mod_id: Option<i32>,
    pub mod_name: Option<String>,
    pub alt_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ApparelRow {
    pub id: i32,
    pub _bg_id: i32,
    pub apparel_id: i32,
    pub apparel_name: String,
    pub alt_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ConsumableRow {
    pub id: i32,
    pub _bg_id: i32,
    pub consumable_id: i32,
    pub consumable_name: String,
    pub alt_id: Option<i32>,
    pub wgt: i32,
}

#[derive(Debug, Clone)]
pub struct RobotModuleRow {
    pub id: i32,
    pub _bg_id: i32,
    pub module_id: i32,
    pub module_name: String,
    pub alt_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct AmmoRow {
    pub ammo_id: i32,
    pub ammo_name: String,
    pub quantity: String,
    pub bg_weapon_id: i32,
}

#[derive(Debug, Clone)]
pub struct GearRow {
    pub gear_id: i32,
    pub gear_name: String,
    pub wgt: i32,
}

//loading all the background data from the db
pub fn load_backgrounds(db: &Db) -> Vec<BackgroundRow> {
    let result = db.block_on(async {
        sqlx::query!(
            r#"SELECT id, origin_id, name, description, caps, misc, trinket, food, forage, bev, chem, ammo, aid, odd, outcast, junk
               FROM backgrounds ORDER BY id"#
        )
        .fetch_all(&db.pool).await
    });
    match result {
        Ok(rows) => rows.into_iter().map(|r| BackgroundRow {
            id: r.id as i32,
            name: r.name.unwrap_or_default(),
            origin_id: r.origin_id.unwrap_or_default() as i32,
            desc: r.description.unwrap_or_default(),
            caps: r.caps.unwrap_or_default() as i32,
            misc: r.misc.unwrap_or_default(),
            trinket: r.trinket.unwrap_or_default() as i32,
            food: r.food.unwrap_or_default() as i32,
            forage: r.forage.unwrap_or_default() as i32,
            bev: r.bev.unwrap_or_default() as i32,
            chem: r.chem.unwrap_or_default() as i32,
            ammo: r.ammo.unwrap_or_default() as i32,
            aid: r.aid.unwrap_or_default() as i32,
            odd: r.odd.unwrap_or_default() as i32,
            outcast: r.outcast.unwrap_or_default() as i32,
            junk: r.junk.unwrap_or_default() as i32,
        }).collect(),
        Err(e) => { eprintln!("load_backgrounds: {e}"); vec![] }
    }
}
pub fn load_background_equipment(db: &Db, id: i32) -> ResolvedBackground {
    let background = load_backgrounds(db).into_iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| BackgroundRow {
            id,
            name: String::new(),
            desc: String::new(),
            origin_id: 0,
            caps: 0,
            misc: String::new(),
            trinket: 0,
            food: 0,
            forage: 0,
            bev: 0,
            chem: 0,
            ammo: 0,
            aid: 0,
            odd: 0,
            outcast: 0,
            junk: 0,
        });
    //weapons
    let weapon_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bw.id, bw.background_id, bw.weapon_id, bw.mod_id, bw.alt_id, w.name AS weapon_name, wm.name AS mod_name
               FROM background_weapons bw
               JOIN weapons w ON w.id = bw.weapon_id
               LEFT JOIN weapon_mods wm ON wm.id = bw.mod_id
               WHERE bw.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let weapons: Vec<WeaponRow> = weapon_rows.into_iter().map(|r| WeaponRow {
        id: r.id as i32,
        _bg_id: r.id as i32,
        weapon_id: r.weapon_id.unwrap_or_default() as i32,
        weapon_name: r.weapon_name.unwrap_or_default(),
        mod_id: r.mod_id.map(|i| i as i32),
        mod_name: r.mod_name,
        alt_id: r.alt_id.map(|i| i as i32),
    }).collect();
    //ammo
    let ammo_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT ba.ammo_id, ba.quantity, ba.bg_weapon_id, a.name AS ammo_name
               FROM background_ammo ba
               JOIN ammo a ON a.id = ba.ammo_id
               WHERE ba.bg_weapon_id IN (
                   SELECT id FROM background_weapons WHERE background_id = ?
               )"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let ammo: Vec<AmmoRow> = ammo_rows.into_iter().map(|r| AmmoRow {
        ammo_id: r.ammo_id.unwrap_or_default() as i32,
        ammo_name: r.ammo_name.unwrap_or_default(),
        quantity: r.quantity.unwrap_or_default(),
        bg_weapon_id: r.bg_weapon_id.unwrap_or_default() as i32,
    }).collect();
    //apparel
    let apparel_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT ba.id, ba.background_id, ba.apparel_id, ba.alt_id,
                      a.name AS apparel_name
               FROM background_apparel ba
               JOIN apparel a ON a.id = ba.apparel_id
               WHERE ba.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let apparel: Vec<ApparelRow> = apparel_rows.into_iter().map(|r| ApparelRow {
        id: r.id as i32,
        _bg_id: r.background_id.unwrap_or_default() as i32,
        apparel_id: r.apparel_id.unwrap_or_default() as i32,
        apparel_name: r.apparel_name.unwrap_or_default(),
        alt_id: r.alt_id.map(|i| i as i32),
    }).collect();
    //consumables
    let consumable_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bc.id, bc.background_id, bc.consumable_id, bc.alt_id, c.wgt, c.name AS consumable_name
               FROM background_consumables bc
               JOIN consumables c ON c.id = bc.consumable_id
               WHERE bc.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let consumables: Vec<ConsumableRow> = consumable_rows.into_iter().map(|r| ConsumableRow {
        id: r.id as i32,
        _bg_id: r.background_id.unwrap_or_default() as i32,
        consumable_id: r.consumable_id.unwrap_or_default() as i32,
        consumable_name: r.consumable_name.unwrap_or_default(),
        alt_id: r.alt_id.map(|i| i as i32),
        wgt: r.wgt.unwrap_or_default() as i32,
    }).collect();
    //robot mods
    let module_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT brm.id, brm.background_id, brm.robot_module_id, brm.alt_id, rm.name AS module_name
               FROM background_robot_modules brm
               JOIN robot_modules rm ON rm.id = brm.robot_module_id
               WHERE brm.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let robot_modules: Vec<RobotModuleRow> = module_rows.into_iter().map(|r| RobotModuleRow {
        id: r.id as i32,
        _bg_id: r.background_id.unwrap_or_default() as i32,
        module_id: r.robot_module_id.unwrap_or_default() as i32,
        module_name: r.module_name.unwrap_or_default(),
        alt_id: r.alt_id.map(|i| i as i32),
    }).collect();
    //gear - no choices for gear
    let gear_rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT bg.gear_id, g.name AS gear_name, g.wgt
               FROM background_gear bg
               JOIN gear g ON g.id = bg.gear_id
               WHERE bg.background_id = ?"#,
            id
        ).fetch_all(&db.pool).await
    }).unwrap_or_default();
    let gear: Vec<GearRow> = gear_rows.into_iter().map(|r| GearRow {
        gear_id: r.gear_id.unwrap_or_default() as i32,
        gear_name: r.gear_name.unwrap_or_default(),
        wgt: r.wgt.unwrap_or_default() as i32,
    }).collect();
    //include the miscellaneous stuff
    let misc = serde_json::from_str::<Vec<String>>(&background.misc)
        .unwrap_or_default()
        .join(", ");

    //put them all together
    ResolvedBackground {
        id: background.id,
        name: background.name,
        desc: background.desc,
        weapon_slots: resolve_weapon_slots(weapons),
        apparel_slots: resolve_apparel_slots(apparel),
        consumable_slots: resolve_consumable_slots(consumables),
        robot_module_slots: resolve_robot_module_slots(robot_modules),
        ammo,
        gear,
        caps: background.caps,
        misc,
        trinket: background.trinket,
        food: background.food,
        forage: background.forage,
        bev: background.bev,
        chem: background.chem,
        ammo_count: background.ammo,
        aid: background.aid,
        odd: background.odd,
        outcast: background.outcast,
        junk: background.junk,
    }
}


//used to handle applying mod effects properly
pub struct EffectNameSets {
    pub effect_names: HashSet<String>,
    pub quality_names: HashSet<String>,
}

impl EffectNameSets {
    pub fn load(db: &Db) -> Self {
        let effects = db.block_on(async {
            sqlx::query!("SELECT name FROM dam_effects").fetch_all(&db.pool).await
        }).unwrap_or_default();
        let qualities = db.block_on(async {
            sqlx::query!("SELECT name FROM qualities").fetch_all(&db.pool).await
        }).unwrap_or_default();
        //this needs to be adjusted to handle X effects/qualities
        Self {
            effect_names: effects.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
            quality_names: qualities.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
        }
    }
    pub async fn load_async(pool: &SqlitePool) -> Self {
        let effects = sqlx::query!("SELECT name FROM dam_effects").fetch_all(pool).await.unwrap_or_default();
        let qualities = sqlx::query!("SELECT name FROM qualities").fetch_all(pool).await.unwrap_or_default();
        //this needs to be adjusted to handle X effects/qualities
        Self {
            effect_names: effects.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
            quality_names: qualities.into_iter()
                .filter_map(|r| r.name)
                .map(|n| n.to_lowercase())
                .collect(),
        }
    }
    pub fn qual_not_eff(&self, name: &str) -> Option<bool> {
        let mut res = self.quality_names.contains(&name.to_lowercase());
        if res { return Some(res) } else {
            res = self.effect_names.contains(&name.to_lowercase());
            if res { return Some(!res) } else { None }
        }
    }
}


pub fn resolve_weapons(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
    character: &Character
) -> (Vec<Weapon>,Vec<AmmoInv>) {
    //grab all the weapon ids that were selected
    let selected_weapon_ids: Vec<i32> = background.weapon_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (WeaponSelSlot::Fixed(opt), _) => vec![opt.bg_weapon_id],
            (WeaponSelSlot::Choice(opts), SlotSelection::Chosen(i)) if *i < opts.len() => vec![opts[*i].bg_weapon_id],
            (WeaponSelSlot::ManyForOne(give_up,get_one ), SlotSelection::ManyForOneChosen(choice)) => if *choice == 0 {
                vec![get_one.bg_weapon_id]
            } else {
                give_up.iter().map(|w| w.bg_weapon_id).collect()
            },
            _ => vec![],
        })
        .collect();
    //if nothing is selected, send an empty vector
    if selected_weapon_ids.is_empty() { return (vec![],vec![]) }

    //grab the entire weapon's data for each selected weapon from the db
    let id_json = serde_json::to_string(&selected_weapon_ids).unwrap_or_default();
    let rows = db.block_on(async {
        sqlx::query!(
            r#"SELECT
                bw.id        AS bg_weapon_id,
                w.id         AS weapon_id,
                w.name       AS weapon_name,
                w.dam, w.dtype, w.rate, w.range, w.wgt,
                s.name       AS skill_name,
                a.name       AS ammo_name,
                a.wgt        AS ammo_wgt,
                a.id         AS ammo_id,
                ba.quantity  AS ammo_quantity,
                wm.id        AS mod_id,
                wm.name      AS mod_name,
                wm.prefix    AS mod_prefix,
                wm.effects   AS mod_effects,
                wm.wgt       AS mod_wgt,
                wm.slot      AS mod_slot
            FROM background_weapons bw
            JOIN weapons w   ON w.id  = bw.weapon_id
            JOIN skills  s   ON s.id  = w.type
            LEFT JOIN weapon_mods wm ON wm.id = bw.mod_id
            LEFT JOIN background_ammo ba ON ba.bg_weapon_id = bw.id
            LEFT JOIN ammo a ON a.id = ba.ammo_id
            WHERE bw.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();
    let mut w_result: Vec<Weapon> = vec![];
    let mut a_result: Vec<AmmoInv> = vec![];

    for row in &rows {
        let weapon_id = row.weapon_id;
        //grab qualities
        let qual_rows = db.block_on(async {
            sqlx::query!(
                r#"SELECT q.name, wq.qual_val
                   FROM weapon_quals wq
                   JOIN qualities q ON q.id = wq.qual_id
                   WHERE wq.weapon_id = ?"#,
                weapon_id
            ).fetch_all(&db.pool).await
        }).unwrap_or_default();
        let mut qualities: Vec<WeaponQuality> = qual_rows.iter().map(|q| WeaponQuality {
            name: q.name.clone().unwrap_or_default(),
            value: q.qual_val.map(|v| v as i32),
        }).collect();

        //grab effects
        let eff_rows = db.block_on(async {
            sqlx::query!(
                r#"SELECT de.name, we.effect_val
                   FROM weapon_effects we
                   JOIN dam_effects de ON de.id = we.effect_id
                   WHERE we.weapon_id = ?"#,
                weapon_id
            ).fetch_all(&db.pool).await
        }).unwrap_or_default();
        let mut effects: Vec<WeaponEffect> = eff_rows.iter().map(|e| WeaponEffect {
            name: e.name.clone().unwrap_or_default(),
            value: e.effect_val.map(|v| v as i32),
        }).collect();

        let damage_str = row.dam.clone().unwrap_or_default();
        let mut damage: i32 = damage_str.trim_end_matches(|c: char| c.is_alphabetic()).parse().unwrap_or(0);
        let mut rate = row.rate.unwrap_or_default() as i32;
        let mut range = row.range.clone().unwrap_or_default();
        let damage_type_str = row.dtype.clone().unwrap_or("".to_string());
        let mut dam_type = parse_damage_type(&damage_type_str);
        let base_wgt = row.wgt.unwrap_or_default() as i32;
        let mod_wgt = row.mod_wgt.unwrap_or_default() as i32;
        let weight = base_wgt + mod_wgt;
        let name = row.weapon_name.clone().unwrap_or_default();
        let prefix = row.mod_prefix.clone().unwrap_or_default();
        let ammo_name = row.ammo_name.clone().unwrap_or("".to_string());

        //target number calcs
        let skill_name = row.skill_name.clone().unwrap_or_default();
        let special: Vec<i32> = character.special.special_block().iter().map(|s| s.value.clone()).collect();
        let skills: Vec<i32>  = character.skills.skill_block().iter().map(|s| s.total.clone()).collect();
        let tags: Vec<bool> = character.skills.skill_block().iter().map(|s| s.is_tagged()).collect();
        let (spec_index, skill_index, skill) = match skill_name.as_str() {
            "Melee Weapons" => (0,7,Skill::MeleeWeapons),
            "Unarmed" => (0,16,Skill::Unarmed),
            "Small Guns" => (5,11,Skill::SmallGuns),
            "Throwing" => (5,15,Skill::Throwing),
            "Energy Weapons" => (1,3,Skill::EnergyWeapons),
            "Explosives" => (1,4,Skill::Explosives),
            "Big Guns" => (2,2,Skill::BigGuns),
            _  => (6,0,Skill::Athletics),
        };
        let spec_value = special[spec_index];
        let skill_total = skills[skill_index];
        let tag = tags[skill_index];
        let target = skill_total + spec_value;

        let name_set = load_mod_effect(db);
        let weapon_mod_eff = resolve_mod_effect(name_set,row.mod_effects.clone(), &mut damage, &mut rate, &mut range, &mut effects, &mut qualities, &mut dam_type);

        let mut weapon_mods: Vec<WeaponMods> = vec![];
        weapon_mods.push(WeaponMods {
            slot: resolve_weapon_slot(row.mod_slot.unwrap_or(0)),
            installed: true,
            id: row.mod_id.unwrap_or(0) as i32,
            name: row.mod_name.clone().unwrap_or("".to_string()),
            prefix: row.mod_prefix.clone().unwrap_or("".to_string()),
            wgt: row.mod_wgt.unwrap_or(0) as i32,
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
        });

        let weap_eff_str: Vec<String> = effects.iter().map(|e| if e.value != Some(0) && e.value.is_some() { format!("{} {}", e.name, e.value.unwrap()) } else { e.name.clone() }).collect();
        let weap_qual_str: Vec<String> = qualities.iter().map(|q| if q.value != Some(0) && q.value.is_some() { format!("{} {}", q.name, q.value.unwrap()) } else { q.name.clone() }).collect();

        w_result.push(Weapon {
            id: weapon_id.unwrap_or(0) as i32,
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
            mods: weapon_mods,
        });
        a_result.push(AmmoInv {
            ammo: AmmoData {
                id: row.ammo_id.unwrap_or(0) as i32,
                name: ammo_name.clone(),
                wgt: row.ammo_wgt.unwrap_or(0) as i32,
            },
            quantity: roll_cd(&row.ammo_quantity.clone().unwrap_or("".to_string()))
        })
    }
    (w_result,a_result)
}

pub fn load_mod_effect(db: &Db) -> EffectNameSets {
    EffectNameSets::load(db)
}

pub async fn load_mod_effect_async(pool: &SqlitePool) -> EffectNameSets {
    EffectNameSets::load_async(pool).await
}


pub fn resolve_apparel(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
) -> Vec<Apparel> {
    let mut result: Vec<Apparel> = vec![];
    let selected_apparel_ids: Vec<i32> = background.apparel_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (ApparelSelSlot::Fixed(opt), _) => vec![opt.bg_apparel_id],
            (ApparelSelSlot::Choice(opts), SlotSelection::Chosen(i)) => vec![opts[*i].bg_apparel_id],
            (ApparelSelSlot::SingleOrDouble(single, double_choices), SlotSelection::SingleOrDoubleChosen(take_single, double_picks)) => if *take_single {
                vec![single.bg_apparel_id]
            } else {
                vec![double_choices[0][double_picks[0].unwrap()].bg_apparel_id, double_choices[1][double_picks[1].unwrap()].bg_apparel_id]
            }
            (ApparelSelSlot::SingleOrPack(single, pack), SlotSelection::SingleOrPackChosen(choice)) => if *choice {
                vec![single.bg_apparel_id]
            } else {
                pack.iter().map(|a| a.bg_apparel_id).collect()
            },
            _ => vec![],
        })
        .collect();
    if selected_apparel_ids.is_empty() { return vec![] } 

    let id_json = serde_json::to_string(&selected_apparel_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                ba.id        AS bg_apparel_id,
                a.id         AS id,
                a.name       AS name,
                a.phys_dr    AS ph_dr,
                a.enrg_dr    AS en_dr,
                a.rads_dr    AS rd_dr,
                a.wgt        AS wgt,
                a.eff        AS effs,
                a.type       AS a_type
            FROM background_apparel ba
            JOIN apparel a   ON a.id  = ba.apparel_id
            WHERE ba.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        let apparel_id = row.id.unwrap() as i32;
        let cover_list: Vec<i64> = db.block_on(async {
            sqlx::query!(
                r#"SELECT
                    ac.location_id AS cid
                FROM apparel_covers ac
                WHERE ac.apparel_id = ?
                "#,
                apparel_id
            ).fetch_all(&db.pool).await
        }).unwrap_or_default().iter().map(|c| c.cid.unwrap()).collect();
        let covers = resolve_apparel_covers(cover_list);
        let effects = vec![row.effs.unwrap_or("".to_string())];

        result.push(Apparel {
            id: apparel_id,
            name: row.name.clone().unwrap_or("".to_string()),
            prefix: "".to_string(),
            apparel_type: resolve_apparel_type(row.a_type.unwrap_or(0)),
            ph_dr: row.ph_dr.unwrap_or(0) as i32,
            en_dr: row.en_dr.unwrap_or(0) as i32,
            rd_dr: row.rd_dr.unwrap_or(0) as i32,
            wgt: row.wgt.unwrap_or(0) as i32,
            effects,
            covers,
            equipped: false,
            db_id: 0,
        })
    }
    result
}


pub fn resolve_consumables(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
) -> Vec<Consumable> {
    let mut result: Vec<Consumable> = vec![];
    let selected_consumable_ids: Vec<i32> = background.consumable_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (ConsumableSelSlot::Fixed(opt), _) => vec![opt.bg_consumable_id],
            (ConsumableSelSlot::Choice(opts), SlotSelection::Chosen(i)) => vec![opts[*i].bg_consumable_id],
            (ConsumableSelSlot::ManyForOne(give_up,get_one ), SlotSelection::ManyForOneChosen(choice)) => if *choice == 0 {
                vec![get_one.bg_consumable_id]
            } else {
                give_up.iter().map(|c| c.bg_consumable_id).collect()
            },
            _ => vec![],
        })
        .collect();
    if selected_consumable_ids.is_empty() { return vec![] } 

    let id_json = serde_json::to_string(&selected_consumable_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                bc.id        AS bg_consumable_id,
                c.id         AS id,
                c.name       AS name,
                c.type       AS c_type, 
                c.heals      AS health,
                c.eff        AS effs,
                c.rads       AS rads,
                c.wgt        AS wgt,
                c.duration   AS duration,
                c.addiction  AS addiction
            FROM background_consumables bc
            JOIN consumables c ON c.id  = bc.consumable_id
            JOIN consumable_types ct ON ct.id  = c.type
            WHERE bc.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        if result.iter().any(|c| c.id == row.bg_consumable_id.unwrap_or(0) as i32) {
            let c_loc = result.iter().position(|c| c.id == row.bg_consumable_id.unwrap_or(0) as i32);
            result[c_loc.unwrap()].quantity += 1;
        } else {
            let addiction: i32 = row.addiction.unwrap_or(0) as i32;
            result.push(Consumable {
                id: row.id.unwrap_or(0) as i32,
                name: row.name.unwrap_or("".to_string()),
                consumable_type: resolve_consumable_type(row.c_type.unwrap_or(0)),
                health: row.health.unwrap_or(0) as i32,
                effects: vec![row.effs.unwrap_or("".to_string())],
                rads: row.rads.unwrap_or(0) as i32,
                wgt: row.wgt.unwrap_or(0) as i32,
                duration: row.duration.unwrap_or("".to_string()),
                addiction,
                quantity: 1,
            })
        }
    }
    result
}


pub fn resolve_robot_modules(
    db: &Db,
    background: &ResolvedBackground,
    selections: &[SlotSelection],
) -> Vec<RobotModule> {
    let mut result: Vec<RobotModule> = vec![];
    let selected_rmod_ids: Vec<i32> = background.robot_module_slots.iter()
        .zip(selections.iter())
        .flat_map(|(slot, sel)| match (slot, sel) {
            (RobotModuleSelSlot::Fixed(opt), _) => vec![opt.bg_module_id],
            (RobotModuleSelSlot::Choice(opts), SlotSelection::Chosen(i)) => vec![opts[*i].bg_module_id],
            _ => vec![],
        })
        .collect();
    if selected_rmod_ids.is_empty() { return vec![] } 

    let id_json = serde_json::to_string(&selected_rmod_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                br.id        AS bg_rmod_id,
                r.id         AS id,
                r.name       AS name,
                r.eff        AS effs,
                r.wgt        AS wgt,
                r.id         AS db_id
                FROM background_robot_modules br
            JOIN robot_modules r ON r.id  = br.robot_module_id
            WHERE br.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        result.push( RobotModule {
            id: row.id.unwrap_or(0) as i32,
            name: row.name.unwrap_or("".to_string()),
            installed: false,
            effect: vec![row.effs.unwrap_or("".to_string())],
            wgt: row.wgt.unwrap_or(0) as i32,
            db_id: row.db_id.unwrap_or(0),
        })
    }
    result
}

pub fn resolve_remaining_eq(
    db: &Db,
    background: &ResolvedBackground,
) -> (Vec<Gear>, Junk, Vec<String>) {
    let mut g_result: Vec<Gear> = vec![];

    let selected_gear_ids: Vec<i32> = background.gear.iter().map(|g| g.gear_id).collect();
    let id_json = serde_json::to_string(&selected_gear_ids).unwrap_or_default();
    let rows = db.block_on( async {
        sqlx::query!(
            r#"SELECT
                bg.id        AS bg_gear_id,
                g.id         AS id,
                g.name       AS name,
                g.eff        AS effs,
                g.wgt        AS wgt
            FROM background_gear bg
            JOIN gear g ON g.id  = bg.gear_id
            WHERE bg.id IN (
                SELECT value FROM json_each(?1)
            )"#,
            id_json
        )
        .fetch_all(&db.pool).await
    }).unwrap_or_default();

    for row in rows {
        if g_result.iter().any(|g| g.id == row.bg_gear_id.unwrap_or(0) as i32) {
            let g_loc = g_result.iter().position(|g| g.id == row.bg_gear_id.unwrap_or(0) as i32);
            g_result[g_loc.unwrap()].quantity += 1;
        } else {
            g_result.push(Gear {
                id: row.id.unwrap_or(0) as i32,
                name: row.name.unwrap_or("".to_string()),
                effect: vec![row.effs.unwrap_or("".to_string())],
                wgt: row.wgt.unwrap_or(0) as i32,
                quantity: 1,
            })
        }
    }
    let junk = Junk {
        common: roll_cd(&format!("{}CD",background.junk)),
        uncommon: 0,
        rare: 0,
    };
    (g_result, junk, vec![background.misc.clone()])
}