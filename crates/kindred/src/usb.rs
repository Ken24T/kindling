use crate::error::KindredError;

pub const AMAZON_VENDOR_ID: u16 = 0x1949;
pub const PAPERWHITE_12_PRODUCT_ID: u16 = 0x9981;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbKindle {
    pub bus_number: u8,
    pub device_address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial: Option<String>,
}

impl UsbKindle {
    pub fn model_name(&self) -> &'static str {
        match self.product_id {
            PAPERWHITE_12_PRODUCT_ID => "Kindle Paperwhite (12th generation)",
            _ => "Unknown Kindle",
        }
    }
}

pub fn discover_kindles() -> Result<Vec<UsbKindle>, KindredError> {
    let devices = rusb::devices()?;
    let mut kindles = Vec::new();

    for device in devices.iter() {
        let descriptor = match device.device_descriptor() {
            Ok(descriptor) => descriptor,
            Err(_) => continue,
        };

        if descriptor.vendor_id() != AMAZON_VENDOR_ID {
            continue;
        }

        if descriptor.product_id() != PAPERWHITE_12_PRODUCT_ID {
            continue;
        }

        // Opening the device lets us read its USB string descriptors.
        // Failure to open is not fatal to discovery: VID/PID still identify it.
        let handle = device.open().ok();

        let manufacturer = handle
            .as_ref()
            .and_then(|handle| handle.read_manufacturer_string_ascii(&descriptor).ok());

        let product = handle
            .as_ref()
            .and_then(|handle| handle.read_product_string_ascii(&descriptor).ok());

        let serial = handle
            .as_ref()
            .and_then(|handle| handle.read_serial_number_string_ascii(&descriptor).ok());

        kindles.push(UsbKindle {
            bus_number: device.bus_number(),
            device_address: device.address(),
            vendor_id: descriptor.vendor_id(),
            product_id: descriptor.product_id(),
            manufacturer,
            product,
            serial,
        });
    }

    Ok(kindles)
}
