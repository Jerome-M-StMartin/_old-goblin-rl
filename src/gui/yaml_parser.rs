// Written by: Jerome M. St.Martin
// Purpose: Adapter/Wrapper for the serde_yaml crate.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Card {
    name: String,
    cmc: u8,
    mana: String,
    txt: String,
}

pub fn run() -> Result<Vec<Card>, serde_yaml::Error> {
    let result = std::fs::File::open("../resources/AtomicCards.json");
    match result {
        Ok(file) => {
            let deserialized: Vec<Card> = serde_yaml::from_reader(file)?;
            return Ok(deserialized);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            panic!("Failed to load file: ../resources/AtomicCards.json");
        }
    }
}

pub fn to_string<T>(thing: &T) -> Result<String, serde_yaml::Error>
where
    T: Serialize,
{
    let yaml_str = serde_yaml::to_string(&thing)?; //Card into YAML text
    return Ok(yaml_str);
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T, serde_yaml::Error>
where
    T: for<'de> Deserialize<'de>,
{
    let t: T = serde_yaml::from_str::<T>(s)?; //from YAML into Card
    return Ok(t);
}

/////////////////////////
//////TESTING STUFF//////
/////////////////////////
struct Test<T>
where
    T: Serialize,
    T: for<'de> Deserialize<'de>,
{
    data: T,
}
impl<T> Test<T>
where
    T: Serialize,
    T: for<'de> Deserialize<'de>,
{
    fn new_int() -> Test<u32> {
        Test { data: 123456789 }
    }
    fn new_mtg() -> Test<Card> {
        let dark_ritual: Card = Card {
            name: "Dark Ritual".to_string(),
            cmc: 1,
            mana: "B".to_string(),
            txt: "Add [BBB] to your mana pool.".to_string(),
        };
        Test { data: dark_ritual }
    }
}

#[test]
fn to_string_test00() {
    let test: Test<u32> = Test::<u32>::new_int();
    let result = to_string(&test.data);
    match result {
        Ok(v) => {
            assert_eq!(v, "---\n123456789\n");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

#[test]
fn from_str_test01<'a>() {
    let test_str: &str = "blobs n' bits";
    let result = from_str::<String>(test_str);
    match result {
        Ok(v) => {
            assert_eq!(v, "blobs n' bits");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

#[test]
fn yaml_parse_test02() {
    let dark_ritual = Test::<Card>::new_mtg().data;
    let to_str_result = to_string(&dark_ritual);
    match to_str_result {
        Ok(v) => {
            let from_str_result = from_str(&v);
            match from_str_result {
                Ok(v) => {
                    let card: Card = v;
                    assert_eq!(card.name, dark_ritual.name);
                    assert_eq!(card.cmc, dark_ritual.cmc);
                    assert_eq!(card.mana, dark_ritual.mana);
                    assert_eq!(card.txt, dark_ritual.txt);
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
}
