use kindred::{
    download_object, list_documents, list_folder_children, list_storage_roots,
    probe_first_mtp_device,
};

pub async fn mtp_probe() -> Result<(), Box<dyn std::error::Error>> {
    let probe = probe_first_mtp_device().await?;

    println!("MTP device opened");
    println!("Manufacturer: {}", probe.manufacturer);
    println!("Model:        {}", probe.model);

    for storage in probe.storages {
        println!();
        println!("Storage:      {}", storage.description);
        println!(
            "Capacity:     {:.2} GB",
            storage.total_capacity_bytes as f64 / 1_000_000_000.0
        );
        println!(
            "Free space:   {:.2} GB",
            storage.free_space_bytes as f64 / 1_000_000_000.0
        );
    }

    Ok(())
}

pub async fn mtp_root() -> Result<(), Box<dyn std::error::Error>> {
    let listings = list_storage_roots().await?;

    if listings.is_empty() {
        println!("No MTP storage found.");
        return Ok(());
    }

    for listing in listings {
        println!("Storage: {}", listing.description);
        println!();
        for object in listing.objects {
            if object.is_folder {
                println!("[DIR]  handle {:<10} {}", object.handle, object.filename);
            } else {
                println!("[FILE] handle {:<10} {}", object.handle, object.filename);
            }
        }
        println!();
    }

    Ok(())
}

pub async fn mtp_documents() -> Result<(), Box<dyn std::error::Error>> {
    let listing = list_documents().await?;

    let Some(listing) = listing else {
        println!("No 'documents' folder found in any storage root.");
        return Ok(());
    };

    println!("Storage:        {}", listing.description);
    println!("Documents dir:  handle {}", listing.documents_handle);
    println!();
    for object in listing.objects {
        if object.is_folder {
            println!("[DIR]  handle {:<10} {}", object.handle, object.filename);
        } else {
            println!("[FILE] handle {:<10} {}", object.handle, object.filename);
        }
    }

    Ok(())
}

pub async fn mtp_folder(handle: u64) -> Result<(), Box<dyn std::error::Error>> {
    let listing = list_folder_children(handle).await?;

    let Some(listing) = listing else {
        println!("No storage found for handle {handle}.");
        return Ok(());
    };

    println!("Storage: {}", listing.description);
    println!("Folder:   handle {handle}");
    println!();
    for object in listing.objects {
        if object.is_folder {
            println!("[DIR]  handle {:<10} {}", object.handle, object.filename);
        } else {
            println!("[FILE] handle {:<10} {}", object.handle, object.filename);
        }
    }

    Ok(())
}

/// Download an object's bytes by handle to a local file (read-only).
pub async fn mtp_getfile(handle: u64, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = download_object(handle).await?;
    std::fs::write(dest, &bytes)?;
    println!(
        "Downloaded handle {handle} ({} bytes) to {dest}",
        bytes.len()
    );
    Ok(())
}
