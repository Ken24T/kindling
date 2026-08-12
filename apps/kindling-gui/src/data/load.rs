//! Load everything the UI shows: identity, inventory, local library.

use kindred::{
    DeviceIdentity, KindleInventory, LocalLibrary, ProfileStore, identify_attached,
    inventory_device,
};

use crate::data::paths::{library_path, profile_path};
use crate::model::DeviceInfo;

/// Result of a full refresh: attached-device identity, device inventory,
/// and the (reconciled) local library.
#[derive(Debug, Clone)]
pub struct LoadResult {
    pub device: Option<DeviceInfo>,
    pub inventory: Option<KindleInventory>,
    pub library: LocalLibrary,
    /// Friendly messages for non-fatal failures (e.g. device busy).
    pub errors: Vec<String>,
}

/// Load identity, inventory and library, reconciling the library on connect.
pub async fn load_all() -> LoadResult {
    let mut errors: Vec<String> = Vec::new();

    let store = load_profiles(&mut errors);
    let identity = load_identity(&store, &mut errors);
    let inventory = load_inventory(&identity, &mut errors).await;
    let mut library = load_library(&mut errors);
    reconcile_and_save(&mut library, &inventory, &identity, &mut errors);

    LoadResult {
        device: identity.map(into_device_info),
        inventory,
        library,
        errors,
    }
}

fn load_profiles(errors: &mut Vec<String>) -> ProfileStore {
    ProfileStore::load(&profile_path()).unwrap_or_else(|error| {
        errors.push(format!("profiles: {error}"));
        ProfileStore::default()
    })
}

fn load_identity(store: &ProfileStore, errors: &mut Vec<String>) -> Option<DeviceIdentity> {
    match identify_attached(store) {
        Ok(identity) => identity,
        Err(error) => {
            errors.push(format!("device: {error}"));
            None
        }
    }
}

async fn load_inventory(
    identity: &Option<DeviceIdentity>,
    errors: &mut Vec<String>,
) -> Option<KindleInventory> {
    if identity.is_none() {
        return None;
    }
    match inventory_device().await {
        Ok(inventory) => inventory,
        Err(error) => {
            errors.push(format!("inventory: {error}"));
            None
        }
    }
}

fn load_library(errors: &mut Vec<String>) -> LocalLibrary {
    LocalLibrary::load(&library_path()).unwrap_or_else(|error| {
        errors.push(format!("library: {error}"));
        LocalLibrary::default()
    })
}

/// Reconcile the library against the device, saving quietly on success.
fn reconcile_and_save(
    library: &mut LocalLibrary,
    inventory: &Option<KindleInventory>,
    identity: &Option<DeviceIdentity>,
    errors: &mut Vec<String>,
) {
    let (Some(inventory), Some(identity)) = (inventory, identity) else {
        return;
    };
    library.reconcile(inventory, identity.serial.as_deref());
    if let Err(error) = library.save(&library_path()) {
        errors.push(format!("library save: {error}"));
    }
}

/// The UI only ever sees the friendly name/model — never the raw serial.
fn into_device_info(identity: DeviceIdentity) -> DeviceInfo {
    DeviceInfo {
        friendly_name: identity.friendly_name,
        model: identity.model,
    }
}
