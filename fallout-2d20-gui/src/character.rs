use uuid::Uuid;

use crate::screens2::special_assignment::SpecialState;

pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub player: Player,
    pub party: Option<Party>,
    pub level: i32,
    pub xp: i32,
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
    pub melee_mod: MeleeModifiers,
    pub defense: i32,
    pub initiative: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub poison_dr: i32,
    pub limb_dr: Limbs,
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
    pub fn new(player: Player, party: Option<Party>) -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
            player: player,
            party: party,
            level: 1,
            xp: 0,
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
            melee_mod: MeleeModifiers::new(),
            defense: 0,
            initiative: 10,
            hp: 10,
            hp_max: 10,
            poison_dr: 0,
            limb_dr: Limbs::new(),
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
    pub fn is_gifted(&self) -> bool {
        self.traits.iter().any(|t| {
            t.id == 7
        })
    }
    pub fn is_mutant(&self) -> bool {
        self.mutant != MutantType::None
    }
    pub fn is_robot(&self) -> bool {
        self.robot != RobotType::None
    }
    pub fn total_skill(&self) -> i32 {
        self.skills.athletics.total +
            self.skills.barter.total +
            self.skills.big_guns.total +
            self.skills.energy_weapons.total +
            self.skills.explosives.total +
            self.skills.lockpick.total +
            self.skills.medicine.total +
            self.skills.melee_weapons.total +
            self.skills.pilot.total +
            self.skills.repair.total +
            self.skills.science.total +
            self.skills.small_guns.total +
            self.skills.sneak.total +
            self.skills.speech.total +
            self.skills.survival.total +
            self.skills.throwing.total +
            self.skills.unarmed.total
    }
}

pub struct Player {
    id: Uuid,
    name: String,
}

impl Player {
    pub fn new() -> Self {
        Self {
            id: (Uuid::now_v7()),
            name: String::new(),
        }
    }
}

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

pub struct Origin {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub can_ghoul: bool,
}

pub struct Background {
    pub id: i32,
    pub name: String,
    pub desc: String,
}

pub struct Trait {
    pub id: i32,
    pub name: String,
    pub desc: String,
}

#[derive(PartialEq)]
pub enum MutantType {
    None,
    SuperMutant,
    Nightkin,
}

#[derive(PartialEq)]
pub enum RobotType {
    None,
    Handy,
    Protectron,
    Robobrain,
    Securitron,
    Synth,
    Assaultron,
}

#[derive(PartialEq)]
pub enum CompanionType {
    None,
    Dogmeat,
    Human,
    Robot,
}

pub enum SpecialAttr {
    Strength,
    Perception,
    Endurance,
    Charisma,
    Intelligence,
    Agility,
    Luck,
}

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
    pub fn apply_max(character: &mut Character) {
        match character.mutant {
            MutantType::None => {
                character.special.intelligence.max = 10;
                character.special.charisma.max = 10;
                character.special.strength.max = 10;
                character.special.endurance.max = 10;
                return
            },
            MutantType::SuperMutant => {
                character.special.intelligence.max = 6;
                character.special.charisma.max = 6;
            },
            MutantType::Nightkin => {
                character.special.intelligence.max = 8;
                character.special.charisma.max = 8;
            }
        }
        character.special.strength.max = 12;
        character.special.endurance.max = 12;
    }
}

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
        self.value < self.max && state.remaining_points(character) == 0
    }
    pub fn can_decrease(&self, character: &Character) -> bool {
        self.value > 4 + if character.is_mutant() { 2 } else { 0 }
    }
}

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
    pub fn standard_tags(&self) -> i32 {
        [
            self.athletics.tagged == TagType::Standard,
            self.barter.tagged == TagType::Standard,
            self.big_guns.tagged == TagType::Standard,
            self.energy_weapons.tagged == TagType::Standard,
            self.explosives.tagged == TagType::Standard,
            self.lockpick.tagged == TagType::Standard,
            self.medicine.tagged == TagType::Standard,
            self.melee_weapons.tagged == TagType::Standard,
            self.pilot.tagged == TagType::Standard,
            self.repair.tagged == TagType::Standard,
            self.science.tagged == TagType::Standard,
            self.small_guns.tagged == TagType::Standard,
            self.sneak.tagged == TagType::Standard,
            self.speech.tagged == TagType::Standard,
            self.survival.tagged == TagType::Standard,
            self.throwing.tagged == TagType::Standard,
            self.unarmed.tagged == TagType::Standard,
        ].iter().filter(|&&b| b).count() as i32
    }
    pub fn trait_tags(&self) -> i32 {
        [
            self.athletics.tagged == TagType::Trait,
            self.barter.tagged == TagType::Trait,
            self.big_guns.tagged == TagType::Trait,
            self.energy_weapons.tagged == TagType::Trait,
            self.explosives.tagged == TagType::Trait,
            self.lockpick.tagged == TagType::Trait,
            self.medicine.tagged == TagType::Trait,
            self.melee_weapons.tagged == TagType::Trait,
            self.pilot.tagged == TagType::Trait,
            self.repair.tagged == TagType::Trait,
            self.science.tagged == TagType::Trait,
            self.small_guns.tagged == TagType::Trait,
            self.sneak.tagged == TagType::Trait,
            self.speech.tagged == TagType::Trait,
            self.survival.tagged == TagType::Trait,
            self.throwing.tagged == TagType::Trait,
            self.unarmed.tagged == TagType::Trait,
        ].iter().filter(|&&b| b).count() as i32
    }
    pub fn perk_tags(&self) -> i32 {
        [
            self.athletics.tagged == TagType::Perk,
            self.barter.tagged == TagType::Perk,
            self.big_guns.tagged == TagType::Perk,
            self.energy_weapons.tagged == TagType::Perk,
            self.explosives.tagged == TagType::Perk,
            self.lockpick.tagged == TagType::Perk,
            self.medicine.tagged == TagType::Perk,
            self.melee_weapons.tagged == TagType::Perk,
            self.pilot.tagged == TagType::Perk,
            self.repair.tagged == TagType::Perk,
            self.science.tagged == TagType::Perk,
            self.small_guns.tagged == TagType::Perk,
            self.sneak.tagged == TagType::Perk,
            self.speech.tagged == TagType::Perk,
            self.survival.tagged == TagType::Perk,
            self.throwing.tagged == TagType::Perk,
            self.unarmed.tagged == TagType::Perk,
        ].iter().filter(|&&b| b).count() as i32
    }
    pub fn total_tags(&self) -> i32 {
        self.standard_tags() + self.trait_tags() + self.perk_tags()
    }
    pub fn zip_skilled(&self) -> Vec<(usize,usize)> {
        let mut zipped = vec![];
        for i in 0..self.athletics.skilled.len() {
            let mut sk_a: usize = 17;
            let mut sk_b: usize = 17;
            match self.athletics.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 0} else {sk_b = 0; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 0; sk_b = 0; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.barter.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 1} else {sk_b = 1; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 1; sk_b = 1; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.big_guns.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 2} else {sk_b = 2; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 2; sk_b = 2; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.energy_weapons.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 3} else {sk_b = 3; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 3; sk_b = 3; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.explosives.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 4} else {sk_b = 4; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 4; sk_b = 4; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.lockpick.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 5} else {sk_b = 5; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 5; sk_b = 5; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.medicine.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 6} else {sk_b = 6; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 6; sk_b = 6; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.melee_weapons.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 7} else {sk_b = 7; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 7; sk_b = 7; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.pilot.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 8} else {sk_b = 8; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 8; sk_b = 8; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.repair.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 9} else {sk_b = 9; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 9; sk_b = 9; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.science.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 10} else {sk_b = 10; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 10; sk_b = 10; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.small_guns.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 11} else {sk_b = 11; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 11; sk_b = 11; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.sneak.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 12} else {sk_b = 12; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 12; sk_b = 12; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.speech.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 13} else {sk_b = 13; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 13; sk_b = 13; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.survival.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 14} else {sk_b = 14; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 14; sk_b = 14; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.throwing.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 15} else {sk_b = 15; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 15; sk_b = 15; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
            match self.unarmed.is_skilled(i) {
                1 => {if sk_a == 17 {sk_a = 16} else {sk_b = 16; zipped.push((sk_a,sk_b)); continue}},
                2 => {sk_a = 16; sk_b = 16; zipped.push((sk_a,sk_b)); continue},
                _ => {}
            }
        }
        zipped
    }
    pub fn available_tags(&self, character: &Character) -> Vec<usize> {
        let mut available = vec![];
        if !self.athletics.is_tagged() { available.push(0) }
        if !self.barter.is_tagged() { available.push(1) }
        if !self.big_guns.is_tagged() { available.push(2) }
        if !self.energy_weapons.is_tagged() { available.push(3) }
        if !self.explosives.is_tagged() { available.push(4) }
        if !self.lockpick.is_tagged() { available.push(5) }
        if !self.medicine.is_tagged() { available.push(6) }
        if !self.melee_weapons.is_tagged() { available.push(7) }
        if !self.pilot.is_tagged() { available.push(8) }
        if !self.repair.is_tagged() { available.push(9) }
        if !self.science.is_tagged() && !character.traits.iter().any(|t| t.id == 27) { available.push(10) }
        if !self.small_guns.is_tagged() { available.push(11) }
        if !self.sneak.is_tagged() { available.push(12) }
        if !self.speech.is_tagged() { available.push(13) }
        if !self.survival.is_tagged() { available.push(14) }
        if !self.throwing.is_tagged() { available.push(15) }
        if !self.unarmed.is_tagged() { available.push(16) }
        available
    }
    pub fn perk_tagged(&self) -> Vec<usize> {
        let mut tagged = vec![];
        if self.athletics.tagged == TagType::Perk {tagged.push(0)}
        if self.barter.tagged == TagType::Perk {tagged.push(1)}
        if self.big_guns.tagged == TagType::Perk {tagged.push(2)}
        if self.energy_weapons.tagged == TagType::Perk {tagged.push(3)}
        if self.explosives.tagged == TagType::Perk {tagged.push(4)}
        if self.lockpick.tagged == TagType::Perk {tagged.push(5)}
        if self.medicine.tagged == TagType::Perk {tagged.push(6)}
        if self.melee_weapons.tagged == TagType::Perk {tagged.push(7)}
        if self.pilot.tagged == TagType::Perk {tagged.push(8)}
        if self.repair.tagged == TagType::Perk {tagged.push(9)}
        if self.science.tagged == TagType::Perk {tagged.push(10)}
        if self.small_guns.tagged == TagType::Perk {tagged.push(11)}
        if self.sneak.tagged == TagType::Perk {tagged.push(12)}
        if self.speech.tagged == TagType::Perk {tagged.push(13)}
        if self.survival.tagged == TagType::Perk {tagged.push(14)}
        if self.throwing.tagged == TagType::Perk {tagged.push(15)}
        if self.unarmed.tagged == TagType::Perk {tagged.push(16)}
        tagged
    }
}

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

#[derive(PartialEq)]
pub enum TagType {
    None,
    Trait,
    Perk,
    Standard,
}

pub struct Perk {
    pub id: i32,
    pub name: String,
    pub desc: Vec<String>,
    pub ranks: i32,
}

pub struct MeleeModifiers {
    pub melee: i32,
    pub unarmed: i32,
    pub sneak: i32,
}

impl MeleeModifiers {
    fn new() -> Self {
        Self {
            melee: 0,
            unarmed: 0,
            sneak: 0,
        }
    }
}

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
    fn new() -> Self {
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
}

pub struct Limb {
    pub active: bool,
    pub ph_dr: i32,
    pub en_dr: i32,
    pub rd_dr: i32,
    pub injuries: i32,
    pub equipped: Option<Apparel>,
}

impl Limb {
    fn new_active() -> Self {
        Self {
            active: true,
            ph_dr: 0,
            en_dr: 0,
            rd_dr: 0,
            injuries: 0,
            equipped: None,
        }
    }
    fn new_inactive() -> Self {
        Self {
            active: false,
            ph_dr: 0,
            en_dr: 0,
            rd_dr: 0,
            injuries: 0,
            equipped: None,
        }
    }
}

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
    pub range_set: i32,
    pub range_chg: i32,
    pub ammo_set: AmmoData,
    pub effect_add: Vec<String>,
    pub effect_rem: Vec<String>,
    pub quality_add: Vec<String>,
    pub quality_rem: Vec<String>,
    pub slot_add: WeaponSlot,
    pub damage_type_set: DamageType,
    pub weapon_add: Weapon,
    pub special_ability: String,
}

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

pub struct AmmoData {
    pub id: i32,
    pub name: String,
    pub wgt: i32
}

pub struct Ammo {
    pub ammo: AmmoData,
    pub variants: Vec<AmmoData>,
}

pub struct AmmoInv {
    pub ammo: AmmoData,
    pub quantity: i32,
}

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
}

pub enum ApparelType {
    Clothing,
    Outfit,
    Headgear,
    Armor,
    PowerArmor,
    RobotArmor,
}

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

pub struct RobotModule {
    pub id: i32,
    pub name: String,
    pub installed: bool,
    pub effect: Vec<String>,
    pub wgt: i32,
}

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

pub enum ConsumableType {
    Chem,
    Food,
    Beverage,
    Other,
    Publication,
}

pub struct Gear {
    pub id: i32,
    pub name: String,
    pub effect: Vec<String>,
    pub wgt: i32,
    pub quantity: i32,
}

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