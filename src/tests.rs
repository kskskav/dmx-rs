/*! `mod tests` */

use super::*;

const TUPLE_CHOICES: &[(&str, &str)] = &[
    ("frogs", "Blue Winged Frogs"),
    ("toads", "Orange Scratchy Toads"),
    ("milk", "A Delicious Milkshake"),
    ("gob", "The Goblins and their King"),
];

const STR_CHOICES: &[&str] = &[
    "In the Court of the Crimson King",
    "Frog on Down to Froggington",
    "Down on the Upside",
    "Aries is SO MISERABLE (she's not joking...)",
];

#[test]
fn builtins() {
    let cfg = Dmx::default();
    let r = cfg.select("tuples", TUPLE_CHOICES).unwrap();
    println!("(tuple) Selected: {:?}", &r);

    let r = cfg.select("&strz", STR_CHOICES).unwrap();
    println!("(&str) Selected: {:?}", &r);
}

/*
This is the code from the README.
*/
#[test]
fn test_readme() {
    
    let choices: &[&str] = &[
        "Choice A",
        "Choice B",
        "Choice C",
        "Both A and B",
        "Both B and C",
        "All Three",
        "None of the Above",
    ];
    
    let dmx = Dmx::default();
    
    match dmx.select("Pick One:", choices).unwrap() {
        None => {
            println!("You declined to select an option.");
        },
        Some(n) => match choices.get(n) {
            None => {
                println!("You somehow chose an invalid choice.");
            },
            Some(choice) => {
                println!("You chose \"{}\".", choice);
            }
        }
    }
}

#[cfg(feature = "config")]
#[test]
fn test_config_file() {
    let dmx = Dmx::from_file("test/dmx_conf.toml").unwrap();
    match dmx.select(">", TUPLE_CHOICES).unwrap() {
        None => {
            println!("You chose [ dramatic pause ] NOTHING!!11");
        },
        Some(n) => match TUPLE_CHOICES.get(n) {
            Some(choice) => {
                println!("You chose \"{}\"", choice.1);
            },
            None => {
                println!("Not sure how you chose that.")
            }
        }
    }
    
}

/*
Code from the # Features section of the README.
*/
#[cfg(feature = "config")]
#[test]
fn readme_config() {
    const CHOICES: &[(&str, &str)] = &[
        ("frog", "a Fire-Breathing, Blue-Winged Frog"),
        ("toad", "a Acid-Blooded, Orange-Eyed Toad"),
        ("cat", "a Psychic Cat (Can Kill You With Its Mind)"),
        ("rat", "a Venom-Fanged Skaven Warlock"),
        ("dog", "Just a Regular Border Collie")
    ];
    
    let dmx = Dmx::from_file("test/dmx_conf.toml").unwrap();
    
    match dmx.select("->", CHOICES).unwrap() {
        None => {
            println!("You have chosen to adventure alone.");
        },
        Some(n) => {
            println!("You will be accompanied by {}", CHOICES[n].1);
        }
    }
}