use std::{path::PathBuf, sync::LazyLock};

use crate::args::Arguments;

/// The folder name for any specific folders we create.
const BACKEND_IDENTIFIER: &str = "ghr_backend";

/// Where to store the configuration.
#[allow(unused)]
static CONF_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("Failed to find data directory for server to use.")
        .join(BACKEND_IDENTIFIER)
});

/// The configuration for the backend.
#[derive(Clone, Debug)]
pub struct Config {
    // postgres stuff.
    // TODO: document these fields when i know more about postgres lmao
    pub postgres_host: &'static str,
    pub postgres_user: &'static str,
}

impl From<Arguments> for Config {
    fn from(value: Arguments) -> Self {
        Self {
            postgres_host: value.postgres_host.leak(),
            postgres_user: value.postgres_user.leak(),
        }
    }
}

static mut CONF: Option<Config> = None;

pub(crate) fn init(clap_args: Arguments) {
    // SAFETY: No other threads are going to access the data before it is
    // initialized.
    if unsafe { CONF.clone() }.is_none() {
        // SAFETY: if anyone tried to access the config right now, it would be
        // `None`, and the program would safely panic.
        //
        // Nonetheless, we don't want that either! So avoid grabbing it before
        // now.
        unsafe { CONF = Some(Config::from(clap_args)) };
    }
}

pub(crate) fn config() -> Config {
    // SAFETY: we only mutate the config once, when starting the program.
    //
    // As such, we know that the config won't change underneath us.
    unsafe { CONF.clone() }.unwrap()
}
