//! Local device profile registry (JSON) and attached-device identity.
//!
//! Profiles map the stable USB serial of a Kindle to a friendly local name
//! ("Ken's Kindle", "Deb's Kindle"). Serials are stored locally only and
//! masked in logs/UI (see the project instructions §18). With at most one
//! device attached, the single USB Kindle is the session device.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::KindredError;
use crate::usb::discover_kindles;

/// A locally-named device profile keyed by stable USB serial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub serial: String,
    pub friendly_name: String,
    pub model: Option<String>,
    /// Unix seconds of the last time this device was profiled.
    pub last_seen_unix: Option<u64>,
}

/// A JSON-backed store of local device profiles.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: Vec<DeviceProfile>,
}

impl ProfileStore {
    /// Load the store from `path`, or return an empty store when absent.
    pub fn load(path: &Path) -> Result<ProfileStore, ProfileError> {
        match fs::read_to_string(path) {
            Ok(content) => Ok(serde_json::from_str(&content)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(ProfileStore::default())
            }
            Err(error) => Err(ProfileError::Io(error)),
        }
    }

    /// Persist the store to `path`, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<(), ProfileError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn lookup(&self, serial: &str) -> Option<&DeviceProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.serial == serial)
    }

    /// Friendly name for a serial, or a default when unprofiled.
    pub fn name_for(&self, serial: &str) -> String {
        self.lookup(serial)
            .map(|profile| profile.friendly_name.clone())
            .unwrap_or_else(|| "Unknown Kindle".to_owned())
    }

    /// Insert or replace the profile for a device.
    pub fn upsert(&mut self, profile: DeviceProfile) {
        if let Some(existing) = self
            .profiles
            .iter_mut()
            .find(|existing| existing.serial == profile.serial)
        {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }
}

/// The identity of the currently attached Kindle.
#[derive(Debug, Clone)]
pub struct DeviceIdentity {
    /// Stable USB serial (when the device exposes one).
    pub serial: Option<String>,
    /// Friendly model name (e.g. "Kindle Paperwhite (12th generation)").
    pub model: String,
    pub friendly_name: String,
}

/// Identify the currently attached Kindle from USB discovery plus the store.
pub fn identify_attached(store: &ProfileStore) -> Result<Option<DeviceIdentity>, KindredError> {
    let Some(kindle) = discover_kindles()?.into_iter().next() else {
        return Ok(None);
    };

    let serial = kindle.serial.clone();
    let friendly_name = serial
        .as_deref()
        .map(|serial| store.name_for(serial))
        .unwrap_or_else(|| "Unknown Kindle".to_owned());

    Ok(Some(DeviceIdentity {
        serial,
        model: kindle.model_name().to_owned(),
        friendly_name,
    }))
}

#[derive(Debug)]
pub enum ProfileError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileError::Io(error) => write!(f, "profile store I/O error: {error}"),
            ProfileError::Json(error) => write!(f, "profile store JSON error: {error}"),
        }
    }
}

impl std::error::Error for ProfileError {}

impl From<std::io::Error> for ProfileError {
    fn from(error: std::io::Error) -> Self {
        ProfileError::Io(error)
    }
}

impl From<serde_json::Error> for ProfileError {
    fn from(error: serde_json::Error) -> Self {
        ProfileError::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(serial: &str, name: &str) -> DeviceProfile {
        DeviceProfile {
            serial: serial.to_owned(),
            friendly_name: name.to_owned(),
            model: None,
            last_seen_unix: None,
        }
    }

    #[test]
    fn load_missing_file_returns_empty_store() {
        let dir = std::env::temp_dir();
        let path = dir.join("kindling_test_missing_profiles.json");
        let _ = std::fs::remove_file(&path);
        let store = ProfileStore::load(&path).unwrap();
        assert!(store.profiles.is_empty());
    }

    #[test]
    fn save_and_load_round_trips() {
        let path = std::env::temp_dir().join("kindling_test_profiles.json");
        let _ = std::fs::remove_file(&path);

        let mut store = ProfileStore::default();
        store.upsert(profile("GN433X11528401SG", "Ken's Kindle"));
        store.save(&path).unwrap();

        let loaded = ProfileStore::load(&path).unwrap();
        assert_eq!(loaded, store);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn name_for_falls_back_to_unknown() {
        let store = ProfileStore::default();
        assert_eq!(store.name_for("GN433X11528401SG"), "Unknown Kindle");
    }

    #[test]
    fn upsert_replaces_existing_profile() {
        let mut store = ProfileStore::default();
        store.upsert(profile("SN1", "Ken's Kindle"));
        store.upsert(profile("SN1", "Renamed Kindle"));
        assert_eq!(store.profiles.len(), 1);
        assert_eq!(store.name_for("SN1"), "Renamed Kindle");
    }
}
