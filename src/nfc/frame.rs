use crate::nfc::commands::Command;

const TFI_HOST: u8 = 0xD4;
const PREAMBLE: [u8; 3] = [0x00, 0x00, 0xFF];

pub fn encode(cmd: Command) -> Vec<u8> {
    let payload = cmd.to_bytes();
    let len = payload.len() + 1;

    let mut packet = Vec::with_capacity(7 + payload.len());
    packet.extend_from_slice(&PREAMBLE);
    packet.push(len as u8);
    packet.push((0x100 - len as u16) as u8);
    packet.push(TFI_HOST);
    packet.extend_from_slice(&payload);

    let dcs_sum: u16 = payload.iter().map(|&b| b as u16).sum::<u16>() + TFI_HOST as u16;
    packet.push((0x100 - (dcs_sum & 0xFF)) as u8);
    packet.push(0x00);

    packet
}
