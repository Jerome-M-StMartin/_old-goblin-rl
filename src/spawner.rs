use std::collections::HashMap;
use bracket_lib::prelude::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use specs::saveload::{SimpleMarker, MarkedBuilder};
use super::{ Stats, Player, Renderable, Name, Position, Viewshed, Hostile, BlocksTile, Rect,
             Item, Heals, Consumable, DamageOnUse, DamageAtom, Ranged,
             AoE, Confusion, SerializeMe, random_table::RandomTable, Equippable,
             EquipmentSlot, Weapon, BasicAttack, Resistances, BlocksAttacks, Menuable,
             Creature, Hunger, HungerState, MagicMapper, Useable, Throwable, Flammable,
             Hidden, EntryTrigger, TileType, Map, Info};

const MAX_MONSTERS: i32 = 4;

//Spawn player; return player entity.
pub fn player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('@'),
            fg: RGB::named(bracket_lib::prelude::YELLOW),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 0,
        })
        .with(Creature {})
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 5, dirty: true })
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
        .with(Hunger { state: HungerState::Satiated, clock: 300 })
        .with(Info::test_new())
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

pub fn spawn_room(map: &Map, rng: &mut RandomNumberGenerator, room: &Rect,
                    map_depth: i32, spawn_list: &mut Vec<(usize, String)>) {

    let mut possible_targets : Vec<usize> = Vec::new();
    { // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 + 1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(_map: &Map, rng: &mut RandomNumberGenerator, area : &[usize],
                        map_depth: i32, spawn_list : &mut Vec<(usize, String)>) {
    
    let spawn_table = room_table(map_depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();
    let mut areas : Vec<usize> = Vec::from(area);

    {// Scope to keep the borrow checker happy
        let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3);
        if num_spawns == 0 { return; }

        for _i in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 0usize }
            else { (rng.roll_dice(1, areas.len() as i32)-1) as usize };

            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(rng));
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

pub fn spawn_entity(ecs: &mut World, spawn: &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let width = map.width as usize;
    let x = (*spawn.0 % width) as i32;
    let y = (*spawn.0 / width) as i32;
    std::mem::drop(map);

    match spawn.1.as_ref() {
        "Door" => door(ecs, x, y),
        "Goblin" => goblin(ecs, x, y),
        "Orc" => orc(ecs, x, y),
        "Health Potion" => health_potion(ecs, x, y),
        "Fireball Scroll" => fireball_scroll(ecs, x, y),
        "Confusion Scroll" => confusion_scroll(ecs, x, y),
        "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
        "Scroll of Chitin" => barrier_scroll(ecs, x, y),
        "Knife" => knife(ecs, x, y),
        "Leather Armor" => leather_armor(ecs, x, y),
        "Longsword" => longsword(ecs, x, y),
        "Round Shield" => round_shield(ecs, x, y),
        "Magic Mapping Scroll" => magic_mapping_scroll(ecs, x, y),
        "Torch" => torch(ecs, x, y),
        "Flint" => flint(ecs, x, y),
        "Bear Trap" => bear_trap(ecs, x, y),
        _ => {}
    }
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", map_depth)
        .add("Health Potion", 2)
        .add("Fireball Scroll", map_depth)
        .add("Confusion Scroll", 0)
        .add("Magic Missile Scroll", 3)
        .add("Scroll of Chitin", 0)
        .add("Knife", 4 - map_depth)
        .add("Leather Armor", map_depth)
        .add("Longsword", map_depth)
        .add("Round Shield", map_depth)
        .add("Magic Mapping Scroll", map_depth)
        .add("Torch", 4)
        .add("Flint", 4)
        .add("Bear Trap", 4)
}

fn orc(ecs: &mut World, x: i32, y: i32) { hostile(ecs, x, y, bracket_lib::prelude::to_cp437('o'), "Orc"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { hostile(ecs, x, y, bracket_lib::prelude::to_cp437('g'), "Goblin"); }

fn door(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
        glyph: bracket_lib::prelude::to_cp437('+'),
        fg: RGB::named(bracket_lib::prelude::CHOCOLATE),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Door".to_string() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn hostile<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: bracket_lib::prelude::FontCharType, name: S) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(bracket_lib::prelude::RED),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 1,
        })
        .with(Creature {})
        .with(Hostile {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 5, dirty: true })
        .with(Name { name: name.to_string() })
        .with(Stats {max_hp: 4, hp: 4,
                     max_fp: 8, fp: 8,
                     max_mp: 2, mp: 2,
                     mind:1, body:1, soul:1})
        .with(BlocksTile {})
        .with(BasicAttack::default())
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(bracket_lib::prelude::ORANGE),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2})
        .with(Name {name: "Scroll of Fireball".to_string() })
        .with(Consumable {})
        .with(Ranged {range: 6})
        .with(Item {})
        .with(Useable { menu_name: "Read".to_string() })
        .with(DamageOnUse {dmg_atoms: vec![DamageAtom::Thermal(20)]})
        .with(AoE {radius: 3})
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: 173,
            fg: RGB::named(bracket_lib::prelude::MAGENTA),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2})
        .with(Name {name: "Health Potion".to_string() })
        .with(Useable { menu_name: "Drink".to_string() })
        .with(Consumable {})
        .with(Heals { duration: 1, amount: 8 })
        .with(Item {})
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(bracket_lib::prelude::MAGENTA),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2})
        .with(Name {name: "Scroll of Magic Missile".to_string() })
        .with(Consumable {})
        .with(Useable { menu_name: "Read".to_string() })
        .with(Ranged {range: 6})
        .with(Item {})
        .with(DamageOnUse {dmg_atoms: vec![DamageAtom::Bludgeon(8)]})
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(bracket_lib::prelude::PINK),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name{name: "Scroll of Confusion".to_string() })
        .with(Item {})
        .with(Useable { menu_name: "Read".to_string() })
        .with(Consumable {})
        .with(Ranged {range: 6})
        .with(Confusion {turns: 4})
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_mapping_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(bracket_lib::prelude::GREEN),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Scroll of Magic Mapping".to_string() })
        .with(Item{})
        .with(Useable { menu_name: "Read".to_string() })
        .with(MagicMapper{})
        .with(Consumable{})
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn barrier_scroll(ecs: &mut World, x: i32, y: i32) {
     ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(')'),
            fg: RGB::named(bracket_lib::prelude::CYAN),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name{name: "Scroll of Chitinflesh".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Useable { menu_name: "Read".to_string() })
        .with(Resistances {
            bludgeon: DamageAtom::Bludgeon(1),
            pierce: DamageAtom::Pierce(1),
            slash: DamageAtom::Slash(1),
            thermal: DamageAtom::Thermal(0) })
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

//Weapons----------------------------------------------------------
fn knife(ecs: &mut World, x: i32, y: i32) {
     ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('-'),
            fg: RGB::named(bracket_lib::prelude::GREY),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name{name: "Knife".to_string() })
        .with(Item {})
        .with(Throwable { dmg: DamageAtom::Pierce(4) })
        .with(Ranged {range: 4})
        .with(Equippable {slot: EquipmentSlot::LeftHand})
        .with(Weapon {primary: Some(DamageAtom::Slash(2)),
                      secondary: Some(DamageAtom::Pierce(1)),
                      tertiary: Some(DamageAtom::Bludgeon(0)) })
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable { glyph: bracket_lib::prelude::to_cp437('/'),
                           fg: RGB::named(bracket_lib::prelude::GREY),
                           bg: RGB::named(bracket_lib::prelude::BLACK),
                           render_order: 2 })
        .with(Item {})
        .with(Name { name: "Longsword".to_string() })
        .with(Equippable {slot: EquipmentSlot::LeftHand})
        .with(Weapon { primary: Some(DamageAtom::Slash(4)),
                       secondary: Some(DamageAtom::Pierce(4)),
                       tertiary: Some(DamageAtom::Bludgeon(1)) })
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
//----------------------------------------------------------/Weapons

fn leather_armor(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            //glyph: bracket_lib::prelude::to_cp437('¥'),
            glyph: 190,
            fg: RGB::named(bracket_lib::prelude::BROWN1),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name { name: "Leather Armor".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Armor })
        .with(Resistances {
            bludgeon: DamageAtom::Bludgeon(1),
            pierce: DamageAtom::Pierce(1),
            slash: DamageAtom::Slash(2),
            thermal: DamageAtom::Thermal(1) })
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn round_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: 10,
            fg: RGB::named(bracket_lib::prelude::BROWN1),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name { name: "Round Shield".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::RightHand })
        .with(BlocksAttacks { chance: 0.5, coverage: 2 })
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn torch(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437(';'),
            fg: RGB::named(bracket_lib::prelude::ORANGE),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name { name: "Torch".to_string() })
        .with(Item {})
        .with(Flammable {})
        .with(Equippable { slot: EquipmentSlot::RightHand })
        .with(Weapon { primary: Some(DamageAtom::Bludgeon(2)),
                       secondary: Some(DamageAtom::Thermal(0)),
                       tertiary: None, })
        .with(Throwable { dmg: DamageAtom::Bludgeon(2)  })
        .with(Ranged { range: 3 })
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn flint(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position {x, y})
        .with(Renderable {
            glyph: 96,
            fg: RGB::named(bracket_lib::prelude::GREY),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name { name: "Flint".to_string() })
        .with(Item {})
        .with(Ranged { range: 1 })
        .with(Useable { menu_name: "Spark".to_string() })
        .with(DamageOnUse {dmg_atoms: vec![DamageAtom::Thermal(0)]})
        .with(Menuable::default())
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

}

fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: bracket_lib::prelude::to_cp437('^'),
            fg: RGB::named(bracket_lib::prelude::RED),
            bg: RGB::named(bracket_lib::prelude::BLACK),
            render_order: 2
        })
        .with(Name { name : "Bear Trap".to_string() })
        .with(Hidden {})
        .with(EntryTrigger { repeatable: false, })
        .with(DamageOnUse { 
            dmg_atoms: vec![DamageAtom::Pierce(1), 
                            DamageAtom::Bludgeon(1)],
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
