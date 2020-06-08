use specs::prelude::*;
use specs_derive::*;
use rltk::RGB;
use serde::{Serialize, Deserialize};
use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use std::collections::HashMap;

//serialization helper code. Each component that contains an Entity must impl ConvertSaveload.
pub struct SerializeMe;

//special component for serializing game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map
}

//Should these be using NullStorage?...
//-------------Fieldless Components-----------------------
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}//--------------------------------------

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name { //universal component
    pub name: String
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Heals { //effect component
    pub heal_amount: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged { //item or effect component
    pub range: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AoE { //effect component
    pub radius: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Confusion {//effect component
    pub turns: i32
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    LeftHand,
    RightHand,
    Head,
    Neck,
    Torso,
    Waist,
    Legs,
    Feet,
    Hands,
    Finger1,
    Finger2,
    Finger3,
    Finger4
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable { //item component
    pub slot: EquipmentSlot
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Equipped { //item component
    pub owner: Entity,
    pub slot: EquipmentSlot
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Unequipped {} //temp flag to signal that unequip logic is required in EquipSystem

//    !!! NOT  SAVEABLE, seems like ConvertSaveload doesn't work with HashMap<_, Entity>?  !!!
#[derive(Component, Debug, Clone)]
pub struct EquippedMap { //creature component
    map: HashMap<EquipmentSlot, Entity>
}

impl EquippedMap {
    pub fn equip(storage: &mut WriteStorage<EquippedMap>,
                 wearer: Entity, slot: EquipmentSlot, equippable: Entity) -> Option<Entity> {
        
        if let Some(equips_map) = storage.get_mut(wearer) {
            equips_map.map.insert(slot, equippable)
        } else {
            //------------------> capacity must match # of EquipmentSlot enum variants.
            let mut new_equipped_map = EquippedMap {map: HashMap::<EquipmentSlot, Entity>::with_capacity(13)};
            new_equipped_map.map.insert(slot, equippable);
            storage.insert(wearer, new_equipped_map).expect("Failed to store EquippedMap.");
            None
        }
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Weapon { //item component
    //Attack Modes
    pub primary: Option<DamageAtom>,
    pub secondary: Option<DamageAtom>,
    pub tertiary: Option<DamageAtom>,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack { //item component
    pub owner: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Stats { //creature component
    //Resources
    pub max_hp: i32,
    pub hp: i32,
    pub max_fp: i32,
    pub fp: i32,
    pub max_mp: i32,
    pub mp: i32,

    //Attributes
    pub mind: i32,
    pub body: i32,
    pub soul: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct BasicAttack { //creature component
    base: DamageAtom,
    current: DamageAtom
}

impl BasicAttack {    
    pub fn modify(storage: &mut WriteStorage<BasicAttack>, target_ent: Entity, delta: DamageAtom) {
        if let Some(basic_attack) = storage.get_mut(target_ent) {
            basic_attack.current = delta;
        } else {
            eprintln!("Tried to modify BasicAttack on ent with no such component.");
        }
    }
    pub fn reset(storage: &mut WriteStorage<BasicAttack>, target_ent: Entity) {
        if let Some(basic_attack) = storage.get_mut(target_ent) {
            basic_attack.current = basic_attack.base;
        } else {
            eprintln!("Tried to modify BasicAttack on ent with no such component.");
        }
    }
}

impl Default for BasicAttack {
    fn default() -> BasicAttack {
        BasicAttack {
            base: DamageAtom::Bludgeon(1),
            current: DamageAtom::Bludgeon(1)
        }
    }
}



//------------------------Damage & Resistance Components--------------------------------------
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DamageAtom { //each instance represents an amount of one damage type 
    Bludgeon(i32),
    Pierce(i32),
    Slash(i32),
    Thermal(i32)
}

impl DamageAtom {
    pub fn value(&self) -> i32 {
        match self {
            DamageAtom::Bludgeon(val) => return *val,
            DamageAtom::Pierce(val) => return *val,
            DamageAtom::Slash(val) => return *val,
            DamageAtom::Thermal(val) => return *val,
        }
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct DamageQueue {
    pub queue: Vec<DamageAtom>
}

impl DamageQueue {
    pub fn queue_damage(storage: &mut WriteStorage<DamageQueue>, target: Entity, dmg: DamageAtom) {
        if let Some(damage_queue) = storage.get_mut(target) {
            damage_queue.queue.push(dmg);
        } else {
            let damage_queue = DamageQueue {queue: vec![dmg]};
            storage.insert(target, damage_queue).expect("Unable to store DamageQueue.");
        }
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Resistances {
    pub bludgeon: DamageAtom,
    pub pierce: DamageAtom,
    pub slash: DamageAtom,
    pub thermal: DamageAtom
}

impl Resistances {
    pub fn modify(&mut self, delta: &Resistances) {
        self.bludgeon =
            DamageAtom::Bludgeon(self.bludgeon.value() + delta.bludgeon.value());
        self.pierce =
            DamageAtom::Pierce(self.pierce.value() + delta.pierce.value());
        self.slash =
            DamageAtom::Slash(self.slash.value() + delta.slash.value());
        self.thermal =
            DamageAtom::Thermal(self.thermal.value() + delta.thermal.value());
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ResistanceDeltas {//similar to intent, removed each tick after application
   pub queue: Vec<DamageAtom>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct DamageOnUse { //rename of InflictsDamage
    pub dmg_atoms: Vec<DamageAtom>
}//---------------------------------------------------------------------------------

//--------------Intent Components----------------
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct MeleeIntent{
    pub target: Entity
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct PickUpIntent {
    pub object: Entity,
    pub desired_by: Entity
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct UseItemIntent {
    pub item: Entity,
    pub target: Option<rltk::Point>
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct DropItemIntent {
    pub item: Entity
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct EquipIntent {
    pub wearer: Entity
}//----------------------------------------------
