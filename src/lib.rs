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


Obviously, you are going to need `dmenu` installed (and a system where it
can run).

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

# Features

`dm_x` has one optional feature, `config`, which provides the ability to
deserialize a `Dmx` configuration from some .toml. This gets
[`serde`](https://serde.rs) (and [`toml`](https://crates.io/crates/toml))
involved, which is kind of a large dependency for an otherwise
dependency-free (save the `dmenu` binary) crate (hence the feature gate).

```
#[cfg(feature = "config")]
{
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
```

Examination of the included "test/dmx_conf.toml" file should make the
required format fairly obvious; all values are optional (defaulting to
the values provided by `Dmx::default()`) and should be just as `dmenu`
would expect them as arguments.

File "test/dmx_conf.toml":

```toml
dmenu     = "/usr/bin/dmenu"
font      = "Terminus-12"
normal_bg = "#88cccc"
normal_fg = "#422"
select_bg = "#422"
select_fg = "#88cccc"
```

*/

#![feature(doc_cfg)]

use std::io::{Read, Write};
use std::path::PathBuf;
#[cfg(feature = "config")]
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(feature = "config")]
mod config;

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
            "10",
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
    
    /**
    Return a `Dmx` configured by a slice of bytes.
    */
    #[doc(cfg(feature = "config"))]
    #[cfg(feature = "config")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Dmx, String> {
        let cfgf = config::ConfigFile::from(&bytes)?;
        
        let mut dmx = Dmx::default();
        if let Some(dmenu_path) = cfgf.dmenu {
            dmx.dmenu = dmenu_path;
        }
        if let Some(font) = cfgf.font {
            dmx.font = font;
        }
        if let Some(nbg) = cfgf.normal_bg {
            dmx.normal_bg = nbg;
        }
        if let Some(nfg) = cfgf.normal_fg {
            dmx.normal_fg = nfg;
        }
        if let Some(sbg) = cfgf.select_bg {
            dmx.select_bg = sbg;
        }
        if let Some(sfg) = cfgf.select_fg {
            dmx.select_fg = sfg;
        }
        
        Ok(dmx)
    }
    
    /**
    Return a `Dmx` configured based on a configuration file.
    */
    #[doc(cfg(feature = "config"))]
    #[cfg(feature = "config")]
    pub fn from_file<P>(p: P) -> Result<Dmx, String>
    where
        P: AsRef<Path>,
    {
        let p = p.as_ref();
        let bytes = std::fs::read(p)
            .map_err(|e| format!("Error reading from \"{}\": {}", p.display(), &e))?;
        Dmx::from_bytes(&bytes)
    }
    
    /**
    Return a `Dmx` based on a byte slice containing some TOML.
    */
    #[doc(cfg(feature = "config"))]
    #[cfg(feature = "config")]
    pub fn from_slice(b: &[u8]) -> Result<Dmx, String> {
        Dmx::from_bytes(b)
    }
    
    /**
    Configure "automagically".
    
    That is, attempt to configure from (in this order):
      * the file specified by the `$DMX_CONFIG` environment variable
      * the file at `$XDG_CONFIG_HOME/dmx.toml`
      * the file at `$HOME/.config/dmx.toml`
      * `Dmx::default()` (this always works)
    */
    #[doc(cfg(feature = "config"))]
    #[cfg(feature = "config")]
    pub fn automagiconf() -> Dmx {
        use std::env::var;
        
        if let Ok(path) = var("DMX_CONFIG") {
            if let Ok(dmx) = Dmx::from_file(path) {
                return dmx;
            }
        }
        
        if let Ok(config_path) = var("XDG_CONFIG_HOME") {
            let mut config_file = PathBuf::from(config_path);
            config_file.push("dmx.toml");
            if let Ok(dmx) = Dmx::from_file(&config_file) {
                return dmx;
            }
        }
        
        if let Ok(home_dir) = var("HOME") {
            let mut config_file = PathBuf::from(home_dir);
            config_file.push(".config");
            config_file.push("dmx.toml");
            if let Ok(dmx) = Dmx::from_file(&config_file) {
                return dmx;
            }
        }
        
        Dmx::default()
    }
}

#[cfg(test)]
mod tests;
