pub enum Command {
    GetFirmwareVersion,
    SamConfiguration([u8; 3]),
    InListPassiveTarget([u8; 2]),
}

impl Command {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Command::GetFirmwareVersion => vec![0x02],
            Command::SamConfiguration(params) => {
                let mut bytes = vec![0x14];
                bytes.extend_from_slice(params);
                bytes
            }
            Command::InListPassiveTarget(params) => {
                let mut bytes = vec![0x4A];
                bytes.extend_from_slice(params);
                bytes
            }
        }
    }
}
