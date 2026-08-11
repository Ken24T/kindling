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
