/// PN532 command definitions.
///
/// Each variant maps to a PN532 host-to-chip command as defined in
/// the PN532 User Manual (Rev. 02, Section 7.3). The enum encodes
/// both the command byte and its parameter payload.
///
/// # Usage
/// Commands are passed to [`crate::nfc::frame::encode`] which wraps
/// them in the UART frame protocol.

/// Firmware version query command code.
const CMD_GET_FIRMWARE_VERSION: u8 = 0x02;

/// Security Access Module configuration command code.
const CMD_SAM_CONFIGURATION: u8 = 0x14;

/// Passive target detection command code.
const CMD_IN_LIST_PASSIVE_TARGET: u8 = 0x4A;

/// PN532 commands supported by this driver.
pub enum Command {
    /// Query chip firmware version (no parameters).
    GetFirmwareVersion,

    /// Configure the Security Access Module.
    /// Payload: [mode, timeout, irq]
    SamConfiguration([u8; 3]),

    /// Poll for a passive target in the RF field.
    /// Payload: [max_targets, baud_rate]
    InListPassiveTarget([u8; 2]),
}

impl Command {
    /// Serialise this command into a byte vector.
    ///
    /// Returns `[command_code | params...]`, the inner payload before
    /// frame wrapping.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Command::GetFirmwareVersion => vec![CMD_GET_FIRMWARE_VERSION],
            Command::SamConfiguration(params) => {
                let mut bytes = vec![CMD_SAM_CONFIGURATION];
                bytes.extend_from_slice(params);
                bytes
            }
            Command::InListPassiveTarget(params) => {
                let mut bytes = vec![CMD_IN_LIST_PASSIVE_TARGET];
                bytes.extend_from_slice(params);
                bytes
            }
        }
    }
}
