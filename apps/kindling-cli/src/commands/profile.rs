use std::path::PathBuf;

use kindred::{DeviceProfile, ProfileStore, discover_kindles, identify_attached};

fn profile_store_path() -> PathBuf {
    if let Some(path) = std::env::var_os("KINDLING_STATE_DIR") {
        return PathBuf::from(path).join("profiles.json");
    }
    directories::ProjectDirs::from("", "", "kindling")
        .map(|dirs| dirs.data_local_dir().join("profiles.json"))
        .unwrap_or_else(|| PathBuf::from("profiles.json"))
}

fn masked_serial(serial: &str) -> String {
    let chars: Vec<char> = serial.chars().collect();

    if chars.len() <= 8 {
        return serial.to_owned();
    }

    let start: String = chars.iter().take(4).collect();
    let end: String = chars.iter().rev().take(4).rev().collect();

    format!("{start}...{end}")
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub async fn identify() -> Result<(), Box<dyn std::error::Error>> {
    let store = ProfileStore::load(&profile_store_path())?;

    let Some(identity) = identify_attached(&store)? else {
        println!("No Kindle attached.");
        return Ok(());
    };

    match &identity.serial {
        Some(serial) => {
            println!(
                "Attached: {}  ({} · serial {})",
                identity.friendly_name,
                identity.model,
                masked_serial(serial)
            );
            if identity.friendly_name == "Unknown Kindle" {
                println!("No profile for this device yet — use 'profile add <name>'.");
            }
        }
        None => println!("Attached: {} (no serial exposed)", identity.friendly_name),
    }

    Ok(())
}

pub async fn list_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let store = ProfileStore::load(&profile_store_path())?;

    if store.profiles.is_empty() {
        println!(
            "No device profiles stored ({}).",
            profile_store_path().display()
        );
        return Ok(());
    }

    println!("Stored profiles ({}):", profile_store_path().display());
    for profile in store.profiles {
        println!(
            "  {}  (serial {})",
            profile.friendly_name,
            masked_serial(&profile.serial)
        );
    }

    Ok(())
}

pub async fn add_profile(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let kindles = discover_kindles()?;
    let Some(kindle) = kindles.into_iter().next() else {
        eprintln!("No Kindle attached — cannot create a profile.");
        std::process::exit(1);
    };
    let Some(ref serial) = kindle.serial else {
        eprintln!("Attached Kindle exposes no USB serial; cannot profile it.");
        std::process::exit(1);
    };

    let path = profile_store_path();
    let mut store = ProfileStore::load(&path)?;
    store.upsert(DeviceProfile {
        serial: serial.clone(),
        friendly_name: name.to_owned(),
        model: Some(kindle.model_name().to_owned()),
        last_seen_unix: Some(now_unix()),
    });
    store.save(&path)?;

    println!(
        "Profiled the attached Kindle as '{name}' ({})",
        path.display()
    );
    Ok(())
}
