use uuid::Uuid;

pub struct Character {
    id: Uuid,
    name: String,
    player: Uuid,
    party: Uuid,
    level: i32,
    xp: i32,
    origin: Origin,
    background: Background,
    traits: Vec<Trait>,
    ghoul: bool,
    mutant: MutantType,
    robot: RobotType,
    robot_hat: Option<Apparel>,
    special: Special,
    luck_points: i32,
    luck_points_max: i32,
    rad_points: i32,
    skills: Skills,
    perks: Vec<Perk>,
    melee_mod: MeleeModifiers,
    defense: i32,
    initiative: i32,
    hp: i32,
    hp_max: i32,
    poison_dr: i32,
    limb_dr: Limbs,
    weapons: Vec<Weapon>,
    ammo: Vec<AmmoInv>,
    apparel: Vec<Apparel>,
    robot_modules: Vec<RobotModule>,
    consumables: Vec<Consumable>,
    gear: Vec<Gear>,
    junk: Junk,
    misc: Vec<String>,
    carry_wgt: i32,
    carry_wgt_max: i32,
    notes: String,
}

pub struct Origin {
    id: i32,
    name: String,
    desc: String,
    can_ghoul: bool,
}

pub struct Background {
    id: i32,
    name: String,
    desc: String,
}

pub struct Trait {
    id: i32,
    name: String,
    desc: String,
}

pub enum MutantType {
    None,
    SuperMutant,
    Nightkin,
}

pub enum RobotType {
    None,
    Handy,
    Protectron,
    Robobrain,
    Securitron,
    Synth,
    Assaultron,
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
    strength: SpecialBlock,
    perception: SpecialBlock,
    endurance: SpecialBlock,
    charisma: SpecialBlock,
    intelligence: SpecialBlock,
    agility: SpecialBlock,
    luck: SpecialBlock,
}

pub struct SpecialBlock {
    value: i32,
    gifted: bool,
    trained: i32,
    max: i32,
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
    athletics: SkillBlock,
	barter: SkillBlock,
	big_guns: SkillBlock,
	energy_weapons: SkillBlock,
	explosives: SkillBlock,
    lockpick: SkillBlock,
	medicine: SkillBlock,
	melee_weapons: SkillBlock,
	pilot: SkillBlock,
	repair: SkillBlock,
    science: SkillBlock,
	small_guns: SkillBlock,
	sneak: SkillBlock,
	speech: SkillBlock,
	survival: SkillBlock,
    throwing: SkillBlock,
	unarmed: SkillBlock,
}

pub struct SkillBlock {
    ranks: i32,
    tagged: TagType,
    skilled: Vec<i32>,
    total: i32,
    max: i32,
}

pub enum TagType {
    None,
    Trait,
    Perk,
    Standard,
}

pub struct Perk {
    id: i32,
    name: String,
    desc: Vec<String>,
    ranks: i32,
}

pub struct MeleeModifiers {
    melee: i32,
    unarmed: i32,
    sneak: i32,
}

pub struct Limbs {
    head: Limb,
    torso: Limb,
    body: Limb,
    arm_left: Limb,
    arm_right: Limb,
    leg_left: Limb,
    leg_right: Limb,
    optics: Limb,
    arm_1: Limb,
    arm_2: Limb,
    arm_3: Limb,
    thruster: Limb,
    wheel: Limb,
    track_left: Limb,
    track_right: Limb,
}

pub struct Limb {
    active: bool,
    ph_dr: i32,
    en_dr: i32,
    rd_dr: i32,
    injuries: i32,
    equipped: Apparel,
}

pub struct Weapon {
    id: i32,
    name: String,
    prefix: String,
    skill: Skill,
    target: i32,
    tag: bool,
    damage: i32,
    effects: Vec<String>, // new struct?
    dam_type: DamageType,
    rate: i32,
    range: String,
    qualities: Vec<String>, // new struct?
    ammo: String,
    wgt: i32,
    mods: Vec<WeaponMods>,
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
    slot: WeaponSlot,
    installed: bool,
    id: i32,
    name: String,
    prefix: String,
    wgt: i32,
    damage_set: i32,
    damage_chg: i32,
    rate_set: i32,
    rate_chg: i32,
    range_set: i32,
    range_chg: i32,
    ammo_set: AmmoData,
    effect_add: Vec<String>,
    effect_rem: Vec<String>,
    quality_add: Vec<String>,
    quality_rem: Vec<String>,
    slot_add: WeaponSlot,
    damage_type_set: DamageType,
    weapon_add: Weapon,
    special_ability: String,
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
    id: i32,
    name: String,
    wgt: i32
}

pub struct Ammo {
    ammo: AmmoData,
    variants: Vec<AmmoData>,
}

pub struct AmmoInv {
    ammo: AmmoData,
    quantity: i32,
}

pub struct Apparel {
    id: i32,
    name: String,
    prefix: String,
    apparel_type: ApparelType,
    ph_dr: i32,
    en_dr: i32,
    rd_dr: i32,
    wgt: i32,
    effects: Vec<String>,
    covers: Vec<BodyLocation>,
    equipped: bool,
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
    id: i32,
    name: String,
    installed: bool,
    effect: Vec<String>,
    wgt: i32,
}

pub struct Consumable {
    id: i32,
    name: String,
    consumable_type: ConsumableType,
    health: i32,
    effects: Vec<String>,
    rads: i32,
    wgt: i32,
    duration: String,
    addiction: i32,
    quantity: i32,
}

pub enum ConsumableType {
    Chem,
    Food,
    Beverage,
    Other,
    Publication,
}

pub struct Gear {
    id: i32,
    name: String,
    effect: Vec<String>,
    wgt: i32,
    quantity: i32,
}

pub struct Junk {
    common: i32,
    uncommon: i32,
    rare: i32,
}