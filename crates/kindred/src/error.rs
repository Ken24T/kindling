//! Kindred's unified error type for device-facing operations.
//!
//! Low-level failures from the USB (`rusb`) and MTP (`mtp-rs`) layers are
//! translated into Kindle-aware categories (project instructions §17) so the
//! application/UI can produce friendly messages while keeping the original
//! diagnostic context available. Categories that do not map to a specific
//! Kindred variant are preserved inside `Usb`/`Mtp` for diagnostics.

use std::fmt;

/// A Kindle-aware error from Kindred's device layer.
///
/// Categories mirror the failure modes hardware applications must expect
/// (§17): no device, unsupported model, mid-operation disconnect, a device
/// held by another client, permissions, full storage, timeouts, stale
/// handles, and malformed object metadata.
#[derive(Debug)]
pub enum KindredError {
    /// No supported Kindle is connected.
    NoDevice,
    /// A Kindle is connected but its model is not supported.
    UnsupportedModel {
        /// USB product id of the unsupported model.
        product_id: u16,
    },
    /// The MTP interface is held by another client (e.g. GVFS on Linux, a
    /// file manager, or another MTP service).
    DeviceBusy,
    /// The OS denied access to the device (e.g. missing Linux udev rules).
    PermissionDenied,
    /// The device refused the operation (read-only storage, write-protected
    /// object, or a Kindling safety guard).
    AccessDenied,
    /// The target storage is full.
    StorageFull,
    /// The device disappeared or stopped responding mid-operation.
    Disconnected,
    /// The operation timed out.
    Timeout,
    /// A previously-valid object handle is no longer valid (the device
    /// re-keyed it); re-list the parent and retry.
    StaleObject,
    /// The requested object or folder was not found on the device.
    NotFound,
    /// Object metadata was malformed or unexpected.
    InvalidObject {
        /// What was invalid.
        message: String,
    },
    /// A local file/OS I/O failure.
    Io(std::io::Error),
    /// USB-layer failure without a more specific Kindred category.
    Usb(rusb::Error),
    /// MTP-layer failure without a more specific Kindred category.
    Mtp(mtp_rs::Error),
}

impl fmt::Display for KindredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KindredError::NoDevice => write!(f, "no Kindle connected"),
            KindredError::UnsupportedModel { product_id } => {
                write!(
                    f,
                    "unsupported Kindle model (USB product id 0x{product_id:04x})"
                )
            }
            KindredError::DeviceBusy => write!(
                f,
                "device is in use by another application (e.g. a file manager or MTP service)"
            ),
            KindredError::PermissionDenied => write!(
                f,
                "permission denied accessing the device (check USB/udev permissions)"
            ),
            KindredError::AccessDenied => write!(f, "access denied"),
            KindredError::StorageFull => write!(f, "device storage is full"),
            KindredError::Disconnected => write!(f, "device disconnected or stopped responding"),
            KindredError::Timeout => write!(f, "device operation timed out"),
            KindredError::StaleObject => {
                write!(
                    f,
                    "stale object handle (device re-keyed it; re-list and retry)"
                )
            }
            KindredError::NotFound => write!(f, "requested object or folder not found"),
            KindredError::InvalidObject { message } => write!(f, "invalid object: {message}"),
            KindredError::Io(error) => write!(f, "local I/O error: {error}"),
            KindredError::Usb(error) => write!(f, "USB error: {error}"),
            KindredError::Mtp(error) => write!(f, "MTP error: {error}"),
        }
    }
}

impl std::error::Error for KindredError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KindredError::Io(error) => Some(error),
            KindredError::Usb(error) => Some(error),
            KindredError::Mtp(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for KindredError {
    fn from(error: std::io::Error) -> Self {
        KindredError::Io(error)
    }
}

impl From<rusb::Error> for KindredError {
    fn from(error: rusb::Error) -> Self {
        match error {
            rusb::Error::NoDevice => KindredError::NoDevice,
            rusb::Error::Access => KindredError::PermissionDenied,
            rusb::Error::Timeout => KindredError::Timeout,
            rusb::Error::Busy => KindredError::DeviceBusy,
            other => KindredError::Usb(other),
        }
    }
}

impl From<mtp_rs::Error> for KindredError {
    fn from(error: mtp_rs::Error) -> Self {
        match error {
            mtp_rs::Error::NoDevice => KindredError::NoDevice,
            mtp_rs::Error::Disconnected => KindredError::Disconnected,
            mtp_rs::Error::DeviceReset => KindredError::Disconnected,
            mtp_rs::Error::Timeout => KindredError::Timeout,
            mtp_rs::Error::StorageFull => KindredError::StorageFull,
            mtp_rs::Error::AccessDenied => KindredError::AccessDenied,
            mtp_rs::Error::PermissionDenied => KindredError::PermissionDenied,
            mtp_rs::Error::ExclusiveAccess => KindredError::DeviceBusy,
            mtp_rs::Error::Busy => KindredError::DeviceBusy,
            mtp_rs::Error::StaleHandle => KindredError::StaleObject,
            mtp_rs::Error::InvalidData { message } => KindredError::InvalidObject { message },
            other => KindredError::Mtp(other),
        }
    }
}

impl From<mtp_rs::UploadError> for KindredError {
    fn from(error: mtp_rs::UploadError) -> Self {
        KindredError::from(error.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_mtp_no_device() {
        let error = KindredError::from(mtp_rs::Error::NoDevice);
        assert!(matches!(error, KindredError::NoDevice));
    }

    #[test]
    fn maps_mtp_disconnect_and_reset() {
        assert!(matches!(
            KindredError::from(mtp_rs::Error::Disconnected),
            KindredError::Disconnected
        ));
        assert!(matches!(
            KindredError::from(mtp_rs::Error::DeviceReset),
            KindredError::Disconnected
        ));
    }

    #[test]
    fn maps_mtp_busy_sources() {
        assert!(matches!(
            KindredError::from(mtp_rs::Error::ExclusiveAccess),
            KindredError::DeviceBusy
        ));
        assert!(matches!(
            KindredError::from(mtp_rs::Error::Busy),
            KindredError::DeviceBusy
        ));
    }

    #[test]
    fn maps_mtp_permission_and_access() {
        assert!(matches!(
            KindredError::from(mtp_rs::Error::PermissionDenied),
            KindredError::PermissionDenied
        ));
        assert!(matches!(
            KindredError::from(mtp_rs::Error::AccessDenied),
            KindredError::AccessDenied
        ));
    }

    #[test]
    fn maps_mtp_storage_full_and_timeout() {
        assert!(matches!(
            KindredError::from(mtp_rs::Error::StorageFull),
            KindredError::StorageFull
        ));
        assert!(matches!(
            KindredError::from(mtp_rs::Error::Timeout),
            KindredError::Timeout
        ));
    }

    #[test]
    fn maps_mtp_stale_handle_and_invalid_data() {
        assert!(matches!(
            KindredError::from(mtp_rs::Error::StaleHandle),
            KindredError::StaleObject
        ));
        assert!(matches!(
            KindredError::from(mtp_rs::Error::InvalidData {
                message: "bad".to_owned()
            }),
            KindredError::InvalidObject { .. }
        ));
    }

    #[test]
    fn unmapped_mtp_errors_are_preserved() {
        assert!(matches!(
            KindredError::from(mtp_rs::Error::Cancelled),
            KindredError::Mtp(mtp_rs::Error::Cancelled)
        ));
    }

    #[test]
    fn maps_usb_errors() {
        assert!(matches!(
            KindredError::from(rusb::Error::NoDevice),
            KindredError::NoDevice
        ));
        assert!(matches!(
            KindredError::from(rusb::Error::Access),
            KindredError::PermissionDenied
        ));
        assert!(matches!(
            KindredError::from(rusb::Error::Timeout),
            KindredError::Timeout
        ));
        assert!(matches!(
            KindredError::from(rusb::Error::Busy),
            KindredError::DeviceBusy
        ));
        assert!(matches!(
            KindredError::from(rusb::Error::Overflow),
            KindredError::Usb(rusb::Error::Overflow)
        ));
    }

    #[test]
    fn display_has_friendly_messages() {
        assert_eq!(KindredError::NoDevice.to_string(), "no Kindle connected");
        assert!(
            KindredError::DeviceBusy
                .to_string()
                .contains("another application")
        );
        assert!(KindredError::StorageFull.to_string().contains("full"));
    }
}
