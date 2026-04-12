use imgui::Ui;
use sdl2::video::Window;
use serde_json;
use crate::db::Db;
use crate::theme::{render_text_wrapped, render_window};
use crate::screens2::special_assignment::SPECIAL_LABELS;
use crate::character::{Character, MutantType, RobotType, Perk};

pub struct PerkState {

}
impl PerkState {
    pub fn new(db: &Db) -> Self {
        Self {

        }
    }
    pub fn is_complete(&self) -> bool {

    }
}

#[derive(Debug, Clone)]
pub struct PerkRow {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub level_req: i32,
    pub ranks: i32,
    pub rank_range: i32,
    pub reqs: Vec<String>,
    pub limits: Vec<String>,
    pub sourcebook: String,
}

