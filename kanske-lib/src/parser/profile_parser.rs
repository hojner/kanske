use std::{str::FromStr, sync::Arc};

use crate::{AppResult, KanskeError};

pub struct Profile {
    pub name: Arc<str>,
}

impl FromStr for Profile {
    type Err = KanskeError;

    fn from_str(s: &str) -> AppResult<Self> {
        let mut parts = s.split_whitespace();
        if parts.next() != Some("profile") {
            return Err(KanskeError::ParsedStringUnexpectedFormat(
                "profile keyword".to_string(),
            ));
        }
        let name: Arc<str> = parts
            .next()
            .map(Arc::from)
            .ok_or_else(|| KanskeError::ParsedStringUnexpectedFormat("profile name".to_string()))?;
        Ok(Self { name })
    }
}
