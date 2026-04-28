use imgui::Ui;
use sdl2::video::Window;
use crate::db::Db;
use crate::character::{Character, TagType};
use crate::theme::{render_text_wrapped, render_window};
use crate::log_on_change;

pub const SKILLS: [&str; 17] = [
    "Athletics", "Barter", "Big Guns", "Energy Weapons", "Explosives",
    "Lockpick", "Medicine", "Melee Weapons", "Pilot", "Repair",
    "Science", "Small Guns", "Sneak", "Speech", "Survival",
    "Throwing", "Unarmed",
];

/*
pub const SKILL_STRUCTS: [&str; 17] = [
    "athletics", "barter", "big_guns", "energy_weapons", "explosives",
    "lockpick", "medicine", "meleeweapons", "pilot", "repair",
    "science", "smallguns", "sneak", "speech", "survival",
    "throwing", "unarmed",
];
*/

pub struct SkillState {
    pub extra_trait_options: Vec<usize>,
    pub extra_tags: Vec<usize>,
    pub extra_trait_count: i32,
    pub forced_trait: bool,
    pub assigned_points: [i32; 17],
    pub available_points: i32,
    pub max_assignable: i32,
    pub total_points: i32,
}

impl SkillState {
    pub fn new(character: Character) -> Self {
        let total_points = character.total_skill();
        let extra_trait_options = if character.has_any_trait(vec![1,24]) {
            vec![3,9,10]
        } else if character.has_any_trait(vec![5,11,21]) {
            vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]
        } else if character.has_trait(2) {
            vec![14]
        } else if character.has_trait(12) {
            vec![3,11]
        } else if character.has_trait(13) {
            vec![1,6,9,10,13]
        } else { vec![] };
        let extra_trait_count = if character.has_any_trait(vec![1, 2, 5, 11, 12, 21, 24]) { 1 } else if character.has_trait(13) { 2 } else { 0 };
        let forced_trait = character.has_trait(2);
        Self {
            extra_trait_options,
            extra_tags: vec![],
            extra_trait_count,
            forced_trait,
            assigned_points: [0; 17],
            available_points: 9 + character.special.intelligence.value + character.level - 1,
            max_assignable: 9 + character.special.intelligence.value + character.level - 1,
            total_points,
        }
    }
    pub fn is_complete(&self, _character: &Character) -> bool {
        self.available_points == 0 && self.extra_tags.len() as i32 == self.extra_trait_count //this might need more checks, which is why character is fed in
    }
}

pub fn render_skill_assignment(
    ui: &Ui,
    window: &Window,
    state: &mut SkillState,
    _db: &Db, //leaving this here so i can pull in the skill descriptions eventually
    character: &mut Character,
) -> f32 {
    let Some((w, h, _token)) = render_window(ui, window, "##skill_assignment", "Skill Assignment")
        else { return 0.0 };

    ui.text("SKILLS");
    ui.separator();
    ui.spacing();

    let remaining = state.available_points;
    //probably want to do checks related to this
    let tags_standard = character.skills.standard_tags();
    let tags_traits = character.skills.trait_tags();
    let _tags_perks = character.skills.perk_tags();
    let total: i32 = state.assigned_points.iter().sum();

    if remaining < 0 {
        render_text_wrapped(true, false, ui, &format!("Skill Points: {}/{} ({})", total, state.max_assignable, remaining), 0.0, w);
    } else {
        ui.text(format!("Skill Points: {}/{} ({} remaining)", total, state.max_assignable, remaining));
    }

    ui.spacing();
    ui.separator();
    ui.spacing();

    if state.extra_trait_options.len() > 0 {
        log_on_change!(state.extra_trait_options);
        let plural = if state.extra_trait_count == 1 {""} else {"s"};
        ui.text(format!(
            "Extra Tag Skill{} ({}/{}): select skill{}", plural, tags_traits, state.extra_trait_count, plural
        ));
        ui.spacing();

        let at_limit = tags_traits >= state.extra_trait_count;
        //basically we get a mutable reference to each of the skills so we can iterate over it, i have no idea if this works the way i think it does
        for (i, skill) in character.skills.mut_skill_block().iter_mut().enumerate() {
            //skip if it's not an extra tag from a trait
            if !state.extra_trait_options.contains(&i) {
                //make sure if it was previously tagged as a trait tag we clear that
                if skill.tagged == TagType::Trait {skill.tagged = TagType::None}
                continue
            }
            //only treat tag skills selected via a trait as chosen
            let is_chosen = skill.tagged == TagType::Trait;
            let is_forced = state.forced_trait && i == 14;
            let is_other = skill.tagged == TagType::Perk || skill.tagged == TagType::Standard;
            {
                let _g = (is_forced || is_other).then(|| ui.begin_disabled(true));
                let mut checked = is_chosen || is_forced || is_other;
                //basically, if we're not at our limit, show all options
                //if we are at our limit, only show the ones we have selected
                if checked || !at_limit {
                    if ui.checkbox(format!("{}##extratag_{}", SKILLS[i], i), &mut checked) {
                        if checked {
                            skill.tagged = TagType::Trait;
                            state.extra_tags.push(i);
                        } else if skill.tagged == TagType::Trait {
                        skill.tagged = TagType::None;
                            state.extra_tags.retain(|&x| x != i);
                        }
                        skill.update();
                    }
                }
            }
        }
        ui.spacing();
        ui.separator();
        ui.spacing();
    }
    ui.text("Tag Skills and Points:");
    ui.spacing();

    let col_ranks  = 175.0_f32;
    let col_tag    = 270.0_f32;
    let col_total  = 330.0_f32;
    let col_max    = 400.0_f32;

    ui.text_disabled("Skill");
    ui.same_line_with_pos(col_ranks);
    ui.text_disabled("Ranks");
    ui.same_line_with_pos(col_tag);
    ui.text_disabled("Tag");
    ui.same_line_with_pos(col_total);
    ui.text_disabled("Total");
    ui.same_line_with_pos(col_max);
    ui.text_disabled("Max");
    ui.separator();

    let _tag_limit = state.extra_trait_count + 3 + if character.has_perk(92) { 1 } else { 0 };
    //let at_tag_limit = character.skills.total_tags() <= tag_limit;
    let at_tag_limit = tags_standard >= 3;
    let forced_trait = character.has_trait(2);
    let forbidden_trait = character.has_trait(27);

    for (i, skill) in character.skills.mut_skill_block().iter_mut().enumerate() {
        let ranks = skill.ranks;
        let tagged = skill.is_tagged();
        let tag_bonus = if tagged { 2 } else { 0 };
        let max = skill.max;
        let input_max = max - tag_bonus;
        let total = skill.total;

        ui.text(SKILLS[i]);
        ui.same_line_with_pos(col_ranks);
        let can_dec = ranks > 0;
        let can_inc = ranks < input_max && state.available_points > 0;

        {
            let _dec = (!can_dec).then(|| ui.begin_disabled(true));
            if ui.button(format!("-##r_{}", i)) {
                skill.ranks -= 1;
                skill.update();
            }
        }
        ui.same_line();
        ui.text(format!("{:1}", ranks));
        ui.same_line();
        {
            let _inc = (!can_inc).then(|| ui.begin_disabled(true));
            if ui.button(format!("+##r_{}", i)) {
                skill.ranks += 1;
                skill.update();
            }
        }

        ui.same_line_with_pos(col_tag);
        let tag_overflow = ranks > (max - 2);
        let is_forced = forced_trait && i == 14;
        let is_forbidden = forbidden_trait && i == 10;
        let is_extra_tagged = skill.tagged == TagType::Perk || skill.tagged == TagType::Trait;
        let is_std_tagged = skill.tagged == TagType::Standard;
        //let is_extra_tagged = state.extra_tags.iter().any(|s| *s == i);
        let tag_disabled = is_forbidden || is_extra_tagged || is_forced || ((at_tag_limit || tag_overflow) && !is_std_tagged);
        //log_on_change!((tag_overflow, is_forced, is_forbidden, is_extra_tagged));
        {
            let _tg = tag_disabled.then(|| ui.begin_disabled(true));
            let mut tag_val = tagged;
            if ui.checkbox(format!("##tag_{}", i), &mut tag_val) {
                if !is_forced && !is_extra_tagged {
                    skill.tagged = if tag_val { TagType::Standard } else { TagType::None }
                }
            }
        }

        ui.same_line_with_pos(col_total);
        if tagged {
            render_text_wrapped(false, true, ui, &format!("{}", total), 0.0, w)
        } else {
            ui.text(format!("{}", total));
        }

        ui.same_line_with_pos(col_max);
        ui.text_disabled(format!("{}", max));

        if is_forbidden {
            ui.same_line();
            render_text_wrapped(true, false, ui, "[cannot tag]", 0.0, w);
        } else if is_forced {
            ui.same_line();
            render_text_wrapped(false, true, ui, "[forced]", 0.0, w);
        }
    }

    return h
}