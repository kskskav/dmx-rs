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