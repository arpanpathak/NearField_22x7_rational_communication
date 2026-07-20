/// PN532 UART frame encoding.
///
/// Encodes a [`Command`] into the PN532's UART packet format as specified
/// in the PN532 User Manual (Rev. 02, Section 7.1.1).
///
/// # Frame format
/// ```text
/// [0x00, 0x00] Preamble
/// [0xFF]       Start of frame
/// [LEN]        Payload length (TFI + CMD + params)
/// [LCS]        Length checksum: 0x100 - LEN
/// [TFI]        Transport format identifier (0xD4 = host to chip)
/// [CMD]        Command byte
/// [PD0..PDn]   Command parameters
/// [DCS]        Data checksum: 0x100 - sum(TFI + CMD + params)
/// [0x00]       Postamble
/// ```

use crate::nfc::commands::Command;

/// Transport format identifier: host to PN532.
const TFI_HOST: u8 = 0xD4;

/// Frame preamble: two zero bytes followed by start-of-frame marker.
const PREAMBLE: [u8; 3] = [0x00, 0x00, 0xFF];

/// Checksum modulus for LCS and DCS calculation.
const CHECKSUM_MOD: u16 = 0x100;

/// Encode a [`Command`] into a fully framed UART packet.
///
/// The returned vector is ready to be written to the serial port.
pub fn encode(cmd: Command) -> Vec<u8> {
    let payload = cmd.to_bytes();
    let len = payload.len() + 1;

    let mut packet = Vec::with_capacity(7 + payload.len());
    packet.extend_from_slice(&PREAMBLE);
    packet.push(len as u8);
    packet.push((CHECKSUM_MOD - len as u16) as u8);
    packet.push(TFI_HOST);
    packet.extend_from_slice(&payload);

    let dcs_sum: u16 = payload.iter().map(|&b| b as u16).sum::<u16>() + TFI_HOST as u16;
    packet.push((CHECKSUM_MOD - (dcs_sum & 0xFF)) as u8);
    packet.push(0x00);

    packet
}
