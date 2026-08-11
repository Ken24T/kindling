use mtp_rs::mtp::MtpDevice;

#[derive(Debug, Clone)]
pub struct MtpStorageSummary {
    pub description: String,
    pub total_capacity_bytes: u64,
    pub free_space_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct MtpProbe {
    pub manufacturer: String,
    pub model: String,
    pub storages: Vec<MtpStorageSummary>,
}

pub async fn probe_first_mtp_device() -> Result<MtpProbe, mtp_rs::Error> {
    let device = MtpDevice::open_first().await?;

    let device_info = device.device_info();

    let manufacturer = device_info.manufacturer.clone();
    let model = device_info.model.clone();

    let mut storage_summaries = Vec::new();

    for storage in device.storages().await? {
        let info = storage.info();

        storage_summaries.push(MtpStorageSummary {
            description: info.description.clone(),
            total_capacity_bytes: info.total_capacity,
            free_space_bytes: info.free_space,
        });
    }

    Ok(MtpProbe {
        manufacturer,
        model,
        storages: storage_summaries,
    })
}

/// Summary of a single MTP object (file or folder), owned by Kindred.
#[derive(Debug, Clone)]
pub struct MtpObjectSummary {
    pub filename: String,
    pub is_folder: bool,
    /// Opaque MTP object handle used to address the object in later operations.
    pub handle: u64,
    /// Size in bytes (0 for folders, or when the device reports none).
    pub size_bytes: u64,
}

impl From<mtp_rs::mtp::ObjectInfo> for MtpObjectSummary {
    fn from(info: mtp_rs::mtp::ObjectInfo) -> Self {
        let is_folder = info.is_folder();
        MtpObjectSummary {
            filename: info.filename,
            is_folder,
            handle: info.handle.0,
            size_bytes: info.size,
        }
    }
}

/// Objects directly inside one folder of an MTP storage.
#[derive(Debug, Clone)]
pub struct MtpStorageListing {
    pub description: String,
    pub objects: Vec<MtpObjectSummary>,
}

/// Open the first MTP device and enumerate the root of each storage.
///
/// `None` as the parent denotes the storage root in the `mtp-rs` API.
pub async fn list_storage_roots() -> Result<Vec<MtpStorageListing>, mtp_rs::Error> {
    let device = MtpDevice::open_first().await?;

    let mut listings = Vec::new();

    for storage in device.storages().await? {
        let info = storage.info();
        let objects = storage.list_objects(None).await?;

        listings.push(MtpStorageListing {
            description: info.description.clone(),
            objects: objects.into_iter().map(MtpObjectSummary::from).collect(),
        });
    }

    Ok(listings)
}

/// Contents of the root `documents` folder of the first storage that has one.
#[derive(Debug, Clone)]
pub struct MtpDocumentsListing {
    pub description: String,
    /// MTP handle of the `documents` folder itself.
    pub documents_handle: u64,
    pub objects: Vec<MtpObjectSummary>,
}

/// Locate the root `documents` folder on the first MTP device storage that has
/// one, then enumerate its immediate children.
pub async fn list_documents() -> Result<Option<MtpDocumentsListing>, mtp_rs::Error> {
    let device = MtpDevice::open_first().await?;

    for storage in device.storages().await? {
        let info = storage.info();
        let root_objects = storage.list_objects(None).await?;

        let Some(documents) = root_objects
            .into_iter()
            .find(|object| object.is_folder() && object.filename == "documents")
        else {
            continue;
        };

        let children = storage.list_objects(Some(documents.handle)).await?;

        return Ok(Some(MtpDocumentsListing {
            description: info.description.clone(),
            documents_handle: documents.handle.0,
            objects: children.into_iter().map(MtpObjectSummary::from).collect(),
        }));
    }

    Ok(None)
}

/// List the immediate children of a folder by MTP handle on the first device.
///
/// Returns `None` when no storage/device is available.
pub async fn list_folder_children(handle: u64) -> Result<Option<MtpStorageListing>, mtp_rs::Error> {
    let device = MtpDevice::open_first().await?;

    let mut storages = device.storages().await?;
    let Some(storage) = storages.pop() else {
        return Ok(None);
    };

    let info = storage.info();
    let children = storage
        .list_objects(Some(mtp_rs::mtp::ObjectHandle(handle)))
        .await?;

    Ok(Some(MtpStorageListing {
        description: info.description.clone(),
        objects: children.into_iter().map(MtpObjectSummary::from).collect(),
    }))
}
