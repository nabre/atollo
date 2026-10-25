//! No registry on macOS and Linux (`platform::registry`): every key reads
//! as missing, and nothing is ever written.

use anyhow::Result;

pub fn user_value(_key: &str, _name: &str) -> Option<String> {
    None
}

pub fn delete_user_key(_key: &str) -> Result<()> {
    Ok(())
}

pub fn set_user_value(_key: &str, _name: &str, _value: &str) -> Result<()> {
    Ok(())
}
