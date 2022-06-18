/*!
An example of a `dmenu`-driven program launcher in Rust.

Launch with
```bash
cargo run --example launcher launcher.json
```

This is mainly meant to be a demonstration of implementing `Item` in a
nontrivial way, and not an actual system program as you would use it,
so there are a lot of `.unwrap()`s and `.expect()`s instead of actual
error handling.

The launcher menu has a heirarchical, nested structure, much like a file
system. At any given level, the options will be a mix of programs to
launch and categories; selecting a program obviously launches that program,
while selecting a category opens a new menu, displaying the items in that
category (which themselves could be a mix of programs and categories).

The `Entry` type for which we implement `Item` is an enum with two variants:
one which contains a `MenuItem` struct to represent a launchable program,
and the other which contains a `MenuDir` struct which represents a category
and contains its own `Vec` of entries.
*/

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use dm_x::*;

/**
String used to seprate levels of heirarchy in the launcher menu, much like
the directory separator in a filesystem path; the second element in the
tuple is the length of the separator (in `char`s).

These are in a `OnceCell` because we'd like them to be theoretically
configurable, instead of hard-coded.
*/
static SEPARATOR: OnceCell<(String, usize)> = OnceCell::new();

/*
These two functions return the length of the separator and a reference to
the actual separator itself.

These aren't strictly necessary, but they reduce the visual noise where
the separator is being used.
*/
fn sep_width() -> usize {
    SEPARATOR.get().expect("uninitialized separator").1
}
fn sep_str() -> &'static str {
    SEPARATOR.get().expect("uninitialized separator").0.as_str()
}

/**
Represents a launchable program in the launcher menu.

The following `MenuItem`:

```
MenuItem {
    key: "mail".to_string(),
    desc: "Open Gmail in Chromium".to_string(),
    exec: [
        "/usr/bin/chromium".to_string(),
        "https://mail.google.com".to_string()
    ]
}
```

will be displayed in the menu thus:

```text
mail     Open Gmail in Chromium
```

and launch `/usr/bin/chromium` with `https://mail.google.com` as a
command-line argument.

However, this program never instantiates these directly; they get
deserialized from a menu configuration file. The above `MenuItem`
would appear in that file as

```json
{
    "key": "mail",
    "desc": "Open Gemail in Chromium",
    "exec": ["/usr/bin/chromium", "https://mail.google.com"]
}
```

See the file `launcher.json` for more examples.
*/
#[derive(Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// easily-typeable key
    pub key: String,
    /// verbose description
    pub desc: String,
    /// command and command line arguments to execute
    pub exec: Vec<String>,
}

/**
Represents a category submenu in the launcher menu.

The following `MenuDir`:

```
MenuDir {
    key: "ssh".to_string(),
    desc: "Common Secure Shell Connections".to_string(),
    items: vec![
        Entry::Item(MenuItem {
            key: "mine".to_string(),
            desc: "me@mydomain.net".to_string(),
            exec: [
                "x-terminal-emulator".to_string(), "-e".to_string(),
                "ssh".to_string(), "me@mydomain.net".to_string()
            ]
        }),
        Entry::Item(MenuItem {
            key: "work".to_string(),
            desc: "flastname@workdomain.com".to_string(),
            exec: [
                "x-terminal-emulator".to_string(), "-e".to_string(),
                "ssh".to_string(), "flastname@workdomain.net".to_string(),
                "-p".to_string(), "2222".to_string()
            ]
        }),
        Entry::Item(MenuItem {
            key: "pi".to_string(),
            desc: "Raspberry Pi on Local Netowrk".to_string(),
            exec: [
                "x-terminal-emulator".to_string(), "-e".to_string(),
                "ssh".to_string(), "me@192.168.1.31".to_string()
            ]
        }),
    ]
}
```

will look thus in the `dmenu` dropdown:

```text
ssh /  Common Secure Shell Connections
```

When selected it will open a new menu that looks like this:

```text
ssh/
     mine  me@mydomain.net
     work  flastname@workdomain.com
     pi    Raspberry Pi on Local Network
```

Each of those entries will open a new terminal window and ssh to the
appropriate target.

However, this program never directly instantiates these; they are
deserialized from a configuration file. In that file, the above
would appear thus:

```json
{
    "key": "ssh",
    "desc" "Common Secure Shell Connections",
    "items": [
        {
            "key": "mine",
            "desc": "me@mydomain.net",
            "exec": ["x-terminal-emulator", "-e", "ssh", "me@mydomain.net"]
        },
        {
            "key: "work",
            "desc": "flastname@workdomain.com",
            "exec": [
                "x-terminal-emulator", "-e", "ssh",
                "flastname@workdomain.net", "-p", "2222"
            ]
        },
        {
            "key": "pi",
            "desc": "Raspberry Pi on Local Network",
            "exec": "x-terminal-emulator", "-e", "ssh", "me@192.168.1.31"]
        }
    ]
}
```

See the file `launcher.json` for more examples.
*/
#[derive(Serialize, Deserialize)]
pub struct MenuDir {
    /// easiy-typeable key
    pub key: String,
    /// verbose description
    pub desc: String,
    /// list of submenu items
    pub items: Vec<Entry>,
}

/**
The `Entry` enum is the type that actually implements `Item`.

It is never explicitly instantiated, or even written in the menu
configuration file. Because of the way the `#[serde(untagged)]`
directive works, the deserializer will just pick the proper
variant based on whether it sees a `MenuItem` or a `MenuDir`.

See the descriptions of `MenuItem` and `MenuDir`, and also the
`launcher.json` file for examples.
*/
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    Item(MenuItem),
    Dir(MenuDir),
}

impl Item for Entry {
    /// This method is the same for both variants; they both just return
    /// the number of `chars` in their `key`s.
    fn key_len(&self) -> usize {
        match self {
            Entry::Item(m) => m.key.chars().count(),
            Entry::Dir(d) => d.key.chars().count(),
        }
    }

    /// The `MenuDir` includes the `SEPARATOR` after its key (so the user
    /// can tell it's a subcategory), while the `MenuItem` just has more space.
    fn line(&self, key_len: usize) -> Vec<u8> {
        match self {
            Entry::Item(m) => format!(
                "{:key_width$}  {}\n",
                &m.key,
                &m.desc,
                key_width = key_len + sep_width()
            )
            .into_bytes(),
            Entry::Dir(d) => format!(
                "{:key_width$}{}  {}\n",
                &d.key,
                sep_str(),
                &d.desc,
                key_width = key_len
            )
            .into_bytes(),
        }
    }
}

/**
Load the data file, which must be specified as the first argument on the
command line.

Attempts to deserialize the data file into (and return) a `Vec<Entry>`.
*/
fn load_data_file() -> Vec<Entry> {
    use std::ffi::OsString;

    let fname: OsString = std::env::args_os()
        .nth(1)
        .expect("You must specify a data file as the argument.");
    let f = std::fs::File::open(&fname).unwrap();
    let v: Vec<Entry> = serde_json::from_reader(&f).unwrap();

    v
}

/**
This function launches `dmenu` repeatedly until the user either chooses a
`MenuItem` or cancels from the top level menu.
*/
fn recursive_select(dmx: &Dmx, prompt: &str, items: &[Entry]) -> Option<MenuItem> {
    loop {
        match dmx.select(prompt, items).unwrap() {
            // This will cancel the process if returned from the highest-level
            // menu, or re-display the next-higher-level menu if returned from
            // below.
            None => return None,
            Some(n) => match &items[n] {
                // If the user selects an item, return that; it will bubble up
                // the stack of calls to `recursive_select()` and get returned
                // to `main()`.
                Entry::Item(m) => return Some(m.clone()),
                // If the user selects a subcategory, call this function on
                // the entries in that subcategory.
                Entry::Dir(d) => {
                    let new_prompt = format!("{}{}{}", prompt, &d.key, sep_str());
                    // If the lower-level call returns a `MenuItem`, bubble
                    // that back up the stack.
                    //
                    // Implicitly, too, because `Dmx::select()` is being called
                    // in a `loop`, if the user cancels at the next lowest
                    // level (and this call to `recursive_select()`)
                    // returns `None`, the current level's category will
                    // be redisplayed.
                    if let Some(m) = recursive_select(dmx, &new_prompt, &d.items) {
                        return Some(m);
                    }
                }
            },
        }
    }
}

/**
Launch a program from the given `chunks` of command line.

The `chunks` will be a reference to the `exec` `Vec` from a `MenuItem`.

This program is meant as an example of implementing (and using) the `Item`
trait, but this particular function is kind of tricky and worth paying
attention to, also.
*/
fn exec<S: AsRef<str>>(chunks: &[S]) -> ! {
    use std::ffi::CString;
    use std::os::raw::c_char;

    // Turn our command line chunks into a `Vec` of `CString`s. (These are
    // null-terminated byte slices.)
    let args: Vec<CString> = chunks
        .iter()
        .map(|c| CString::new(c.as_ref().as_bytes()).unwrap())
        .collect();
    // Now create a `Vec` of _pointers_ to our `CString`s.
    let mut arg_ptrs: Vec<*const c_char> = args.iter().map(|a| a.as_ptr()).collect();
    // Now terminate our pointer `Vec` with a null pointer, because that's how
    // libc's `execvp()` knows where the end is.
    arg_ptrs.push(std::ptr::null());
    // Now instantiate a pointer to our null-terminated array of pointers to
    // null-terminated arrays of bytes. This is how `execvp()` needs it.
    let argv: *const *const c_char = arg_ptrs.as_ptr();

    // Now here's the tricky part that I screwed up at first: The second
    // argument to `execvp()` needs to be the pointer to the array of pointers.
    // The _first_ argument needs to be _the first pointer in that array_.
    // That particular value gets passed to this function twice, once
    // as the first argument, and then once again as the first element
    // of the array pointed to by the second argument. If you do this wrong
    // you'll get segfaults.
    let res = unsafe { libc::execvp(arg_ptrs[0], argv) };

    // `execvp()` shouldn't return, so we'll panic whether it returns an
    // error or not.
    if res < 0 {
        panic!("Error executing: {}", &res);
    } else {
        panic!("Exec... returned for some reason?");
    }
}

fn main() {
    let items = load_data_file();
    
    // In an actual program, these next two lines would probably be
    // accompanied by some configuration in order to customize the
    // appearance of `dmenu`.
    #[cfg(not(feature = "config"))]
    let dmx = Dmx::default();
    #[cfg(feature = "config")]
    let dmx = Dmx::automagiconf();
    SEPARATOR.set(("/".to_owned(), 1)).unwrap();

    match recursive_select(&dmx, "", &items) {
        None => {
            println!("Nothing selected!");
        }
        Some(m) => {
            exec(&m.exec);
        }
    }
}
