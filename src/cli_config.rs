use confy::ConfyError;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "aicom";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CliConfig {
    pub gemini_api_key: Option<String>,
}

impl CliConfig {
    pub fn load() -> Result<Self, ConfyError> {
        confy::load(APP_NAME, None)
    }

    pub fn save(&self) -> Result<(), ConfyError> {
        confy::store(APP_NAME, None, self)
    }
}
