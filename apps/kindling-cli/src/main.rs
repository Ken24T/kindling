use kindred::discover_kindles;

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
        println!();
        println!("Model:      {}", kindle.model_name());
        println!(
            "USB ID:     {:04x}:{:04x}",
            kindle.vendor_id, kindle.product_id
        );

        if let Some(manufacturer) = &kindle.manufacturer {
            println!("Manufacturer: {manufacturer}");
        }

        if let Some(product) = &kindle.product {
            println!("Product:    {product}");
        }

        if let Some(serial) = &kindle.serial {
            println!("Serial:     {}", masked_serial(serial));
        } else {
            println!("Serial:     <unavailable>");
        }

        println!(
            "Location:   bus {} device {}",
            kindle.bus_number, kindle.device_address
        );
        println!("Transport:  USB");
    }

    Ok(())
}

fn usage() {
    eprintln!("Usage: kindling-cli <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  devices    List connected supported Kindle devices");
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("devices") => {
            if let Err(error) = devices() {
                eprintln!("Error: {error}");
                std::process::exit(1);
            }
        }
        _ => usage(),
    }
}
