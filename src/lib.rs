/*!
`dm_x` is a library to facilitate using
[`dmenu`](https://tools.suckless.org/dmenu/) as a way for a user to select
from a list of options.

`dmenu` is a great tool for keyboard-driven environments, as it allows
selection of options in a way that's faster and more convenient than
clicking on an item in a list, or selecting from a drop-down. This small
library offers three things when it comes to using this tool:

  * A convenient way to interact with `dmenu`---both in feeding it options
    and in reporting which was selected.
  * Separating the _text_ of an option offered by `dmenu` from the
    _programmatinc function_ of that option.
  * A mechanism for providing the user with an easily-typeable
    mnemonic along with more descriptive option text (both of which are,
    again, distinct from the function of the option in question).

```
# use dm_x::Dmx;
static REGRESSION_TYPES: &[(&str, &str)] = &[
    ("linear", "Linear Regression"),
    ("p2", "Quadratic Regression"),
    ("p3", "Cubic Regression"),
    ("log", "Logistic Regression"),
    ("ln", "Logarithmic Regression"),
    ("sin", "Sinusoidal Regression")
];

let dmx = Dmx::default();

match dmx.select("method:", REGRESSION_TYPES).unwrap() {
    Some(n) => {
        println!("{}", REGRESSION_TYPES[n].1);
    },
    None => {
        println!("No regression type chosen.");
    },
}

```
*/
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const NEWLINE: u8 = b'\n';

/**
Implement this trait for any types you want to use as `dmenu` selectors.

The original use case for the whole `dm_x` idea was as a wrapper around
`dmenu` to allow more descriptive options than just the text of each
choice. Part of the idea was that each option would have a "verbose"
description, but then also start with an easy-to-type "key" for fast
selection with the keyboard.

So wrapping `dmenu` this way to implement a program launcher might look
like

```text
ff       Firefox Web Browser
geany    Geany Text Editor
pragha   Pragha Music Player
soffice  LibreOffice Suite
term     Alacritty Terminal Emulator
vlc      VLC Media Player
wx       Current Local Weather
```

The `Item::key_len()` function is meant to return the length of the
"key" string, so that the length of the longest key can be passed
to the `Item::line()` function, so it can format format its line
nicely, so all the "verbose" elements line up.

See the implementation of `Item` for two-tuples of `AsRef<str>` for
a concrete example that may be explanatory.
*/
pub trait Item {
    /**
    Return the length of this `Item`'s "key". If your type's formatting
    scheme doesn't have a "key" portion or care about its length, then
    this function's return value doesn't matter.
    */
    fn key_len(&self) -> usize;

    /**
    Format the text of this `Item`'s option line as it should be displayed
    in `dmenu`. `Dmx::select()` will call `Item::key_len()` on each item
    in the slice of `Item`s passed to it; the largest of these values will
    then be passed as the `key_len` argument when it calls `Item::line()`
    to generate each `Item`'s dmenu line.
    */
    fn line(&self, key_len: usize) -> Vec<u8>;
}

/**
The reference, more-or-less "originally-intended" implementation of this
trait. Each tuple is treated as a `("key", "verbose description")` pair,
and formatted thus.

To get the choices from the trait description presented to you, you could
pass the following slice to `Dmx::select()`:

```
let items = &[
    ("ff", "Firefox Web Browser"),
    ("geany", "Geany Text Editor"),
    ("pragha", "Pragha Music Player"),
    // ...
    ("wx", "Current Local Weather"),
];
```
*/
impl<T, U> Item for (T, U)
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    fn key_len(&self) -> usize {
        self.0.as_ref().chars().count()
    }

    fn line(&self, key_len: usize) -> Vec<u8> {
        format!(
            "{:kwidth$}  {}\n",
            &self.0.as_ref(),
            &self.1.as_ref(),
            kwidth = key_len
        )
        .into_bytes()
    }
}

/**
The most basic possible implementation, this just presents a list of
options verbatim with no "key" business or special formatting or
any of that jazz.
*/
impl Item for &str {
    fn key_len(&self) -> usize {
        0
    }
    fn line(&self, _: usize) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

/**
This struct contains all the arguments necessary to pass to `dmenu` on the
command line.
*/
pub struct Dmx {
    /// Path to the `dmenu` binary. If it's in your system's `$PATH`, the
    /// default value of `"dmenu"` should work fine.`
    pub dmenu: PathBuf,
    /// Font to use, in xls or xfontsel format, depending on what your version
    /// of `dmenu` supports.
    pub font: String,
    /// item background color
    pub normal_bg: String,
    /// item foreground color
    pub normal_fg: String,
    /// selected item background color
    pub select_bg: String,
    /// selected item foreground color
    pub select_fg: String,
}

impl std::default::Default for Dmx {
    fn default() -> Self {
        Dmx {
            dmenu: "dmenu".into(),
            font: "LiberationMono-12".to_owned(),
            normal_bg: "#222".to_owned(),
            normal_fg: "#aaa".to_owned(),
            select_bg: "#888".to_owned(),
            select_fg: "#aff".to_owned(),
        }
    }
}

impl Dmx {
    /*
    Generate a `Command` to pass to `dmenu`.
    */
    fn cmd(&self, prompt: &str, n_items: usize) -> Command {
        let mut c = Command::new(&self.dmenu);
        c.args([
            "-l",
            &n_items.to_string(),
            "-p",
            prompt,
            "-fn",
            &self.font,
            "-nb",
            &self.normal_bg,
            "-nf",
            &self.normal_fg,
            "-sb",
            &self.select_bg,
            "-sf",
            &self.select_fg,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

        c
    }

    /**
    Launch `dmenu` to select an `Item`.

    Returns the slice index of the `Item` selected, or `None` if cancelled.
    */
    pub fn select<S, I>(&self, prompt: S, items: &[I]) -> Result<Option<usize>, String>
    where
        S: AsRef<str>,
        I: Item,
    {
        let klen: usize = items.iter().map(|x| x.key_len()).max().unwrap_or(0);

        let output: Vec<Vec<u8>> = items
            .iter()
            .map(|x| {
                let mut v = x.line(klen);
                if Some(&NEWLINE) == v.last() {
                    v
                } else {
                    v.push(NEWLINE);
                    v
                }
            })
            .collect();

        let mut child = self
            .cmd(prompt.as_ref(), output.len())
            .spawn()
            .map_err(|e| format!("Unable to launch dmenu: {}", &e))?;

        {
            let mut stdin = child.stdin.take().unwrap();
            for line in output.iter() {
                stdin
                    .write_all(line)
                    .map_err(|e| format!("Error writing to dmenu subprocess: {}", &e))?;
            }
            stdin
                .flush()
                .map_err(|e| format!("Error writing to dmenu subprocess: {}", &e))?;
        }

        let mut stdout = child.stdout.take().unwrap();
        child
            .wait()
            .map_err(|e| format!("dmenu subprocess returned error: {}", &e))?;
        let mut choice_bytes: Vec<u8> = Vec::new();
        let _ = stdout
            .read_to_end(&mut choice_bytes)
            .map_err(|e| format!("Error reading dmenu output: {}", &e))?;

        for (n, line) in output.iter().enumerate() {
            if *line == choice_bytes {
                return Ok(Some(n));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
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
}
