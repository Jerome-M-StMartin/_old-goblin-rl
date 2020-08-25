use specs::prelude::*;
use specs_derive::*;
use rltk::RGB;
use serde::{Serialize, Deserialize};
use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use std::ops::{Add, Sub};

//serialization helper code. Each component that contains an Entity must impl ConvertSaveload.
pub struct SerializeMe;

//special component for serializing game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map
}

//Does the Component macro know to use NullStorage?...
//-------------Fieldless Components-----------------------
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hostile {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Creature {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Bleeding {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Flammable {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Aflame {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct JustMoved {}
//--------------------------------------

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Door { 
    pub open: bool 
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {
    pub repeatable: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Lightsource {
    pub is_lit: bool,
    pub radius: i32,
}

impl Default for Lightsource {
    fn default() -> Self {
        Lightsource {
            is_lit: false,
            radius: 10,
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Useable {
    pub menu_name: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Throwable {
    pub dmg: DamageAtom,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Healing {
    pub duration: i32,
    pub amount: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32
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

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Particle {
    pub lifetime_ms: f32,
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

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Heals {
    //item component, not to be confused with "Healing"
    //This CAUSES Healing when this item is used.
    pub duration: i32,
    pub amount: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum HungerState { Stuffed, Satiated, Hungry, Famished, Starving }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hunger {
    pub state: HungerState,
    pub clock: i32,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack { //item component
    pub owner: Entity
}

/*#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
pub enum BuffAtom {
    Dodge,
    Block,
    Resistance,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Buff { //item or creature component
    pub buff_type: BuffAtom,
    pub buff_value: i32,
}*/
    
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    LeftHand,
    RightHand,
    Helm,
    Armor,
    Boots,
    Gloves,
    Necklace,
    Ring,
    Back
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
pub struct Equippable { //item component
    pub slot: EquipmentSlot
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Equipped { //item component
    pub owner: Entity,
    pub slot: EquipmentSlot
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Weapon { //item component
    //Attack Modes
    pub primary: Option<DamageAtom>,
    pub secondary: Option<DamageAtom>,
    pub tertiary: Option<DamageAtom>,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct BlocksAttacks {
    pub chance: f32,
    pub coverage: u8
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct BasicAttack { //creature component
    base: DamageAtom,
    pub current: DamageAtom
}

impl BasicAttack {    
    pub fn modify(storage: &mut WriteStorage<BasicAttack>, target: Entity, delta: DamageAtom) {
        if let Some(basic_attack) = storage.get_mut(target) {
            basic_attack.current = delta;
        } else {
            eprintln!("Tried to modify BasicAttack of ent with no such component.");
        }
    }

    pub fn reset(storage: &mut WriteStorage<BasicAttack>, target: Entity) {
        if let Some(basic_attack) = storage.get_mut(target) {
            basic_attack.current = basic_attack.base;
        } else {
            eprintln!("Tried to reset BasicAttack of ent with no such component.");
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

//------------------------Damage, Resistance, & Immunity Components-----------------------------
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DamageAtom { 
    Bludgeon(i32),
    Pierce(i32),
    Slash(i32),
    Thermal(i32),
    Bleed,
    Poison,
    Starvation,
    Suffocation,
    Venom
}

impl DamageAtom {
    pub fn value(&self) -> i32 {
        match self {
            DamageAtom::Bludgeon(val) => return *val,
            DamageAtom::Pierce(val) => return *val,
            DamageAtom::Slash(val) => return *val,
            DamageAtom::Thermal(val) => return *val,
            _ => return 1,
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

#[derive(Component, Debug, ConvertSaveload, Clone, Copy)]
pub struct Immunities {
    pub bleed: DamageAtom,
    pub poison: DamageAtom,
    pub starvation: DamageAtom,
    pub suffocation: DamageAtom,
    pub venom: DamageAtom,
}

#[derive(Component, Debug, ConvertSaveload, Clone, Copy)]
pub struct Resistances {
    pub bludgeon: DamageAtom,
    pub pierce: DamageAtom,
    pub slash: DamageAtom,
    pub thermal: DamageAtom
}

impl Add for Resistances {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            bludgeon: DamageAtom::Bludgeon( self.bludgeon.value() + other.bludgeon.value() ),
            pierce: DamageAtom::Pierce( self.pierce.value() + other.pierce.value() ),
            slash: DamageAtom::Slash( self.slash.value() + other.slash.value() ),
            thermal: DamageAtom::Thermal( self.thermal.value() + other.thermal.value() ) 
        }
    }
}

impl Sub for Resistances {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            bludgeon: DamageAtom::Bludgeon( self.bludgeon.value() - other.bludgeon.value() ),
            pierce: DamageAtom::Pierce( self.pierce.value() - other.pierce.value() ),
            slash: DamageAtom::Slash( self.slash.value() - other.slash.value() ),
            thermal: DamageAtom::Thermal( self.thermal.value() - other.thermal.value() ) 
        }
    }
}

impl Default for Resistances {
    fn default() -> Resistances {
        Resistances {
            bludgeon: DamageAtom::Bludgeon(0),
            pierce: DamageAtom::Pierce(0),
            slash: DamageAtom::Slash(0),
            thermal: DamageAtom::Thermal(0)
        }
    }

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
    pub item: Entity,
    pub desired_by: Entity
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct UseItemIntent {
    pub item: Entity,
    pub target: Option<rltk::Point>
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct ThrowIntent {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct DropItemIntent {
    pub item: Entity
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct EquipIntent {
    pub item: Entity
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct UnequipIntent {
    pub item: Entity
}//----------------------------------------------

#[derive(Component, Debug, ConvertSaveload)]
pub struct Info {
    pub name: String,
    //pub stats: ?,
    //pub actions: Vec<(Action, String)>, //refactor of MenuOption & Menuable
    pub actions: Vec<(MenuOption, String)>, //refactor of MenuOption & Menuable
    pub desc: String,
    pub lore: Vec<String>,
}

impl Info {
    pub fn test_new() -> Info {
        Info {
            name: "[Name Goes Here]".to_string(),
            actions: vec![(MenuOption::Attack, "Action A".to_string()),
                          (MenuOption::PickUp, "Action B".to_string()),
                          (MenuOption::Equip, "Action C".to_string())],
            desc: "Watch - as I combine all the juice from the mind, heel up, wheel up, bring it back come rewind...".to_string(),
            lore: Vec::new(),
        }
    }
}

/*#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
enum Action {
    PickUp,
    DropIt,
    Use,
    Equip,
    Unequip,
    Attack,
    Throw,
}*/

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MenuOption {
    PickUp,
    DropIt,
    Use,
    Equip,
    Unequip,
    Attack,
    Throw,
    //Examine
}

//Provides a good vessel for learning how to allow a system to run less than once-per-frame.
#[derive(Debug, Component, ConvertSaveload, Clone)]
pub struct Menuable {
    pub options: Vec<(MenuOption, String)>,
}

impl Default for Menuable {
    fn default() -> Menuable {
        Menuable {
            options: Vec::<(MenuOption, String)>::new(), 
        }
    }
}

//--------------Behavior/AI Components-----------
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Behavior {
    behavior_bitmask: u64,
    //In yet-unwritted BehaviorSystem, create tree to
    //continuously query the bitmask for behaviors, in
    //order of priority. The tree itself determines
    //each bit's/behavior's priority, not the order of
    //bits in the mask.
}
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Home { pub home: rltk::Point }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Fear { pub bit: u64 }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Terror { pub bit: u64 }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Inquisitive { pub bit: u64 } //Huh? What was that noise?

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Alert { pub bit: u64 } // !

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Berzerk { pub bit: u64 }

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hungry { pub bit: u64 }
//-----------------------------------------------
