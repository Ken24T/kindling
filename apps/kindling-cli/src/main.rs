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
    eprintln!("  mtp-getfile <h> <dest>  Download an object by MTP handle to a local file");
    eprintln!("  inventory       List the device library as books");
    eprintln!("  mtp-write-test  Run the controlled upload/readback/cleanup test");
    eprintln!("  copy-book <a> <d>  Copy a book (by ASIN) to a local directory");
    eprintln!("  add-book <path>    Copy a local file to the Kindle documents folder");
    eprintln!("  remove-book <asin> Remove a book (content + sidecar) by ASIN");
    eprintln!("  remove-added <h>   Remove a Kindling-controlled object by handle");
    eprintln!("  identify           Identify the attached Kindle (serial + profile)");
    eprintln!("  profiles           List local device profiles");
    eprintln!("  profile add <n>    Profile the attached Kindle with a friendly name");
    eprintln!("  library            List the local library");
    eprintln!("  library reconcile  Reconcile the library against the attached Kindle");
    eprintln!("  library add <p>    Add a local book file to the local library");
    eprintln!("  library collections  List local collections");
    eprintln!(
        "  library collection <add|rename|delete|add-book|remove-book> ...  Manage local collections"
    );
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
        Some("mtp-getfile") => {
            let mut args = std::env::args().skip(2);
            let (handle, dest) = match (args.next(), args.next()) {
                (Some(handle), Some(dest)) => match handle.parse::<u64>() {
                    Ok(handle) => (handle, dest),
                    Err(_) => {
                        eprintln!("Usage: kindling-cli mtp-getfile <handle> <dest>");
                        std::process::exit(1);
                    }
                },
                _ => {
                    eprintln!("Usage: kindling-cli mtp-getfile <handle> <dest>");
                    std::process::exit(1);
                }
            };
            block_on(commands::mtp_getfile(handle, &dest))
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
        Some("library") => {
            let mut args = std::env::args().skip(2);
            match args.next().as_deref() {
                None => block_on(commands::list_library()),
                Some("reconcile") => block_on(commands::reconcile_library()),
                Some("add") => match args.next() {
                    Some(path) => block_on(commands::add_library(&path)),
                    None => {
                        eprintln!("Usage: kindling-cli library add <path>");
                        std::process::exit(1);
                    }
                },
                Some("collections") => block_on(commands::list_collections()),
                Some("covers") => block_on(commands::scan_covers_cmd()),
                Some("collection") => {
                    let mut sub = args;
                    match (sub.next().as_deref(), sub.next(), sub.next()) {
                        (Some("add"), Some(name), _) => block_on(commands::collection_add(&name)),
                        (Some("rename"), Some(old), Some(new)) => {
                            block_on(commands::collection_rename(&old, &new))
                        }
                        (Some("delete"), Some(name), _) => {
                            block_on(commands::collection_delete(&name))
                        }
                        (Some("add-book"), Some(name), Some(key)) => {
                            block_on(commands::collection_add_book(&name, &key))
                        }
                        (Some("remove-book"), Some(name), Some(key)) => {
                            block_on(commands::collection_remove_book(&name, &key))
                        }
                        _ => {
                            eprintln!(
                                "Usage: kindling-cli library collection [add <name>|rename <old> <new>|delete <name>|add-book <name> <key>|remove-book <name> <key>]"
                            );
                            std::process::exit(1);
                        }
                    }
                }
                Some(other) => {
                    eprintln!("Unknown library subcommand '{other}'.");
                    eprintln!(
                        "Usage: kindling-cli library [reconcile|add <path>|collections|collection ...]"
                    );
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
