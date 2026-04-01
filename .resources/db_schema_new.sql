PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS apparel (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "type" INTEGER, -- smallint
  "dog" INTEGER, -- bool
  "phys_dr" INTEGER, -- smallint
  "enrg_dr" INTEGER, -- smallint
  "rads_dr" INTEGER, -- smallint
  "eff" TEXT, -- json array of strings
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
  "base_health" INTEGER, -- smallint
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id),
FOREIGN KEY (type) REFERENCES apparel_types (id)
);
CREATE TABLE IF NOT EXISTS character_apparel_mods (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_apparel_id" INTEGER, -- smallint
  "mod_id" INTEGER, -- smallint
FOREIGN KEY (character_apparel_id) REFERENCES character_apparel (id),
FOREIGN KEY (mod_id) REFERENCES apparel_mods (id)
);
CREATE TABLE IF NOT EXISTS consumables (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "type" INTEGER, -- smallint
  "heals" INTEGER, -- smallint
  "eff" TEXT, -- json array of strings
  "rads" INTEGER, -- smallint
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
  "duration" TEXT, -- single char
  "addiction" TEXT,
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id),
FOREIGN KEY (type) REFERENCES consumable_types (id)
);
CREATE TABLE IF NOT EXISTS settlements (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "party_id" TEXT, -- uuid
  "is_campsite" INTEGER, -- bool
  "npc_leader" INTEGER, -- smallint
  "people" INTEGER, -- smallint
  "food" INTEGER, -- smallint
  "water" INTEGER, -- smallint
  "power" INTEGER, -- smallint
  "defense" INTEGER, -- smallint
  "beds" INTEGER, -- smallint
  "happiness" INTEGER, -- smallint
  "income" INTEGER, -- smallint
  "stockpile" TEXT[],
FOREIGN KEY (party_id) REFERENCES parties (id),
FOREIGN KEY (npc_leader) REFERENCES active_npc_characters (id)
);
CREATE TABLE IF NOT EXISTS character_weapon_mods (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_weapon_id" INTEGER, -- smallint
  "mod_id" INTEGER, -- smallint
FOREIGN KEY (character_weapon_id) REFERENCES character_weapons (id),
FOREIGN KEY (mod_id) REFERENCES weapon_mods (id)
);
CREATE TABLE IF NOT EXISTS character_tags (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "athletics" INTEGER, -- bool
  "barter" INTEGER, -- bool
  "bigGuns" INTEGER, -- bool
  "energyWeapons" INTEGER, -- bool
  "explosives" INTEGER, -- bool
  "lockpick" INTEGER, -- bool
  "medicine" INTEGER, -- bool
  "meleeWeapons" INTEGER, -- bool
  "pilot" INTEGER, -- bool
  "repair" INTEGER, -- bool
  "science" INTEGER, -- bool
  "smallGuns" INTEGER, -- bool
  "sneak" INTEGER, -- bool
  "speech" INTEGER, -- bool
  "survival" INTEGER, -- bool
  "throwing" INTEGER, -- bool
  "unarmed" INTEGER, -- bool
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS factions (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "party_id" TEXT, -- uuid
  "party_reputation" INTEGER, -- smallint
  "faction_reputation" TEXT, -- json
FOREIGN KEY (party_id) REFERENCES parties (id)
);
CREATE TABLE IF NOT EXISTS weapon_slot_available (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "weapon_id" INTEGER, -- smallint
  "slot_id" INTEGER, -- smallint
FOREIGN KEY (weapon_id) REFERENCES weapons (id),
FOREIGN KEY (slot_id) REFERENCES weapon_slots (id)
);
CREATE TABLE IF NOT EXISTS apparel_slots (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT
);
CREATE TABLE IF NOT EXISTS consumable_types (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT
);
CREATE TABLE IF NOT EXISTS core_food_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS core_armor_loot (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "roll_value" INTEGER, -- smallint
  "apparel_id" INTEGER, -- smallint
FOREIGN KEY (apparel_id) REFERENCES apparel (id)
);
CREATE TABLE IF NOT EXISTS character_armor_legendary (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_apparel_id" INTEGER, -- smallint
  "legendary_id" INTEGER, -- smallint
FOREIGN KEY (character_apparel_id) REFERENCES character_apparel (id),
FOREIGN KEY (legendary_id) REFERENCES armor_legendary (id)
);
CREATE TABLE IF NOT EXISTS background_robot_modules (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "background_id" INTEGER, -- smallint
  "robot_module_id" INTEGER, -- smallint
  "alt_id" INTEGER, -- smallint
FOREIGN KEY (background_id) REFERENCES backgrounds (id),
FOREIGN KEY (robot_module_id) REFERENCES robot_modules (id),
FOREIGN KEY (alt_id) REFERENCES background_robot_modules (id)
);
CREATE TABLE IF NOT EXISTS extended_tests (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "party_id" TEXT, -- uuid
  "name" TEXT,
  "breakthroughs" INTEGER, -- smallint
FOREIGN KEY (party_id) REFERENCES parties (id)
);
CREATE TABLE IF NOT EXISTS core_random_loot_gear (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "gear_id" INTEGER, -- smallint
  "quantity" TEXT,
FOREIGN KEY (gear_id) REFERENCES gear (id)
);
CREATE TABLE IF NOT EXISTS weapon_recipe_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "weapon_id" INTEGER, -- smallint
  "skill_id" INTEGER, -- smallint
FOREIGN KEY (weapon_id) REFERENCES weapon_recipes (id),
FOREIGN KEY (skill_id) REFERENCES skills (id)
);
CREATE TABLE IF NOT EXISTS apparel_mod_available (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "apparel_id" INTEGER, -- smallint
  "mod_id" INTEGER, -- smallint
FOREIGN KEY (apparel_id) REFERENCES apparel (id),
FOREIGN KEY (mod_id) REFERENCES apparel_mods (id)
);
CREATE TABLE IF NOT EXISTS weapon_mods (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "prefix" TEXT,
  "effects" TEXT, -- json array of strings
  "slot" INTEGER, -- smallint
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
FOREIGN KEY (slot) REFERENCES weapon_slots (id)
);
CREATE TABLE IF NOT EXISTS robot_armor_recipe_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "robot_armor_id" INTEGER, -- smallint
  "skill_id" INTEGER, -- smallint
FOREIGN KEY (robot_armor_id) REFERENCES robot_armor_recipes (id),
FOREIGN KEY (skill_id) REFERENCES skills (id)
);
CREATE TABLE IF NOT EXISTS robot_module_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "robot_module" INTEGER, -- smallint
  "complexity" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
FOREIGN KEY (complexity) REFERENCES recipe_materials (complexity),
FOREIGN KEY (robot_module) REFERENCES robot_modules (id)
);
CREATE TABLE IF NOT EXISTS weapon_mod_available (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "weapon_id" INTEGER, -- smallint
  "mod_id" INTEGER, -- smallint
FOREIGN KEY (weapon_id) REFERENCES weapons (id),
FOREIGN KEY (mod_id) REFERENCES weapon_mods (id)
);
CREATE TABLE IF NOT EXISTS traits (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "description" TEXT
);
CREATE TABLE IF NOT EXISTS parties (
  "id" TEXT PRIMARY KEY, -- uuid
  "name" TEXT,
  "ap_players" INTEGER, -- smallint
  "ap_gm" INTEGER -- smallint
);
CREATE TABLE IF NOT EXISTS storefronts (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "party_id" TEXT, -- uuid
  "owning_npc" TEXT, -- uuid
  "caps" INTEGER, -- smallint
  "inventory" INTEGER, -- smallint
  "junk" TEXT, -- json
FOREIGN KEY (party_id) REFERENCES parties (id),
FOREIGN KEY (owning_npc) REFERENCES npc_characters (id),
FOREIGN KEY (inventory) REFERENCES store_inventory (id)
);
CREATE TABLE IF NOT EXISTS diseases (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "eff" TEXT,
  "duration" TEXT,
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id)
);
CREATE TABLE IF NOT EXISTS chem_recipe_perks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "chem_id" INTEGER, -- smallint
  "perk_id" INTEGER, -- smallint
  "rank" INTEGER, -- smallint
FOREIGN KEY (chem_id) REFERENCES chem_recipes (id),
FOREIGN KEY (perk_id) REFERENCES perks (id)
);
CREATE TABLE IF NOT EXISTS core_publications_loot (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "roll_value" INTEGER, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS character_powerarmor_frames (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "head" INTEGER, -- smallint
  "la" INTEGER, -- smallint
  "ra" INTEGER, -- smallint
  "torso" INTEGER, -- smallint
  "ll" INTEGER, -- smallint
  "rl" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
  "equipped" INTEGER, -- bool
  "location" TEXT,
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (head) REFERENCES character_powerarmor_pieces (id),
FOREIGN KEY (la) REFERENCES character_powerarmor_pieces (id),
FOREIGN KEY (ra) REFERENCES character_powerarmor_pieces (id),
FOREIGN KEY (torso) REFERENCES character_powerarmor_pieces (id),
FOREIGN KEY (ll) REFERENCES character_powerarmor_pieces (id),
FOREIGN KEY (rl) REFERENCES character_powerarmor_pieces (id)
);
CREATE TABLE IF NOT EXISTS apparel_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "apparel_mod" INTEGER, -- smallint
  "complexity" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
FOREIGN KEY (complexity) REFERENCES recipe_materials (complexity),
FOREIGN KEY (apparel_mod) REFERENCES apparel_mods (id)
);
CREATE TABLE IF NOT EXISTS character_ammo (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "ammo_id" INTEGER, -- smallint
  "quantity" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
FOREIGN KEY (ammo_id) REFERENCES ammo (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS chem_recipe_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "chem_id" INTEGER, -- smallint
  "skill_id" INTEGER, -- smallint
FOREIGN KEY (chem_id) REFERENCES chem_recipes (id),
FOREIGN KEY (skill_id) REFERENCES skills (id)
);
CREATE TABLE IF NOT EXISTS encounter_tables (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "type" TEXT,
  "die_roll" INTEGER, -- smallint
  "name" TEXT,
  "description" TEXT
);
CREATE TABLE IF NOT EXISTS sourcebooks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT
);
CREATE TABLE IF NOT EXISTS cook_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "consumable" INTEGER, -- smallint
  "junk_materials" TEXT, -- json
  "complexity" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
FOREIGN KEY (complexity) REFERENCES recipe_materials (complexity),
FOREIGN KEY (consumable) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS qualities (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "description" TEXT,
  "opposed_to" INTEGER, -- smallint
FOREIGN KEY (opposed_to) REFERENCES qualities (id)
);
CREATE TABLE IF NOT EXISTS core_random_loot_robot_modules (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "robot_module_id" INTEGER, -- smallint
FOREIGN KEY (robot_module_id) REFERENCES robot_modules (id)
);
CREATE TABLE IF NOT EXISTS chem_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "consumable" INTEGER, -- smallint
  "junk_materials" TEXT, -- json
  "complexity" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
FOREIGN KEY (complexity) REFERENCES recipe_materials (complexity),
FOREIGN KEY (consumable) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS active_npc_character_special (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "npc_character_id" TEXT, -- uuid
  "strength" INTEGER, -- smallint
  "perception" INTEGER, -- smallint
  "endurance" INTEGER, -- smallint
  "charisma" INTEGER, -- smallint
  "intelligence" INTEGER, -- smallint
  "agility" INTEGER, -- smallint
  "luck" INTEGER, -- smallint
FOREIGN KEY (npc_character_id) REFERENCES active_npc_characters (id)
);
CREATE TABLE IF NOT EXISTS settlement_npc_characters (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "settlement_id" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
FOREIGN KEY (settlement_id) REFERENCES settlements (id),
FOREIGN KEY (character_id) REFERENCES active_npc_characters (id)
);
CREATE TABLE IF NOT EXISTS core_clothing_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "apparel_id" INTEGER, -- smallint
FOREIGN KEY (apparel_id) REFERENCES apparel (id)
);
CREATE TABLE IF NOT EXISTS character_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "athletics" INTEGER, -- smallint
  "barter" INTEGER, -- smallint
  "bigGuns" INTEGER, -- smallint
  "energyWeapons" INTEGER, -- smallint
  "explosives" INTEGER, -- smallint
  "lockpick" INTEGER, -- smallint
  "medicine" INTEGER, -- smallint
  "meleeWeapons" INTEGER, -- smallint
  "pilot" INTEGER, -- smallint
  "repair" INTEGER, -- smallint
  "science" INTEGER, -- smallint
  "smallGuns" INTEGER, -- smallint
  "sneak" INTEGER, -- smallint
  "speech" INTEGER, -- smallint
  "survival" INTEGER, -- smallint
  "throwing" INTEGER, -- smallint
  "unarmed" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS character_weapon_legendary (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_weapon_id" INTEGER, -- smallint
  "legendary_id" INTEGER, -- smallint
FOREIGN KEY (character_weapon_id) REFERENCES character_weapons (id),
FOREIGN KEY (legendary_id) REFERENCES weapon_legendary (id)
);
CREATE TABLE IF NOT EXISTS character_gear (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "gear_id" INTEGER, -- smallint
  "quantity" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
FOREIGN KEY (gear_id) REFERENCES gear (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS character_addictions (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS character_powerarmor_piece_mods (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "piece_id" INTEGER, -- smallint
  "mod_id" INTEGER, -- smallint
FOREIGN KEY (piece_id) REFERENCES character_powerarmor_pieces (id),
FOREIGN KEY (mod_id) REFERENCES apparel_mods (id)
);
CREATE TABLE IF NOT EXISTS character_publications_read (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "publication_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (publication_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS cook_recipe_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "cook_id" INTEGER, -- smallint
  "skill_id" INTEGER, -- smallint
FOREIGN KEY (cook_id) REFERENCES cook_recipes (id),
FOREIGN KEY (skill_id) REFERENCES skills (id)
);
CREATE TABLE IF NOT EXISTS repair_materials (
  "rarity" INTEGER PRIMARY KEY, -- smallint
  "common" INTEGER, -- smallint
  "uncommon" INTEGER, -- smallint
  "rare" INTEGER -- smallint
);
CREATE TABLE IF NOT EXISTS chem_recipe_consumables (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "chem_id" INTEGER, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (chem_id) REFERENCES chem_recipes (id),
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS armor_legendary (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "eff" TEXT,
  "head_roll" INTEGER, -- smallint
  "arm_roll" INTEGER, -- smallint
  "torso_roll" INTEGER, -- smallint
  "leg_roll" INTEGER -- smallint
);
CREATE TABLE IF NOT EXISTS character_consumables (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "consumable_id" INTEGER, -- smallint
  "quantity" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
FOREIGN KEY (consumable_id) REFERENCES consumables (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS robot_armor_recipe_perks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "robot_armor_id" INTEGER, -- smallint
  "perk_id" INTEGER, -- smallint
  "rank" INTEGER, -- smallint
FOREIGN KEY (robot_armor_id) REFERENCES robot_armor_recipes (id),
FOREIGN KEY (perk_id) REFERENCES perks (id)
);
CREATE TABLE IF NOT EXISTS weapon_quals (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "weapon_id" INTEGER, -- smallint
  "qual_id" INTEGER, -- smallint
  "qual_val" INTEGER, -- smallint
FOREIGN KEY (weapon_id) REFERENCES weapons (id),
FOREIGN KEY (qual_id) REFERENCES qualities (id)
);
CREATE TABLE IF NOT EXISTS scavenging_locations (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT
);
CREATE TABLE IF NOT EXISTS core_random_loot_consumables (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS perks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "description" TEXT,
  "ranks" INTEGER, -- smallint
  "rank_range" INTEGER, -- smallint
  "level_req" INTEGER, -- smallint
  "reqs" TEXT, -- json array of strings
  "limits" TEXT, -- json array of strings
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id)
);
CREATE TABLE IF NOT EXISTS apparel_types (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT
);
CREATE TABLE IF NOT EXISTS core_random_loot_misc (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "item" TEXT
);
CREATE TABLE IF NOT EXISTS creature_types (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT
);
CREATE TABLE IF NOT EXISTS npc_character_special (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "npc_character_id" TEXT, -- uuid
  "strength" INTEGER, -- smallint
  "perception" INTEGER, -- smallint
  "endurance" INTEGER, -- smallint
  "charisma" INTEGER, -- smallint
  "intelligence" INTEGER, -- smallint
  "agility" INTEGER, -- smallint
  "luck" INTEGER, -- smallint
FOREIGN KEY (npc_character_id) REFERENCES npc_characters (id)
);
CREATE TABLE IF NOT EXISTS characters (
  "id" TEXT PRIMARY KEY, --uuid
  "player_id" TEXT, -- uuid
  "character_name" TEXT,
  "xp" INTEGER,
  "origin" INTEGER, -- smallint
  "luck_points" INTEGER, -- smallint
  "current_health" INTEGER, -- smallint
  "rad_points" INTEGER, -- smallint
  "head_hp" INTEGER, -- smallint
  "head_inj" INTEGER, -- smallint
  "la_hp" INTEGER, -- smallint
  "la_inj" INTEGER, -- smallint
  "ra_hp" INTEGER, -- smallint
  "ra_inj" INTEGER, -- smallint
  "torso_hp" INTEGER, -- smallint
  "torso_inj" INTEGER, -- smallint
  "ll_hp" INTEGER, -- smallint
  "ll_inj" INTEGER, -- smallint
  "rl_hp" INTEGER, -- smallint
  "rl_inj" INTEGER, -- smallint
  "caps" INTEGER, -- smallint
  "hunger" INTEGER, -- smallint
  "thirst" INTEGER, -- smallint
  "sleep" INTEGER, -- smallint
  "exposure" INTEGER, -- smallint
  "party_id" TEXT, -- uuid
FOREIGN KEY (player_id) REFERENCES players (id),
FOREIGN KEY (origin) REFERENCES origins (id),
FOREIGN KEY (party_id) REFERENCES parties (id)
);
CREATE TABLE IF NOT EXISTS skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "special" INTEGER, -- smallint
  "examples" TEXT,
  "description" TEXT,
FOREIGN KEY (special) REFERENCES special (id)
);
CREATE TABLE IF NOT EXISTS weapon_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "weapon_mod" INTEGER, -- smallint
  "complexity" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
FOREIGN KEY (complexity) REFERENCES recipe_materials (complexity),
FOREIGN KEY (weapon_mod) REFERENCES weapon_mods (id)
);
CREATE TABLE IF NOT EXISTS character_apparel (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "apparel_id" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
  "equipped" INTEGER, -- bool
  "legendary" INTEGER, -- bool
FOREIGN KEY (apparel_id) REFERENCES apparel (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS background_ammo (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "ammo_id" INTEGER, -- smallint
  "quantity" TEXT,
  "bg_weapon_id" INTEGER, -- smallint
FOREIGN KEY (ammo_id) REFERENCES ammo (id),
FOREIGN KEY (bg_weapon_id) REFERENCES background_weapons (id)
);
CREATE TABLE IF NOT EXISTS weapon_legendary (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "roll_table" INTEGER, -- smallint
  "name" TEXT,
  "eff" TEXT,
  "sg_roll" INTEGER, -- smallint
  "ew_roll" INTEGER, -- smallint
  "bg_roll" INTEGER, -- smallint
  "mw_roll" INTEGER, -- smallint
  "u_roll" INTEGER -- smallint
);
CREATE TABLE IF NOT EXISTS party_membership (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "party_id" TEXT, -- uuid
  "character_id" TEXT, -- uuid
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (party_id) REFERENCES parties (id)
);
CREATE TABLE IF NOT EXISTS gear (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "eff" TEXT, -- json array of strings
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id)
);
CREATE TABLE IF NOT EXISTS core_chem_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS core_random_loot_money (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "prewar" INTEGER, -- bool
  "d20s" INTEGER -- smallint
);
CREATE TABLE IF NOT EXISTS character_weapon_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "weapon_recipe_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (weapon_recipe_id) REFERENCES weapon_recipes (id)
);
CREATE TABLE IF NOT EXISTS store_inventory (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "source_table" TEXT,
  "source_id" INTEGER, -- smallint
  "quantity" INTEGER, -- smallint
  "other_data" TEXT
);
CREATE TABLE IF NOT EXISTS origin_traits (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "origin_id" INTEGER, -- smallint
  "trait_id" INTEGER, -- smallint
  "is_ghoul_trait" INTEGER, -- bool
FOREIGN KEY (origin_id) REFERENCES origins (id),
FOREIGN KEY (trait_id) REFERENCES traits (id)
);
CREATE TABLE IF NOT EXISTS npc_characters (
  "id" TEXT PRIMARY KEY, -- uuid
  "name" TEXT,
  "lvl" INTEGER, -- smallint
  "type" INTEGER, -- smallint
  "keywords" TEXT, -- json array of strings
  "rads_dr" INTEGER, -- smallint
  "poison_dr" INTEGER, -- smallint
  "head_hp" INTEGER, -- smallint
  "head_inj" INTEGER, -- smallint
  "head_phys_dr" INTEGER, -- smallint
  "head_enrg_dr" INTEGER, -- smallint
  "la_hp" INTEGER, -- smallint
  "la_inj" INTEGER, -- smallint
  "la_phys_dr" INTEGER, -- smallint
  "la_enrg_dr" INTEGER, -- smallint
  "ra_hp" INTEGER, -- smallint
  "ra_inj" INTEGER, -- smallint
  "ra_phys_dr" INTEGER, -- smallint
  "ra_enrg_dr" INTEGER, -- smallint
  "torso_hp" INTEGER, -- smallint
  "torso_inj" INTEGER, -- smallint
  "torso_phys_dr" INTEGER, -- smallint
  "torso_enrg_dr" INTEGER, -- smallint
  "ll_hp" INTEGER, -- smallint
  "ll_inj" INTEGER, -- smallint
  "ll_phys_dr" INTEGER, -- smallint
  "ll_enrg_dr" INTEGER, -- smallint
  "rl_hp" INTEGER, -- smallint
  "rl_inj" INTEGER, -- smallint
  "rl_phys_dr" INTEGER, -- smallint
  "rl_enrg_dr" INTEGER, -- smallint
  "attacks" TEXT, -- json array of strings
  "abilities" TEXT, -- json array of strings
  "caps" TEXT,
  "inventory" TEXT, -- json array of strings
FOREIGN KEY (type) REFERENCES character_types (id)
);
CREATE TABLE IF NOT EXISTS core_foraging_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS extended_test_characters (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "test_id" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
FOREIGN KEY (test_id) REFERENCES extended_tests (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS character_cook_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "cook_recipe_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (cook_recipe_id) REFERENCES cook_recipes (id)
);
CREATE TABLE IF NOT EXISTS core_melee_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "weapon_id" INTEGER, -- smallint
FOREIGN KEY (weapon_id) REFERENCES weapons (id)
);
CREATE TABLE IF NOT EXISTS wanderer_publications_loot (
  "roll_value" INTEGER, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS active_npc_creatures (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "creature" INTEGER, -- smallint
  "name" TEXT,
  "lvl" INTEGER, -- smallint
  "body" INTEGER, -- smallint
  "mind" INTEGER, -- smallint
  "melee" INTEGER, -- smallint
  "guns" INTEGER, -- smallint
  "other" INTEGER, -- smallint
  "defense" INTEGER, -- smallint
  "initiative" INTEGER, -- smallint
  "max_health" INTEGER, -- smallint
  "current_health" INTEGER, -- smallint
  "phys_dr" INTEGER, -- smallint
  "enrg_dr" INTEGER, -- smallint
  "rads_dr" INTEGER, -- smallint
  "pois_dr" INTEGER, -- smallint
  "head_hp" INTEGER, -- smallint
  "head_inj" INTEGER, -- smallint
  "la_hp" INTEGER, -- smallint
  "la_inj" INTEGER, -- smallint
  "ra_hp" INTEGER, -- smallint
  "ra_inj" INTEGER, -- smallint
  "torso_hp" INTEGER, -- smallint
  "torso_inj" INTEGER, -- smallint
  "ll_hp" INTEGER, -- smallint
  "ll_inj" INTEGER, -- smallint
  "rl_hp" INTEGER, -- smallint
  "rl_inj" INTEGER, -- smallint
  "attacks" TEXT, -- json array of strings
  "abilities" TEXT, -- json array of strings
  "inventory" TEXT, -- json array of strings
  "party_id" TEXT, -- uuid
  "notes" TEXT,
  "owning_character" INTEGER, -- smallint
FOREIGN KEY (creature) REFERENCES npc_creatures (id),
FOREIGN KEY (party_id) REFERENCES parties (id),
FOREIGN KEY (owning_character) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS character_apparel_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "apparel_recipe_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (apparel_recipe_id) REFERENCES apparel_recipes (id)
);
CREATE TABLE IF NOT EXISTS background_gear (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "background_id" INTEGER, -- smallint
  "gear_id" INTEGER, -- smallint
FOREIGN KEY (background_id) REFERENCES backgrounds (id),
FOREIGN KEY (gear_id) REFERENCES gear (id)
);
CREATE TABLE IF NOT EXISTS character_robot_modules (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "module_id" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
  "equipped" INTEGER, -- bool
FOREIGN KEY (module_id) REFERENCES robot_modules (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS weapon_effects (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "weapon_id" INTEGER, -- smallint
  "effect_id" INTEGER, -- smallint
  "effect_val" INTEGER, -- smallint
FOREIGN KEY (weapon_id) REFERENCES weapons (id),
FOREIGN KEY (effect_id) REFERENCES dam_effects (id)
);
CREATE TABLE IF NOT EXISTS special (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "description" TEXT
);
CREATE TABLE IF NOT EXISTS core_nuka_loot (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "roll_value" INTEGER, -- smallint
  "consumable_id" INTEGER, -- smallint
  "empties" TEXT,
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS robot_module_recipe_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "robot_module_id" INTEGER, -- smallint
  "skill_id" INTEGER, -- smallint
FOREIGN KEY (robot_module_id) REFERENCES robot_module_recipes (id),
FOREIGN KEY (skill_id) REFERENCES skills (id)
);
CREATE TABLE IF NOT EXISTS encounters (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "source_table" TEXT,
  "source_id" INTEGER, -- smallint
  "initiative" INTEGER, -- smallint
  "party_id" TEXT, -- uuid
FOREIGN KEY (party_id) REFERENCES parties (id)
);
CREATE TABLE IF NOT EXISTS npc_character_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "npc_character_id" TEXT, -- uuid
  "athletics" INTEGER, -- smallint
  "barter" INTEGER, -- smallint
  "bigGuns" INTEGER, -- smallint
  "energyWeapons" INTEGER, -- smallint
  "explosives" INTEGER, -- smallint
  "lockpick" INTEGER, -- smallint
  "medicine" INTEGER, -- smallint
  "meleeWeapons" INTEGER, -- smallint
  "pilot" INTEGER, -- smallint
  "repair" INTEGER, -- smallint
  "science" INTEGER, -- smallint
  "smallGuns" INTEGER, -- smallint
  "sneak" INTEGER, -- smallint
  "speech" INTEGER, -- smallint
  "survival" INTEGER, -- smallint
  "throwing" INTEGER, -- smallint
  "unarmed" INTEGER, -- smallint
FOREIGN KEY (npc_character_id) REFERENCES npc_characters (id)
);
CREATE TABLE IF NOT EXISTS ammo (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "roll_quantity" TEXT,
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id)
);
CREATE TABLE IF NOT EXISTS background_weapons (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "background_id" INTEGER, -- smallint
  "weapon_id" INTEGER, -- smallint
  "mod_id" INTEGER, -- smallint
  "alt_id" INTEGER, -- smallint
FOREIGN KEY (background_id) REFERENCES backgrounds (id),
FOREIGN KEY (weapon_id) REFERENCES weapons (id),
FOREIGN KEY (mod_id) REFERENCES weapon_mods (id),
FOREIGN KEY (alt_id) REFERENCES background_weapons (id)
);
CREATE TABLE IF NOT EXISTS recipe_materials (
  "complexity" INTEGER PRIMARY KEY, -- smallint
  "common" INTEGER, -- smallint
  "uncommon" INTEGER, -- smallint
  "rare" INTEGER -- smallint
);
CREATE TABLE IF NOT EXISTS character_perks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "perk_id" INTEGER, -- smallint
  "rank" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (perk_id) REFERENCES perks (id)
);
CREATE TABLE IF NOT EXISTS character_diseases (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "disease_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (disease_id) REFERENCES diseases (id)
);
CREATE TABLE IF NOT EXISTS character_types (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT
);
CREATE TABLE IF NOT EXISTS origins (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "description" TEXT,
  "can_ghoul" INTEGER, -- bool
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id)
);
CREATE TABLE IF NOT EXISTS core_thrown_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "weapon_id" INTEGER, -- smallint
  "quantity" TEXT,
FOREIGN KEY (weapon_id) REFERENCES weapons (id)
);
CREATE TABLE IF NOT EXISTS apparel_slot_available (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "apparel_id" INTEGER, -- smallint
  "slot_id" INTEGER, -- smallint
FOREIGN KEY (apparel_id) REFERENCES apparel (id),
FOREIGN KEY (slot_id) REFERENCES apparel_slots (id)
);
CREATE TABLE IF NOT EXISTS character_traits (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "trait_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (trait_id) REFERENCES traits (id)
);
CREATE TABLE IF NOT EXISTS active_npc_characters (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "name" TEXT,
  "lvl" INTEGER, -- smallint
  "current_health" INTEGER, -- smallint
  "rads_dr" INTEGER, -- smallint
  "poison_dr" INTEGER, -- smallint
  "head_hp" INTEGER, -- smallint
  "head_inj" INTEGER, -- smallint
  "head_phys_dr" INTEGER, -- smallint
  "head_enrg_dr" INTEGER, -- smallint
  "la_hp" INTEGER, -- smallint
  "la_inj" INTEGER, -- smallint
  "la_phys_dr" INTEGER, -- smallint
  "la_enrg_dr" INTEGER, -- smallint
  "ra_hp" INTEGER, -- smallint
  "ra_inj" INTEGER, -- smallint
  "ra_phys_dr" INTEGER, -- smallint
  "ra_enrg_dr" INTEGER, -- smallint
  "torso_hp" INTEGER, -- smallint
  "torso_inj" INTEGER, -- smallint
  "torso_phys_dr" INTEGER, -- smallint
  "torso_enrg_dr" INTEGER, -- smallint
  "ll_hp" INTEGER, -- smallint
  "ll_inj" INTEGER, -- smallint
  "ll_phys_dr" INTEGER, -- smallint
  "ll_enrg_dr" INTEGER, -- smallint
  "rl_hp" INTEGER, -- smallint
  "rl_inj" INTEGER, -- smallint
  "rl_phys_dr" INTEGER, -- smallint
  "rl_enrg_dr" INTEGER, -- smallint
  "attacks" TEXT, -- json array of strings
  "abilities" TEXT, -- json array of strings
  "caps" TEXT,
  "inventory" TEXT, -- json array of strings
  "party_id" TEXT, -- uuid
  "notes" TEXT,
  "owning_character" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES npc_characters (id),
FOREIGN KEY (party_id) REFERENCES parties (id),
FOREIGN KEY (owning_character) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS character_robot_armor_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "robot_armor_recipe_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (robot_armor_recipe_id) REFERENCES robot_armor_recipes (id)
);
CREATE TABLE IF NOT EXISTS backgrounds (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "origin_id" INTEGER, -- smallint
  "caps" INTEGER, -- smallint
  "misc" TEXT,
  "trinket" INTEGER, -- smallint
  "food" INTEGER, -- smallint
  "forage" INTEGER, -- smallint
  "bev" INTEGER, -- smallint
  "chem" INTEGER, -- smallint
  "ammo" INTEGER, -- smallint
  "aid" INTEGER, -- smallint
  "odd" INTEGER, -- smallint
  "outcast" INTEGER, -- smallint
  "junk" INTEGER, -- smallint
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (origin_id) REFERENCES origins (id),
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id)
);
CREATE TABLE IF NOT EXISTS powerarmor_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "apparel_mod" INTEGER, -- smallint
  "complexity" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
FOREIGN KEY (complexity) REFERENCES recipe_materials (complexity),
FOREIGN KEY (apparel_mod) REFERENCES apparel_mods (id)
);
CREATE TABLE IF NOT EXISTS character_weapons (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "weapon_id" INTEGER, -- smallint
  "character_id" TEXT, -- uuid
FOREIGN KEY (weapon_id) REFERENCES weapons (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS background_apparel (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "background_id" INTEGER, -- smallint
  "apparel_id" INTEGER, -- smallint
  "alt_id" INTEGER, -- smallint
FOREIGN KEY (background_id) REFERENCES backgrounds (id),
FOREIGN KEY (apparel_id) REFERENCES apparel (id),
FOREIGN KEY (alt_id) REFERENCES background_apparel (id)
);
CREATE TABLE IF NOT EXISTS core_ammo_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "ammo_id" INTEGER, -- smallint
  "other" TEXT,
FOREIGN KEY (ammo_id) REFERENCES ammo (id)
);
CREATE TABLE IF NOT EXISTS core_ranged_loot (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "roll_value" INTEGER, -- smallint
  "weapon_id" INTEGER, -- smallint
  "mod_id" INTEGER, -- smallint
FOREIGN KEY (weapon_id) REFERENCES weapons (id),
FOREIGN KEY (mod_id) REFERENCES weapon_mods (id)
);
CREATE TABLE IF NOT EXISTS players (
  "id" TEXT PRIMARY KEY, -- uuid
  "username" TEXT,
  "auth" TEXT
);
CREATE TABLE IF NOT EXISTS robot_module_perks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "robot_module_id" INTEGER, -- smallint
  "perk_id" INTEGER, -- smallint
  "rank" INTEGER, -- smallint
FOREIGN KEY (robot_module_id) REFERENCES robot_modules (id),
FOREIGN KEY (perk_id) REFERENCES perks (id)
);
CREATE TABLE IF NOT EXISTS character_special (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "strength" INTEGER, -- smallint
  "perception" INTEGER, -- smallint
  "endurance" INTEGER, -- smallint
  "charisma" INTEGER, -- smallint
  "intelligence" INTEGER, -- smallint
  "agility" INTEGER, -- smallint
  "luck" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS character_chem_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "chem_recipe_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (chem_recipe_id) REFERENCES chem_recipes (id)
);
CREATE TABLE IF NOT EXISTS ammo_variants (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "variant" INTEGER, -- smallint
  "base" INTEGER, -- smallint
FOREIGN KEY (variant) REFERENCES ammo (id),
FOREIGN KEY (base) REFERENCES ammo (id)
);
CREATE TABLE IF NOT EXISTS active_npc_character_skills (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "npc_character_id" TEXT, -- uuid
  "athletics" INTEGER, -- smallint
  "barter" INTEGER, -- smallint
  "bigGuns" INTEGER, -- smallint
  "energyWeapons" INTEGER, -- smallint
  "explosives" INTEGER, -- smallint
  "lockpick" INTEGER, -- smallint
  "medicine" INTEGER, -- smallint
  "meleeWeapons" INTEGER, -- smallint
  "pilot" INTEGER, -- smallint
  "repair" INTEGER, -- smallint
  "science" INTEGER, -- smallint
  "smallGuns" INTEGER, -- smallint
  "sneak" INTEGER, -- smallint
  "speech" INTEGER, -- smallint
  "survival" INTEGER, -- smallint
  "throwing" INTEGER, -- smallint
  "unarmed" INTEGER, -- smallint
FOREIGN KEY (npc_character_id) REFERENCES active_npc_characters (id)
);
CREATE TABLE IF NOT EXISTS character_powerarmor_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "powerarmor_recipe_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (powerarmor_recipe_id) REFERENCES powerarmor_recipes (id)
);
CREATE TABLE IF NOT EXISTS cook_recipe_consumables (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "cook_id" INTEGER, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (cook_id) REFERENCES cook_recipes (id),
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
CREATE TABLE IF NOT EXISTS settlement_npc_creatures (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "settlement_id" INTEGER, -- smallint
  "creature_id" INTEGER, -- smallint
FOREIGN KEY (settlement_id) REFERENCES settlements (id),
FOREIGN KEY (creature_id) REFERENCES active_npc_creatures (id)
);
CREATE TABLE IF NOT EXISTS apparel_mods (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "slot" INTEGER, -- smallint
  "phys_dr" INTEGER, -- smallint
  "enrg_dr" INTEGER, -- smallint
  "rads_dr" INTEGER, -- smallint
  "health" INTEGER, -- smallint
  "effects" TEXT, -- json array of strings
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
  "skill" INTEGER, -- smallint
FOREIGN KEY (slot) REFERENCES apparel_slots (id),
FOREIGN KEY (skill) REFERENCES skills (id)
);
CREATE TABLE IF NOT EXISTS weapon_mod_perks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "mod_id" INTEGER, -- smallint
  "perk_id" INTEGER, -- smallint
  "rank" INTEGER, -- smallint
FOREIGN KEY (mod_id) REFERENCES weapon_mods (id),
FOREIGN KEY (perk_id) REFERENCES perks (id)
);
CREATE TABLE IF NOT EXISTS npc_creatures (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "lvl" INTEGER, -- smallint
  "type" INTEGER, -- smallint
  "keywords" TEXT, -- json array of strings
  "body" INTEGER, -- smallint
  "mind" INTEGER, -- smallint
  "melee" INTEGER, -- smallint
  "guns" INTEGER, -- smallint
  "other" INTEGER, -- smallint
  "defense" INTEGER, -- smallint
  "initiative" INTEGER, -- smallint
  "health" INTEGER, -- smallint
  "phys_dr" INTEGER, -- smallint
  "enrg_dr" INTEGER, -- smallint
  "rads_dr" INTEGER, -- smallint
  "pois_dr" INTEGER, -- smallint
  "attacks" TEXT, -- json array of strings
  "abilities" TEXT, -- json array of strings
  "butcher" TEXT,
  "salvage" TEXT,
  "caps" TEXT,
  "junk" TEXT,
  "inventory" TEXT, -- json array of strings
FOREIGN KEY (type) REFERENCES creature_types (id)
);
CREATE TABLE IF NOT EXISTS robot_armor_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "apparel" INTEGER, -- smallint
  "complexity" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
FOREIGN KEY (complexity) REFERENCES recipe_materials (complexity),
FOREIGN KEY (apparel) REFERENCES apparel (id)
);
CREATE TABLE IF NOT EXISTS body_locations (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "alternate_names" TEXT -- json array of strings
);
CREATE TABLE IF NOT EXISTS character_robot_modules_recipes (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "character_id" TEXT, -- uuid
  "robot_modules_recipe_id" INTEGER, -- smallint
FOREIGN KEY (character_id) REFERENCES characters (id),
FOREIGN KEY (robot_modules_recipe_id) REFERENCES robot_module_recipes (id)
);
CREATE TABLE IF NOT EXISTS character_powerarmor_pieces (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "piece_id" INTEGER, -- smallint
  "mods_applied" TEXT, -- json array of integers
  "character_id" TEXT, -- uuid
FOREIGN KEY (piece_id) REFERENCES apparel (id),
FOREIGN KEY (character_id) REFERENCES characters (id)
);
CREATE TABLE IF NOT EXISTS background_consumables (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "background_id" INTEGER, -- smallint
  "consumable_id" INTEGER, -- smallint
  "alt_id" INTEGER, -- smallint
FOREIGN KEY (background_id) REFERENCES backgrounds (id),
FOREIGN KEY (consumable_id) REFERENCES consumables (id),
FOREIGN KEY (alt_id) REFERENCES background_consumables (id)
);
CREATE TABLE IF NOT EXISTS weapon_slots (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT
);
CREATE TABLE IF NOT EXISTS apparel_mod_perks (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "mod_id" INTEGER, -- smallint
  "perk_id" INTEGER, -- smallint
  "rank" INTEGER, -- smallint
FOREIGN KEY (mod_id) REFERENCES apparel_mods (id),
FOREIGN KEY (perk_id) REFERENCES perks (id)
);
CREATE TABLE IF NOT EXISTS dam_effects (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "description" TEXT
);
CREATE TABLE IF NOT EXISTS robot_modules (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "eff" TEXT, -- json array of strings
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id)
);
CREATE TABLE IF NOT EXISTS weapons (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "name" TEXT,
  "type" INTEGER, -- smallint
  "dam" TEXT,
  "dtype" TEXT,
  "rate" INTEGER, -- smallint
  "range" TEXT, -- single char
  "wgt" INTEGER, -- smallint
  "cost" INTEGER, -- smallint
  "rarity" INTEGER, -- smallint
  "ammo" INTEGER, -- smallint
  "sourcebook_id" INTEGER, -- smallint
FOREIGN KEY (sourcebook_id) REFERENCES sourcebooks (id),
FOREIGN KEY (ammo) REFERENCES ammo (id),
FOREIGN KEY (type) REFERENCES skills (id)
);
CREATE TABLE IF NOT EXISTS apparel_covers (
  "id" INTEGER PRIMARY KEY AUTOINCREMENT,
  "apparel_id" INTEGER, -- smallint
  "location_id" INTEGER, -- smallint
FOREIGN KEY (apparel_id) REFERENCES apparel (id),
FOREIGN KEY (location_id) REFERENCES body_locations (id)
);
CREATE TABLE IF NOT EXISTS core_beverage_loot (
  "roll_value" INTEGER PRIMARY KEY, -- smallint
  "consumable_id" INTEGER, -- smallint
FOREIGN KEY (consumable_id) REFERENCES consumables (id)
);
