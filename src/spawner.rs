use std::collections::HashMap;
use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};
use super::{ Stats, Player, Renderable, Name, Position, Viewshed, Monster, BlocksTile, Rect,
             map::MAPWIDTH, Item, Heals, Consumable, DamageOnUse, DamageAtom, Ranged,
             AoE, Confusion, SerializeMe, random_table::RandomTable, Equippable,
             EquipmentSlot, Weapon, BasicAttack, Resistances};

const MAX_MONSTERS: i32 = 4;

//Spawn player; return player entity.
pub fn player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: "Player".to_string() })
        .with(Stats {max_hp: 8,
                     hp: 8,
                     max_fp: 8,
                     fp: 8,
                     max_mp: 8,
                     mp: 8,
                     mind:1, body:1, soul:1})
        .with(BasicAttack::default())
        .with(Resistances::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room : &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();

    // Scope for borrow
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        for _i in 0 .. num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Confusion Scroll" => confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            "Scroll of Chitin" => barrier_scroll(ecs, x, y),
            "Knife" => knife(ecs, x, y),
            "Leather Armor" => leather_armor(ecs, x, y),
            _ => {}
        }
    }
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 9)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 4)
        .add("Fireball Scroll", 1 + map_depth)
        .add("Confusion Scroll", 0 + map_depth)
        .add("Magic Missile Scroll", 2)
        .add("Scroll of Chitin", 0 + map_depth)
        .add("Knife", 6 - map_depth)
        .add("Leather Armor", 6 - map_depth)
}

fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('o'), "Orc"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x, y, rltk::to_cp437('g'), "Goblin"); }

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Monster {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: name.to_string() })
        .with(Stats {max_hp: 4, hp: 4,
                     max_fp: 8, fp: 8,
                     max_mp: 2, mp: 2,
                     mind:1, body:1, soul:1})
        .with(BlocksTile {})
        .with(BasicAttack::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2})
        .with(Name {name: "Scroll of Fireball".to_string() })
        .with(Consumable {})
        .with(Ranged {range: 6})
        .with(Item {})
        .with(DamageOnUse {dmg_atoms: vec![DamageAtom::Thermal(20)]})
        .with(AoE {radius: 3})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2})
        .with(Name {name: "Health Potion".to_string() })
        .with(Consumable {})
        .with(Heals {heal_amount: 8})
        .with(Item {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2})
        .with(Name {name: "Scroll of Magic Missile".to_string() })
        .with(Consumable {})
        .with(Ranged {range: 6})
        .with(Item {})
        .with(DamageOnUse {dmg_atoms: vec![DamageAtom::Bludgeon(8)]})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{name: "Scroll of Confusion".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged {range: 6})
        .with(Confusion {turns: 4})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn barrier_scroll(ecs: &mut World, x: i32, y: i32) {
     ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{name: "Scroll of Chitinflesh".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Resistances {
            bludgeon: DamageAtom::Bludgeon(1),
            pierce: DamageAtom::Pierce(1),
            slash: DamageAtom::Slash(1),
            thermal: DamageAtom::Thermal(0) })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn knife(ecs: &mut World, x: i32, y: i32) {
     ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::GREY),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{name: "Knife".to_string() })
        .with(Item {})
        .with(Equippable {slot: EquipmentSlot::LeftHand})
        .with(Weapon {primary: Some(DamageAtom::Slash(4)),
                      secondary: Some(DamageAtom::Pierce(1)),
                      tertiary: Some(DamageAtom::Bludgeon(0)) })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn leather_armor(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: rltk::to_cp437('¥'),
            fg: RGB::named(rltk::BROWN1),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name {name: "Leather Armor".to_string()})
        .with(Item {})
        .with(Equippable {slot: EquipmentSlot::Armor})
        .with(Resistances {
            bludgeon: DamageAtom::Bludgeon(1),
            pierce: DamageAtom::Pierce(1),
            slash: DamageAtom::Slash(2),
            thermal: DamageAtom::Thermal(1) })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
