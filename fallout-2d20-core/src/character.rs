use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::{
    get_staggered_bonus,
    states::SpecialState, structs::Version,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Character {
    pub id: Uuid,
    pub version: Version,
    pub name: String,
    pub player: Player,
    pub party: Party,
    pub level: i32,
    pub xp: i32,
    pub xp_next: i32,
    pub origin: Option<Origin>,
    pub background: Option<Background>,
    pub traits: Vec<Trait>,
    pub ghoul: bool,
    pub mutant: MutantType,
    pub robot: RobotType,
    pub companion: CompanionType,
    pub robot_hat: Option<Apparel>,
    pub special: Special,
    pub luck_points: i32,
    pub luck_points_max: i32,
    pub rad_points: i32,
    pub skills: Skills,
    pub perks: Vec<Perk>,
    pub flagged_perks: Vec<i32>,
    pub melee_mod: MeleeModifiers,
    pub defense: i32,
    pub initiative: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub base_dr: BaseDR,
    pub poison_dr: i32,
    pub limb_dr: Limbs,
    pub caps: i32,
    pub weapons: Vec<Weapon>,
    pub ammo: Vec<AmmoInv>,
    pub apparel: Vec<Apparel>,
    pub robot_modules: Vec<RobotModule>,
    pub consumables: Vec<Consumable>,
    pub gear: Vec<Gear>,
    pub junk: Junk,
    pub misc: Vec<String>,
    pub carry_wgt: i32,
    pub carry_wgt_max: i32,
    pub notes: String,
}

impl Character {
    pub fn new(player: Player, party: Party, version: Version) -> Self {
        Self {
            id: (Uuid::now_v7()),
            version,
            name: String::new(),
            player: player,
            party: party,
            level: 1,
            xp: 0,
            xp_next: 100,
            origin: None,
            background: None,
            traits: vec![],
            ghoul: false,
            mutant: MutantType::None,
            robot: RobotType::None,
            companion: CompanionType::None,
            robot_hat: None,
            special: Special::new(),
            luck_points: 5,
            luck_points_max: 5,
            rad_points: 0,
            skills: Skills::new(),
            perks: vec![],
            flagged_perks: vec![],
            melee_mod: MeleeModifiers::new(),
            defense: 0,
            initiative: 10,
            hp: 10,
            hp_max: 10,
            base_dr: BaseDR::new(),
            poison_dr: 0,
            limb_dr: Limbs::new(),
            caps: 0,
            weapons: vec![],
            ammo: vec![],
            apparel: vec![],
            robot_modules: vec![],
            consumables: vec![],
            gear: vec![],
            junk: Junk::new(),
            misc: vec![],
            carry_wgt: 0,
            carry_wgt_max: 200,
            notes: String::new(),
        }
    }
    pub fn reset(&mut self){
        self.id = Uuid::now_v7();
        self.name = String::new();
        self.level = 1;
        self.xp = 0;
        self.xp_next = 100;
        self.origin = None;
        self.background = None;
        self.traits = vec![];
        self.ghoul = false;
        self.mutant = MutantType::None;
        self.robot = RobotType::None;
        self.companion = CompanionType::None;
        self.robot_hat = None;
        self.special = Special::new();
        self.luck_points = 5;
        self.luck_points_max = 5;
        self.rad_points = 0;
        self.skills = Skills::new();
        self.perks = vec![];
        self.flagged_perks = vec![];
        self.melee_mod = MeleeModifiers::new();
        self.defense = 0;
        self.initiative = 10;
        self.hp = 10;
        self.hp_max = 10;
        self.base_dr = BaseDR::new();
        self.poison_dr = 0;
        self.limb_dr = Limbs::new();
        self.caps = 0;
        self.weapons = vec![];
        self.ammo = vec![];
        self.apparel = vec![];
        self.robot_modules = vec![];
        self.consumables = vec![];
        self.gear = vec![];
        self.junk = Junk::new();
        self.misc = vec![];
        self.carry_wgt = 0;
        self.carry_wgt_max = 200;
        self.notes = String::new();
    }
    pub fn is_gifted(&self) -> bool {
        self.has_trait(7)
    }
    pub fn is_mutant(&self) -> bool {
        self.mutant != MutantType::None
    }
    pub fn is_robot(&self) -> bool {
        self.robot != RobotType::None
    }
    pub fn total_skill(&self) -> i32 {
        self.skills.skill_block().iter().map(|s| s.total).sum()
    }
    pub fn total_skill_ranks(&self) -> i32 {
        self.skills.skill_block().iter().map(|s| s.ranks).sum()
    }
    pub fn has_trait(&self, id: i32) -> bool {
        self.traits.iter().any(|t| t.id == id)
    }
    pub fn has_any_trait(&self, id: Vec<i32>) -> bool {
        self.traits.iter().any(|t| id.contains(&t.id))
    }
    pub fn has_perk(&self, id: i32) -> bool {
        self.perks.iter().any(|p| p.id == id)
    }
    pub fn perk_ranks(&self, id: i32) -> i32 {
        self.perks.iter().find(|p| p.id == id).map(|p| p.ranks).unwrap_or(0)
    }
    pub fn update_type(&mut self) {
        if self.origin.is_some() {
            //set mutant status
            self.mutant = match self.origin.clone().unwrap().id {
                3 => MutantType::SuperMutant,
                16 => MutantType::Nightkin,
                _ => MutantType::None,
            };
            //set robot status
            self.robot = match self.origin.clone().unwrap().id {
                4 => RobotType::Handy,
                9 => RobotType::Protectron,
                10 => RobotType::Robobrain,
                11 => RobotType::Securitron,
                12 => RobotType::Synth,
                14 => RobotType::Assaultron,
                _ => RobotType::None,
            };
        }
    }
    pub fn calculate_xp(&mut self) {
        self.xp = self.level * (self.level - 1) * 50;
    }
    pub fn calculate_level(&mut self) {
        self.level = (0.5 + ((25.0 + (2.0 * self.xp as f32)).sqrt() / 10.0)).floor() as i32;
    }
    pub fn calculate_xp_next(&mut self) {
        self.xp_next = (self.level + 1) * self.level * 50 - self.xp;
    }
    pub fn calculate_carry_weight(&mut self) {
        let strong_back = (self.perk_ranks(91)) * 25;
        self.carry_wgt_max = if self.has_any_trait(vec![4,19,20,23]) {
            150
        } else if self.has_trait(18) {
            225
        } else if self.has_trait(9) {
            150 + (5 * self.special.strength.value) + strong_back
        } else {
            150 + (10 * self.special.strength.value) + strong_back
        };
        let mut total_weight = 0;
        for w in self.weapons.clone() {
            total_weight += w.wgt;
        }
        for a in self.ammo.clone() {
            total_weight += a.ammo.wgt * a.quantity;
        }
        for a in self.apparel.clone() {
            total_weight += a.wgt;
        }
        for c in self.consumables.clone() {
            total_weight += c.wgt * c.quantity;
        }
        for m in self.robot_modules.clone() {
            total_weight += m.wgt;
        }
        for g in self.gear.clone() {
            total_weight += g.wgt * g.quantity;
        }
        total_weight += (self.junk.common + self.junk.uncommon + self.junk.rare) * 2;
        self.carry_wgt = total_weight;
    }
    pub fn calculate_poison_dr(&mut self) {
        self.poison_dr = if self.is_mutant() || self.is_robot() {
            99
        } else if self.has_perk(87) {
            2
        } else {
            0
        };
    }
    pub fn calculate_base_dr(&mut self) {
        let rd_dr = if self.is_mutant() || self.is_robot() {
            99
        } else {
            let atom = if self.origin.clone().unwrap().id == 13 { 1 } else { 0 };
            let rad_res = self.perk_ranks(73);
            atom + rad_res
        };
        let barbarian = if self.has_perk(8) {
            get_staggered_bonus(self.special.strength.value)
        } else { 0 };
        let toughness = self.perk_ranks(94);
        let evasive = if self.has_perk(167) {
            get_staggered_bonus(self.special.agility.value)
        } else { 0 };
        let ph_dr = barbarian + toughness + evasive;
        //energy dr
        let refractor = self.perk_ranks(74);
        let en_dr = evasive + refractor;

        self.base_dr = BaseDR {
            ph_dr,
            en_dr,
            rd_dr,
        };
    }
    pub fn set_base_points(&mut self) {
        self.hp = self.hp_max;
        self.luck_points = self.luck_points_max;
    }
    pub fn calculate_combat_stats(&mut self) {
        let agi = self.special.agility.value;
        let per = self.special.perception.value;
        let end = self.special.endurance.value;
        let lck = self.special.luck.value;
        //defense
        self.defense = if agi >= 9 { 2 } else { 1 };
        //initiative
        self.initiative = per + agi;
        //max hp
        self.hp_max = end + lck + self.perk_ranks(51) * end;
    }
    pub fn calculate_lp(&mut self) {
        self.luck_points_max = if self.is_gifted() { self.special.luck.value - 1 } else { self.special.luck.value };
    }
    pub fn set_companion(&mut self) {
        self.companion = if self.has_perk(28) {
            CompanionType::Dogmeat
        } else if self.has_perk(105) {
            CompanionType::Human
        } else if self.has_perk(118) {
            CompanionType::Robot
        } else {
            CompanionType::None
        };
    }
    pub fn full_update(&mut self) {
        self.calculate_level();
        self.calculate_xp_next();
        self.update_type();
        self.limb_dr.update_active(self.robot.clone());
        self.special.apply_max(&self.clone());
        self.skills.apply_max(&self.clone());
        self.calculate_carry_weight();
        self.calculate_base_dr();
        self.calculate_poison_dr();
        self.calculate_combat_stats();
        self.melee_mod.calculate(self.clone());
        self.calculate_lp();
        self.set_companion();
    }
    pub fn compute_stats(&mut self) -> bool {
        //carry weight
        self.calculate_carry_weight();
        //poison dr
        self.calculate_poison_dr();
        //base dr
        self.calculate_base_dr();
        //combat stats
        self.calculate_combat_stats();
        let is_nocturnal = self.has_perk(111);
        //melee damage
        self.melee_mod.calculate(self.clone());
        //max luck points
        self.calculate_lp();
        //companion
        self.set_companion();
        is_nocturnal
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
}

impl Player {
    pub fn new() -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Party {
    pub id: Uuid,
    pub name: String,
    pub ap_players: i32,
    pub ap_gm: i32,
    pub max_ap: i32,
}

impl Party {
    pub fn new() -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
            ap_players: 0,
            ap_gm: 0,
            max_ap: 6,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Origin {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub can_ghoul: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Background {
    pub id: i32,
    pub name: String,
    pub desc: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Trait {
    pub id: i32,
    pub name: String,
    pub desc: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum MutantType {
    None,
    SuperMutant,
    Nightkin,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum RobotType {
    None,
    Handy,
    Protectron,
    Robobrain,
    Securitron,
    Synth,
    Assaultron,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CompanionType {
    None,
    Dogmeat,
    Human,
    Robot,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SpecialAttr {
    Strength,
    Perception,
    Endurance,
    Charisma,
    Intelligence,
    Agility,
    Luck,
}
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Special {
    pub strength: SpecialBlock,
    pub perception: SpecialBlock,
    pub endurance: SpecialBlock,
    pub charisma: SpecialBlock,
    pub intelligence: SpecialBlock,
    pub agility: SpecialBlock,
    pub luck: SpecialBlock,
}

impl Special {
    fn new() -> Self {
        Self {
            strength: SpecialBlock::new(),
            perception: SpecialBlock::new(),
            endurance: SpecialBlock::new(),
            charisma: SpecialBlock::new(),
            intelligence: SpecialBlock::new(),
            agility: SpecialBlock::new(),
            luck: SpecialBlock::new(),
        }
    }
    pub fn apply_max(&mut self, character: &Character) {
        match character.mutant {
            MutantType::None => {
                self.intelligence.max = 10;
                self.charisma.max = 10;
                self.strength.max = 10;
                self.endurance.max = 10;
                return
            },
            MutantType::SuperMutant => {
                self.intelligence.max = 6;
                self.charisma.max = 6;
            },
            MutantType::Nightkin => {
                self.intelligence.max = 8;
                self.charisma.max = 8;
            }
        }
        self.strength.max = 12;
        self.endurance.max = 12;
    }
    pub fn mut_special_block(&mut self) -> [&mut SpecialBlock; 7] {
        [
            &mut self.strength,
            &mut self.perception,
            &mut self.endurance,
            &mut self.charisma,
            &mut self.intelligence,
            &mut self.agility,
            &mut self.luck,
        ]
    }
    pub fn special_block(&self) -> [SpecialBlock; 7] {
        [
            self.strength.clone(),
            self.perception.clone(),
            self.endurance.clone(),
            self.charisma.clone(),
            self.intelligence.clone(),
            self.agility.clone(),
            self.luck.clone(),
        ]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SpecialBlock {
    pub value: i32,
    pub gifted: bool,
    pub trained: i32,
    pub max: i32,
}

impl SpecialBlock {
    fn new() -> Self {
        Self {
            value: 5,
            gifted: false,
            trained: 0,
            max: 10,
        }
    }
    pub fn can_increase(&self, state: &SpecialState, character: &Character) -> bool {
        self.value < self.max && state.remaining_points(character) > 0
    }
    pub fn can_decrease(&self, character: &Character) -> bool {
        self.value > 4 + if character.is_mutant() { 2 } else { 0 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Skill {
    Athletics,
	Barter,
	BigGuns,
	EnergyWeapons,
	Explosives,
    Lockpick,
	Medicine,
	MeleeWeapons,
	Pilot,
	Repair,
    Science,
	SmallGuns,
	Sneak,
	Speech,
	Survival,
    Throwing,
	Unarmed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Skills {
    pub athletics: SkillBlock,
	pub barter: SkillBlock,
	pub big_guns: SkillBlock,
	pub energy_weapons: SkillBlock,
	pub explosives: SkillBlock,
    pub lockpick: SkillBlock,
	pub medicine: SkillBlock,
	pub melee_weapons: SkillBlock,
	pub pilot: SkillBlock,
	pub repair: SkillBlock,
    pub science: SkillBlock,
	pub small_guns: SkillBlock,
	pub sneak: SkillBlock,
	pub speech: SkillBlock,
	pub survival: SkillBlock,
    pub throwing: SkillBlock,
	pub unarmed: SkillBlock,
}

impl Skills {
    fn new() -> Self {
        Self {
            athletics: SkillBlock::new(),
            barter: SkillBlock::new(),
            big_guns: SkillBlock::new(),
            energy_weapons: SkillBlock::new(),
            explosives: SkillBlock::new(),
            lockpick: SkillBlock::new(),
            medicine: SkillBlock::new(),
            melee_weapons: SkillBlock::new(),
            pilot: SkillBlock::new(),
            repair: SkillBlock::new(),
            science: SkillBlock::new(),
            small_guns: SkillBlock::new(),
            sneak: SkillBlock::new(),
            speech: SkillBlock::new(),
            survival: SkillBlock::new(),
            throwing: SkillBlock::new(),
            unarmed: SkillBlock::new(),
        }
    }
    pub fn skill_block(&self) -> [SkillBlock; 17] {
        [
            self.athletics.clone(),
            self.barter.clone(),
            self.big_guns.clone(),
            self.energy_weapons.clone(),
            self.explosives.clone(),
            self.lockpick.clone(),
            self.medicine.clone(),
            self.melee_weapons.clone(),
            self.pilot.clone(),
            self.repair.clone(),
            self.science.clone(),
            self.small_guns.clone(),
            self.sneak.clone(),
            self.speech.clone(),
            self.survival.clone(),
            self.throwing.clone(),
            self.unarmed.clone(),
        ]
    }
    pub fn mut_skill_block(&mut self) -> [&mut SkillBlock; 17] {
        [
            &mut self.athletics,
            &mut self.barter,
            &mut self.big_guns,
            &mut self.energy_weapons,
            &mut self.explosives,
            &mut self.lockpick,
            &mut self.medicine,
            &mut self.melee_weapons,
            &mut self.pilot,
            &mut self.repair,
            &mut self.science,
            &mut self.small_guns,
            &mut self.sneak,
            &mut self.speech,
            &mut self.survival,
            &mut self.throwing,
            &mut self.unarmed,
        ]
    }
    pub fn standard_tags(&self) -> i32 {
        self.skill_block().iter().filter(|t| t.tagged == TagType::Standard).count() as i32
    }
    pub fn trait_tags(&self) -> i32 {
        self.skill_block().iter().filter(|t| t.tagged == TagType::Trait).count() as i32
    }
    pub fn perk_tags(&self) -> i32 {
        self.skill_block().iter().filter(|t| t.tagged == TagType::Perk).count() as i32
    }
    pub fn total_tags(&self) -> i32 {
        self.standard_tags() + self.trait_tags() + self.perk_tags()
    }
    pub fn zip_skilled(&self) -> Vec<(usize,usize)> {
        let mut zipped = vec![];
        let skilled_count = self.athletics.skilled.len();
        for rank in 0..skilled_count {
            let mut sk_a: usize = 17;
            for (i, skill) in self.skill_block().iter().enumerate() {
                match skill.is_skilled(rank) {
                    1 => {if sk_a == 17 {sk_a = i} else {zipped.push((sk_a,i)); continue}},
                    2 => {zipped.push((i,i)); continue},
                    _ => {},
                }
            }
        }
        zipped
    }
    pub fn available_tags(&self, character: &Character) -> Vec<usize> {
        let mut available = vec![];
        for (i,skill) in self.skill_block().iter().enumerate() {
            if !skill.is_tagged() && !(i == 10 && character.has_trait(27)) { available.push(i) }
        }
        available
    }
    pub fn perk_tagged(&self) -> Vec<usize> {
        let mut tagged = vec![];
        for (i,skill) in self.skill_block().iter().enumerate() {
            if skill.tagged == TagType::Perk {tagged.push(i)}
        }
        tagged
    }
    pub fn apply_max(&mut self, character: &Character) {
        let skill_max = character.level.clamp(3,6);
        let mutant = character.is_mutant();
        let good = character.has_trait(13);
        let skills = self.mut_skill_block();
        for i in 0..17 {
            if mutant {
                skills[i].max = 4.min(skill_max);
            } else {
                skills[i].max = skill_max;
            }
            if good && [0,2,3,4,5,7,8,11,12,14,15,16].contains(&i) {
                skills[i].max = 4.min(skill_max);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SkillBlock {
    pub ranks: i32,
    pub tagged: TagType,
    //skilled will create a new entry in every skill, either 0, 1 or 2. this aligns the skill selections with each rank of the perk
    pub skilled: Vec<i32>,
    pub total: i32,
    pub max: i32,
}

impl SkillBlock {
    fn new() -> Self {
        Self {
            ranks: 0,
            tagged: TagType::None,
            skilled: vec![],
            total: 0,
            max: 3,
        }
    }
    pub fn is_tagged(&self) -> bool {
        self.tagged != TagType::None
    }
    pub fn update(&mut self) {
        self.total = self.ranks + if self.is_tagged() { 2 } else { 0 };
    }
    pub fn is_skilled(&self, rank: usize) -> i32 {
        self.skilled[rank]
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum TagType {
    None,
    Trait,
    Perk,
    Standard,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Perk {
    pub id: i32,
    pub name: String,
    pub desc: Vec<String>,
    pub ranks: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MeleeModifiers {
    pub melee: i32,
    pub unarmed: i32,
    pub sneak: i32,
}

impl MeleeModifiers {
    pub fn new() -> Self {
        Self {
            melee: 0,
            unarmed: 0,
            sneak: 0,
        }
    }
    pub fn calculate(&mut self, character: Character) {
        let brutal = if character.has_trait(8) { 1 } else { 0 };
        let built = if character.has_trait(23) { 1 } else { 0 };
        self.melee = get_staggered_bonus(character.special.strength.value) + brutal + built;
        self.unarmed = if character.has_perk(46) { 1 } else { 0 };
        self.sneak = if character.has_perk(61) { 2 } else { 0 };
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BaseDR {
    pub ph_dr: i32,
    pub en_dr: i32,
    pub rd_dr: i32,
}

impl BaseDR {
    pub fn new() -> Self {
        Self {
            ph_dr: 0,
            en_dr: 0,
            rd_dr: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Limbs {
    pub head: Limb,
    pub torso: Limb,
    pub body: Limb,
    pub arm_left: Limb,
    pub arm_right: Limb,
    pub leg_left: Limb,
    pub leg_right: Limb,
    pub optics: Limb,
    pub arm_1: Limb,
    pub arm_2: Limb,
    pub arm_3: Limb,
    pub thruster: Limb,
    pub wheel: Limb,
    pub track_left: Limb,
    pub track_right: Limb,
}

impl Limbs {
    pub fn new() -> Self {
        Self {
            head: Limb::new_active(),
            torso: Limb::new_active(),
            body: Limb::new_inactive(),
            arm_left: Limb::new_active(),
            arm_right: Limb::new_active(),
            leg_left: Limb::new_active(),
            leg_right: Limb::new_active(),
            optics: Limb::new_inactive(),
            arm_1: Limb::new_inactive(),
            arm_2: Limb::new_inactive(),
            arm_3: Limb::new_inactive(),
            thruster: Limb::new_inactive(),
            wheel: Limb::new_inactive(),
            track_left: Limb::new_inactive(),
            track_right: Limb::new_inactive(),
        }
    }
    pub fn update_active(&mut self, robot_type: RobotType) {
        match robot_type {
            RobotType::Handy => {
                self.head = Limb::new_inactive();
                self.torso = Limb::new_inactive();
                self.body = Limb::new_active();
                self.arm_left = Limb::new_inactive();
                self.arm_right = Limb::new_inactive();
                self.leg_left = Limb::new_inactive();
                self.leg_right = Limb::new_inactive();
                self.optics = Limb::new_active();
                self.arm_1 = Limb::new_active();
                self.arm_2 = Limb::new_active();
                self.arm_3 = Limb::new_active();
                self.thruster = Limb::new_active();
                self.wheel = Limb::new_inactive();
                self.track_left = Limb::new_inactive();
                self.track_right = Limb::new_inactive();
            },
            RobotType::Robobrain => {
                self.head = Limb::new_active();
                self.torso = Limb::new_inactive();
                self.body = Limb::new_active();
                self.arm_left = Limb::new_active();
                self.arm_right = Limb::new_active();
                self.leg_left = Limb::new_inactive();
                self.leg_right = Limb::new_inactive();
                self.optics = Limb::new_inactive();
                self.arm_1 = Limb::new_inactive();
                self.arm_2 = Limb::new_inactive();
                self.arm_3 = Limb::new_inactive();
                self.thruster = Limb::new_inactive();
                self.wheel = Limb::new_inactive();
                self.track_left = Limb::new_active();
                self.track_right = Limb::new_active();
            },
            RobotType::Securitron => {
                self.head = Limb::new_active();
                self.torso = Limb::new_inactive();
                self.body = Limb::new_active();
                self.arm_left = Limb::new_active();
                self.arm_right = Limb::new_active();
                self.leg_left = Limb::new_inactive();
                self.leg_right = Limb::new_inactive();
                self.optics = Limb::new_inactive();
                self.arm_1 = Limb::new_inactive();
                self.arm_2 = Limb::new_inactive();
                self.arm_3 = Limb::new_inactive();
                self.thruster = Limb::new_inactive();
                self.wheel = Limb::new_active();
                self.track_left = Limb::new_inactive();
                self.track_right = Limb::new_inactive();
            },
            RobotType::None => {
                self.head = Limb::new_active();
                self.torso = Limb::new_active();
                self.body = Limb::new_inactive();
                self.arm_left = Limb::new_active();
                self.arm_right = Limb::new_active();
                self.leg_left = Limb::new_active();
                self.leg_right = Limb::new_active();
                self.optics = Limb::new_inactive();
                self.arm_1 = Limb::new_inactive();
                self.arm_2 = Limb::new_inactive();
                self.arm_3 = Limb::new_inactive();
                self.thruster = Limb::new_inactive();
                self.wheel = Limb::new_inactive();
                self.track_left = Limb::new_inactive();
                self.track_right = Limb::new_inactive();
            }
            _ => {
                self.head = Limb::new_active();
                self.torso = Limb::new_inactive();
                self.body = Limb::new_active();
                self.arm_left = Limb::new_active();
                self.arm_right = Limb::new_active();
                self.leg_left = Limb::new_active();
                self.leg_right = Limb::new_active();
                self.optics = Limb::new_inactive();
                self.arm_1 = Limb::new_inactive();
                self.arm_2 = Limb::new_inactive();
                self.arm_3 = Limb::new_inactive();
                self.thruster = Limb::new_inactive();
                self.wheel = Limb::new_inactive();
                self.track_left = Limb::new_inactive();
                self.track_right = Limb::new_inactive();
            }
        }
    }
    pub fn mut_active_limbs(&mut self) -> Vec<(&mut Limb,String)> {
        let mut active: Vec<(&mut Limb, String)> = vec![];
        if self.head.active { active.push((&mut self.head, "head".to_string())) };
        if self.torso.active { active.push((&mut self.torso, "torso".to_string())) };
        if self.body.active { active.push((&mut self.body, "body".to_string())) };
        if self.arm_left.active { active.push((&mut self.arm_left, "arm_left".to_string())) };
        if self.arm_right.active { active.push((&mut self.arm_right, "arm_right".to_string())) };
        if self.leg_left.active { active.push((&mut self.leg_left, "leg_left".to_string())) };
        if self.leg_right.active { active.push((&mut self.leg_right, "leg_right".to_string())) };
        if self.optics.active { active.push((&mut self.optics, "optics".to_string())) };
        if self.arm_1.active { active.push((&mut self.arm_1, "arm_1".to_string())) };
        if self.arm_2.active { active.push((&mut self.arm_2, "arm_2".to_string())) };
        if self.arm_3.active { active.push((&mut self.arm_3, "arm_3".to_string())) };
        if self.thruster.active { active.push((&mut self.thruster, "thruster".to_string())) };
        if self.wheel.active { active.push((&mut self.wheel, "wheel".to_string())) };
        if self.track_left.active { active.push((&mut self.track_left, "track_left".to_string())) };
        if self.track_right.active { active.push((&mut self.track_right, "track_right".to_string())) };
        active
    }
    pub fn update_dr(&mut self, base_dr: BaseDR, ironclad_ranks: i32, junk: i32, junk_ranks: i32) {
        for (loc,_) in self.mut_active_limbs() {
            let mut equip_dr = BaseDR::new();
                let junk_dr = (junk / 5).min(junk_ranks);
            for item in loc.equipped.clone() {
                equip_dr.ph_dr += item.ph_dr + if item.apparel_type == ApparelType::Armor {ironclad_ranks} else { 0 };
                equip_dr.en_dr += item.en_dr + if item.apparel_type == ApparelType::Armor {ironclad_ranks} else { 0 };
                equip_dr.rd_dr += item.rd_dr;
            }
            loc.ph_dr = base_dr.ph_dr + equip_dr.ph_dr + junk_dr;
            loc.en_dr = base_dr.en_dr + equip_dr.en_dr + junk_dr;
            if base_dr.rd_dr < 99 {
                loc.rd_dr = base_dr.rd_dr + equip_dr.rd_dr;
            } else {
                loc.rd_dr = 99;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Limb {
    pub active: bool,
    pub ph_dr: i32,
    pub en_dr: i32,
    pub rd_dr: i32,
    pub injuries: i32,
    pub equipped: Vec<Apparel>,
}

impl Limb {
    fn new_active() -> Self {
        Self {
            active: true,
            ph_dr: 0,
            en_dr: 0,
            rd_dr: 0,
            injuries: 0,
            equipped: vec![],
        }
    }
    fn new_inactive() -> Self {
        Self {
            active: false,
            ph_dr: 0,
            en_dr: 0,
            rd_dr: 0,
            injuries: 0,
            equipped: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Weapon {
    pub id: i32,
    pub name: String,
    pub prefix: String,
    pub skill: Skill,
    pub target: i32,
    pub tag: bool,
    pub damage: i32,
    pub effects: Vec<String>, // new struct?
    pub dam_type: DamageType,
    pub rate: i32,
    pub range: String,
    pub qualities: Vec<String>, // new struct?
    pub ammo: String,
    pub wgt: i32,
    pub mods: Vec<WeaponMods>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DamageType {
    Ph,
    En,
    PhEn,
    Rad,
    EnRad,
    Poi,
    All,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WeaponMods {
    pub slot: WeaponSlot,
    pub installed: bool,
    pub id: i32,
    pub name: String,
    pub prefix: String,
    pub wgt: i32,
    pub damage_set: i32,
    pub damage_chg: i32,
    pub rate_set: i32,
    pub rate_chg: i32,
    pub range_chg: i32,
    //pub ammo_set: Option<AmmoData>,
    pub ammo_set: Option<String>,
    pub effect_add:  Vec<(String, Option<i32>)>,
    pub effect_rem:  Vec<(String, Option<i32>)>,
    pub quality_add: Vec<(String, Option<i32>)>,
    pub quality_rem: Vec<(String, Option<i32>)>,
    //pub slot_add: Option<WeaponSlot>,
    pub slot_add: String,
    pub damage_type_set: Option<DamageType>,
    //pub weapon_add: Option<Weapon>,
    pub weapon_add: String,
    pub special_ability: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WeaponSlot {
    None,
    Receiver,
    Barrel,
    Stock,
    Grip,
    Magazine,
    Sights,
    Muzzle,
    Capacitors,
    Dish,
    Fuel,
    Tank,
    Nozzle,
    Blade,
    Blunt,
    Frame,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ammo {
    pub ammo: AmmoData,
    pub variants: Vec<AmmoData>,
}
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AmmoData {
    pub id: i32,
    pub name: String,
    pub wgt: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AmmoInv {
    pub ammo: AmmoData,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Apparel {
    pub id: i32,
    pub name: String,
    pub prefix: String,
    pub apparel_type: ApparelType,
    pub ph_dr: i32,
    pub en_dr: i32,
    pub rd_dr: i32,
    pub wgt: i32,
    pub effects: Vec<String>,
    pub covers: Vec<BodyLocation>,
    pub equipped: bool,
    pub db_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ApparelType {
    Clothing,
    Outfit,
    Headgear,
    Armor,
    PowerArmor,
    RobotArmor,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BodyLocation {
    None,
    Head,
    ArmLeft,
    ArmRight,
    Torso,
    LegLeft,
    LegRight,
    Optics,
    Arm1,
    Arm2,
    Arm3,
    Body,
    Thruster,
    Wheel,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RobotModule {
    pub id: i32,
    pub name: String,
    pub installed: bool,
    pub effect: Vec<String>,
    pub wgt: i32,
    pub db_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Consumable {
    pub id: i32,
    pub name: String,
    pub consumable_type: ConsumableType,
    pub health: i32,
    pub effects: Vec<String>,
    pub rads: i32,
    pub wgt: i32,
    pub duration: String,
    pub addiction: i32,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ConsumableType {
    Chem,
    Food,
    Beverage,
    Other,
    Publication,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Gear {
    pub id: i32,
    pub name: String,
    pub effect: Vec<String>,
    pub wgt: i32,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Junk {
    pub common: i32,
    pub uncommon: i32,
    pub rare: i32,
}

impl Junk {
    fn new() -> Self {
        Self {
            common: 0,
            uncommon: 0,
            rare: 0,
        }
    }
}