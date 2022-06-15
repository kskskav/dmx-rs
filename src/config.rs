/*!
Optional module for configuring `Dmx` structs with a configuration file.

The use case for this feature is a user having several programs that use
`dmenu` (like in a desktop environment); this allows the appearance of
`dmenu` in all of those programs to be configured with a single
configuration file.
*/
use std::path::PathBuf;

use serde::{Deserialize};

#[derive(Deserialize)]
pub struct ConfigFile {
    pub dmenu: Option<PathBuf>,
    pub font: Option<String>,
    pub normal_bg: Option<String>,
    pub normal_fg: Option<String>,
    pub select_bg: Option<String>,
    pub select_fg: Option<String>,
}

impl ConfigFile {
    pub fn from<S>(s: S) -> Result<ConfigFile, String>
    where
        S: AsRef<[u8]>,
    {
        let s = s.as_ref();
        let cfgfile = toml::from_slice(s)
            .map_err(|e| format!("Error deserializing Dmx config: {}", &e))?;
        Ok(cfgfile)
    }
}