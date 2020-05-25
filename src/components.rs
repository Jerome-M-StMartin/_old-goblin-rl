use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};

#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct Player {}


#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive(Component, Debug)]
pub struct Name {
    pub name: String
}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub max_fp: i32,
    pub fp: i32,
    pub max_mp: i32,
    pub mp: i32,
    pub defense: i32,
    pub power: i32
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity
}

#[derive(Component, Debug)]
pub struct Damage {
    pub amount: Vec<i32>
}

impl Damage {
    pub fn new_damage(storage: &mut WriteStorage<Damage>, target: Entity, amount: i32) {
        if let Some(damage_queue) = storage.get_mut(target) {
            damage_queue.amount.push(amount);
        } else {
            let damage = Damage { amount: vec![amount] };
            storage.insert(target, damage).expect("Unable to store damage on target.");
        }
    }
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount: i32
}
