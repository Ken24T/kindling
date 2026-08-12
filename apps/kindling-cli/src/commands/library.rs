use std::path::{Path, PathBuf};

use kindred::{
    LibraryRecord, LocalLibrary, ProfileStore, identify_attached, inventory_device,
    parse_book_filename,
};

fn library_path() -> PathBuf {
    if let Some(path) = std::env::var_os("KINDLING_STATE_DIR") {
        return PathBuf::from(path).join("library.json");
    }
    directories::ProjectDirs::from("", "", "kindling")
        .map(|dirs| dirs.data_local_dir().join("library.json"))
        .unwrap_or_else(|| PathBuf::from("library.json"))
}

fn profile_path() -> PathBuf {
    if let Some(path) = std::env::var_os("KINDLING_STATE_DIR") {
        return PathBuf::from(path).join("profiles.json");
    }
    directories::ProjectDirs::from("", "", "kindling")
        .map(|dirs| dirs.data_local_dir().join("profiles.json"))
        .unwrap_or_else(|| PathBuf::from("profiles.json"))
}

/// Directory holding user-supplied cover images (`<key>.<ext>`).
fn covers_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("KINDLING_LIBRARY_DIR") {
        return PathBuf::from(path).join("covers");
    }
    if let Some(path) = std::env::var_os("KINDLING_STATE_DIR") {
        return PathBuf::from(path).join("library").join("covers");
    }
    directories::ProjectDirs::from("", "", "kindling")
        .map(|dirs| dirs.data_local_dir().join("library").join("covers"))
        .unwrap_or_else(|| PathBuf::from("library/covers"))
}

fn status_label(record: &LibraryRecord) -> &'static str {
    match (record.on_device, record.local_path.is_some()) {
        (true, true) => "both",
        (true, false) => "on device",
        (false, _) => "local only",
    }
}

pub async fn list_library() -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let library = LocalLibrary::load(&path)?;

    if library.records.is_empty() {
        println!("Local library is empty ({}).", path.display());
        return Ok(());
    }

    println!("Local library ({}):", path.display());
    for record in &library.records {
        println!(
            "  {}  [{}]  {}",
            record.title.replace('_', " "),
            status_label(record),
            record.format.label(),
        );
    }
    println!("  ---");
    println!("  {} records", library.records.len());

    Ok(())
}

pub async fn reconcile_library() -> Result<(), Box<dyn std::error::Error>> {
    let Some(inventory) = inventory_device().await? else {
        eprintln!("No Kindle attached — cannot reconcile the library.");
        std::process::exit(1);
    };

    // Attach the device serial so records track which device held the book.
    let store = ProfileStore::load(&profile_path())?;
    let serial = identify_attached(&store)?
        .and_then(|identity| identity.serial)
        .or_else(|| {
            eprintln!(
                "Warning: no USB serial for the attached device; records will not track the device."
            );
            None
        });

    let path = library_path();
    let mut library = LocalLibrary::load(&path)?;
    let before = library.records.len();
    library.reconcile(&inventory, serial.as_deref());
    library.save(&path)?;

    let mut on_device = 0;
    let mut local_only = 0;
    let mut both = 0;
    for record in &library.records {
        match (record.on_device, record.local_path.is_some()) {
            (true, true) => both += 1,
            (true, false) => on_device += 1,
            (false, _) => local_only += 1,
        }
    }

    println!(
        "Reconciled library against the attached Kindle ({}):",
        path.display()
    );
    println!("  records:     {} (was {})", library.records.len(), before);
    println!("  on device:   {on_device}");
    println!("  local only:  {local_only}");
    println!("  both:        {both}");

    Ok(())
}

pub async fn add_library(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Path::new(path);
    let size_bytes = std::fs::metadata(file_path)
        .map_err(|error| format!("cannot read {path}: {error}"))?
        .len();
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("local path has no usable filename")?;

    let Some((title, asin, format)) = parse_book_filename(file_name) else {
        eprintln!("Cannot parse '{file_name}' as a Title_ASIN.ext book filename.");
        std::process::exit(1);
    };

    let library_path = library_path();
    let mut library = LocalLibrary::load(&library_path)?;
    let key = asin.unwrap_or_else(|| format!("dict:{title}"));
    library.upsert(LibraryRecord {
        key,
        title,
        format,
        size_bytes,
        local_path: Some(path.to_owned()),
        cover_path: None,
        on_device: false,
        last_seen_device: None,
    });
    library.save(&library_path)?;

    println!("Added '{file_name}' to the local library.");
    Ok(())
}

pub async fn scan_covers_cmd() -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let mut library = LocalLibrary::load(&path)?;
    let covers = covers_dir();
    let changed = library.scan_covers(&covers)?;
    library.save(&path)?;
    println!(
        "Scanned covers in {} — {} records updated.",
        covers.display(),
        changed
    );
    Ok(())
}

pub async fn list_collections() -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let library = LocalLibrary::load(&path)?;

    if library.collections.is_empty() {
        println!("No local collections ({}).", path.display());
        return Ok(());
    }

    println!("Local collections ({}):", path.display());
    for collection in &library.collections {
        println!(
            "  {}  [{} books]",
            collection.name,
            collection.book_keys.len()
        );
    }
    Ok(())
}

pub async fn collection_add(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let mut library = LocalLibrary::load(&path)?;
    library.create_collection(name)?;
    library.save(&path)?;
    println!("Created collection '{name}'.");
    Ok(())
}

pub async fn collection_rename(old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let mut library = LocalLibrary::load(&path)?;
    library.rename_collection(old, new)?;
    library.save(&path)?;
    println!("Renamed collection '{old}' to '{new}'.");
    Ok(())
}

pub async fn collection_delete(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let mut library = LocalLibrary::load(&path)?;
    library.delete_collection(name);
    library.save(&path)?;
    println!("Deleted collection '{name}'.");
    Ok(())
}

pub async fn collection_add_book(name: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let mut library = LocalLibrary::load(&path)?;
    library.add_book_to_collection(name, key)?;
    library.save(&path)?;
    println!("Added '{key}' to collection '{name}'.");
    Ok(())
}

pub async fn collection_remove_book(
    name: &str,
    key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = library_path();
    let mut library = LocalLibrary::load(&path)?;
    library.remove_book_from_collection(name, key)?;
    library.save(&path)?;
    println!("Removed '{key}' from collection '{name}'.");
    Ok(())
}
