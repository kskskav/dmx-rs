# `dm_x`

A Rust crate for using [`dmenu`](https://tools.suckless.org/dmenu/), a
keyboard-driven menu originally written for use with tiling window managers.

Using this crate involves implementing the `Item` trait for your type,
and then passing a slice of those to the `Dmx::select()` method.

`Item` is already implemented for `&str`, so the following should work:

```rust
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
```

See the `examples/` for non-trivial implementation of `Item`.