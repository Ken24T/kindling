use futures::executor::block_on;
use kindred::{
    discover_kindles, inventory_device, list_documents, list_folder_children, list_storage_roots,
    probe_first_mtp_device, run_controlled_transfer_test,
};

fn masked_serial(serial: &str) -> String {
    let chars: Vec<char> = serial.chars().collect();

    if chars.len() <= 8 {
        return serial.to_owned();
    }

    let start: String = chars.iter().take(4).collect();
    let end: String = chars.iter().rev().take(4).rev().collect();

    format!("{start}...{end}")
}

fn devices() -> Result<(), Box<dyn std::error::Error>> {
    let kindles = discover_kindles()?;

    if kindles.is_empty() {
        println!("No supported Kindle devices detected.");
        return Ok(());
    }

    for kindle in kindles {
        println!("Kindle detected");
        println!("Model:        {}", kindle.model_name());
        println!(
            "USB ID:       {:04x}:{:04x}",
            kindle.vendor_id, kindle.product_id
        );

        if let Some(manufacturer) = &kindle.manufacturer {
            println!("Manufacturer: {manufacturer}");
        }

        if let Some(product) = &kindle.product {
            println!("Product:      {product}");
        }

        if let Some(serial) = &kindle.serial {
            println!("Serial:       {}", masked_serial(serial));
        } else {
            println!("Serial:       <unavailable>");
        }

        println!(
            "Location:     bus {} device {}",
            kindle.bus_number, kindle.device_address
        );
        println!("Transport:    USB");
    }

    Ok(())
}

async fn mtp_probe() -> Result<(), Box<dyn std::error::Error>> {
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

async fn mtp_root() -> Result<(), Box<dyn std::error::Error>> {
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
                println!("[DIR]  {}", object.filename);
            } else {
                println!("[FILE] {}", object.filename);
            }
        }
        println!();
    }

    Ok(())
}

async fn mtp_documents() -> Result<(), Box<dyn std::error::Error>> {
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
            println!("[DIR]  {}", object.filename);
        } else {
            println!("[FILE] {}", object.filename);
        }
    }

    Ok(())
}

async fn mtp_folder(handle: u64) -> Result<(), Box<dyn std::error::Error>> {
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

async fn inventory() -> Result<(), Box<dyn std::error::Error>> {
    let inventory = inventory_device().await?;

    let Some(inventory) = inventory else {
        println!("No Kindle inventory found.");
        return Ok(());
    };

    println!("Storage: {}", inventory.storage_description);
    println!("Books:   {}", inventory.books.len());
    println!();

    for book in &inventory.books {
        println!("{}", book.title.replace('_', " "));
        println!(
            "  {} | {:.2} MB | asin: {}",
            book.format.label(),
            book.size_bytes as f64 / 1_000_000.0,
            book.asin.as_deref().unwrap_or("-"),
        );
        println!(
            "  sidecar: {} | metadata objects: {}",
            if book.sidecar_handle.is_some() {
                "yes"
            } else {
                "no"
            },
            book.metadata_handles.len(),
        );
        println!();
    }

    Ok(())
}

async fn mtp_write_test() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_controlled_transfer_test().await?;

    println!("Controlled transfer test:");
    println!("  folder:     {}", result.folder_name);
    println!(
        "  uploaded:   {} ({} bytes)",
        result.uploaded_file, result.uploaded_bytes
    );
    println!(
        "  readback:   {}",
        if result.verified_readback {
            "verified"
        } else {
            "MISMATCH"
        }
    );
    println!(
        "  cleaned up: {}",
        if result.cleaned_up { "yes" } else { "no" }
    );
    Ok(())
}

fn usage() {
    eprintln!("Usage: kindling-cli <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  devices         List connected supported Kindle devices");
    eprintln!("  mtp-probe       Open the first MTP device and inspect its storage");
    eprintln!("  mtp-root        List the root objects of each MTP storage");
    eprintln!("  mtp-documents   List the contents of the root documents folder");
    eprintln!("  mtp-folder <h>  List the contents of a folder by MTP handle");
    eprintln!("  inventory       List the device library as books");
    eprintln!("  mtp-write-test  Run the controlled upload/readback/cleanup test");
}

fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("devices") => devices(),
        Some("mtp-probe") => block_on(mtp_probe()),
        Some("mtp-root") => block_on(mtp_root()),
        Some("mtp-documents") => block_on(mtp_documents()),
        Some("mtp-folder") => {
            let handle = match std::env::args()
                .nth(2)
                .and_then(|arg| arg.parse::<u64>().ok())
            {
                Some(handle) => handle,
                None => {
                    eprintln!("Usage: kindling-cli mtp-folder <handle>");
                    std::process::exit(1);
                }
            };
            block_on(mtp_folder(handle))
        }
        Some("inventory") => block_on(inventory()),
        Some("mtp-write-test") => block_on(mtp_write_test()),
        _ => {
            usage();
            return;
        }
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
