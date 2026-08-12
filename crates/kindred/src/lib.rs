mod collections;
mod covers;
mod error;
mod inventory;
mod library;
mod manage;
mod mtp;
mod profile;
mod transfer;
mod usb;

pub use collections::LocalCollection;
pub use error::KindredError;

pub use inventory::{Book, BookFormat, KindleInventory, inventory_device, parse_book_filename};

pub use library::{BookStatus, LibraryError, LibraryRecord, LocalLibrary};

pub use manage::{add_book_to_kindle, copy_book_from_kindle, remove_added_object, remove_book};

pub use mtp::{
    MtpDocumentsListing, MtpObjectSummary, MtpProbe, MtpStorageListing, MtpStorageSummary,
    download_object, list_documents, list_folder_children, list_storage_roots,
    probe_first_mtp_device,
};

pub use profile::{DeviceIdentity, DeviceProfile, ProfileError, ProfileStore, identify_attached};

pub use transfer::{TransferTestResult, run_controlled_transfer_test};

pub use usb::{AMAZON_VENDOR_ID, PAPERWHITE_12_PRODUCT_ID, UsbKindle, discover_kindles};
