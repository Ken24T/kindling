use std::path::Path;

use kindred::{
    Book, KindleInventory, add_book_to_kindle, copy_book_from_kindle, inventory_device,
    remove_added_object, remove_book, run_controlled_transfer_test,
};

pub async fn inventory() -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn mtp_write_test() -> Result<(), Box<dyn std::error::Error>> {
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

fn find_book<'a>(inventory: &'a KindleInventory, asin: &str) -> Option<&'a Book> {
    inventory
        .books
        .iter()
        .find(|book| book.asin.as_deref() == Some(asin))
}

pub async fn copy_book(asin: &str, dest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let Some(inventory) = inventory_device().await? else {
        eprintln!("No Kindle inventory found.");
        std::process::exit(1);
    };
    let Some(book) = find_book(&inventory, asin) else {
        eprintln!("No book with ASIN {asin} in the inventory.");
        std::process::exit(1);
    };
    let dest = copy_book_from_kindle(book, Path::new(dest_dir)).await?;
    println!(
        "Copied '{}' to {}",
        book.title.replace('_', " "),
        dest.display()
    );
    Ok(())
}

pub async fn add_book(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let handle = add_book_to_kindle(Path::new(path)).await?;
    println!("Added {path} to the Kindle documents folder (object handle {handle}).");
    Ok(())
}

pub async fn remove_book_cmd(asin: &str) -> Result<(), Box<dyn std::error::Error>> {
    let Some(inventory) = inventory_device().await? else {
        eprintln!("No Kindle inventory found.");
        std::process::exit(1);
    };
    let Some(book) = find_book(&inventory, asin) else {
        eprintln!("No book with ASIN {asin} in the inventory.");
        std::process::exit(1);
    };
    println!("Removing from Kindle:");
    println!("  title:    {}", book.title.replace('_', " "));
    println!(
        "  format:   {} | {:.2} MB",
        book.format.label(),
        book.size_bytes as f64 / 1_000_000.0
    );
    println!(
        "  sidecar:  {}",
        if book.sidecar_handle.is_some() {
            "yes"
        } else {
            "no"
        }
    );
    println!("  metadata: {} objects", book.metadata_handles.len());
    remove_book(book).await?;
    println!("Removed.");
    Ok(())
}

pub async fn remove_added(handle: u64) -> Result<(), Box<dyn std::error::Error>> {
    remove_added_object(handle).await?;
    println!("Removed Kindling-controlled object {handle}.");
    Ok(())
}
