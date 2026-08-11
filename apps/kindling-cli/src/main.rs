use futures::executor::block_on;
use kindred::{discover_kindles, probe_first_mtp_device};

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

fn usage() {
    eprintln!("Usage: kindling-cli <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  devices     List connected supported Kindle devices");
    eprintln!("  mtp-probe   Open the first MTP device and inspect its storage");
}

fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("devices") => devices(),
        Some("mtp-probe") => block_on(mtp_probe()),
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
