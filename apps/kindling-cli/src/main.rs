mod commands;

use futures::executor::block_on;

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
    eprintln!("  copy-book <a> <d>  Copy a book (by ASIN) to a local directory");
    eprintln!("  add-book <path>    Copy a local file to the Kindle documents folder");
    eprintln!("  remove-book <asin> Remove a book (content + sidecar) by ASIN");
    eprintln!("  remove-added <h>   Remove a Kindling-controlled object by handle");
    eprintln!("  identify           Identify the attached Kindle (serial + profile)");
    eprintln!("  profiles           List local device profiles");
    eprintln!("  profile add <n>    Profile the attached Kindle with a friendly name");
}

fn main() {
    let result = match std::env::args().nth(1).as_deref() {
        Some("devices") => block_on(commands::devices()),
        Some("mtp-probe") => block_on(commands::mtp_probe()),
        Some("mtp-root") => block_on(commands::mtp_root()),
        Some("mtp-documents") => block_on(commands::mtp_documents()),
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
            block_on(commands::mtp_folder(handle))
        }
        Some("inventory") => block_on(commands::inventory()),
        Some("mtp-write-test") => block_on(commands::mtp_write_test()),
        Some("copy-book") => {
            let mut args = std::env::args().skip(2);
            match (args.next(), args.next()) {
                (Some(asin), Some(dest)) => block_on(commands::copy_book(&asin, &dest)),
                _ => {
                    eprintln!("Usage: kindling-cli copy-book <asin> <dest-dir>");
                    std::process::exit(1);
                }
            }
        }
        Some("add-book") => match std::env::args().nth(2) {
            Some(path) => block_on(commands::add_book(&path)),
            None => {
                eprintln!("Usage: kindling-cli add-book <path>");
                std::process::exit(1);
            }
        },
        Some("remove-book") => match std::env::args().nth(2) {
            Some(asin) => block_on(commands::remove_book_cmd(&asin)),
            None => {
                eprintln!("Usage: kindling-cli remove-book <asin>");
                std::process::exit(1);
            }
        },
        Some("remove-added") => {
            let handle = match std::env::args()
                .nth(2)
                .and_then(|arg| arg.parse::<u64>().ok())
            {
                Some(handle) => handle,
                None => {
                    eprintln!("Usage: kindling-cli remove-added <handle>");
                    std::process::exit(1);
                }
            };
            block_on(commands::remove_added(handle))
        }
        Some("identify") => block_on(commands::identify()),
        Some("profiles") => block_on(commands::list_profiles()),
        Some("profile") => {
            let mut args = std::env::args().skip(2);
            match (args.next().as_deref(), args.next()) {
                (Some("add"), Some(name)) => block_on(commands::add_profile(&name)),
                _ => {
                    eprintln!("Usage: kindling-cli profile add <name>");
                    std::process::exit(1);
                }
            }
        }
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
