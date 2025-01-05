use std::io::Write;

use flate2::write::ZlibEncoder;
use gyra_codec::{coding::Encoder, packet::Packet, variadic_int::VarInt};
use log::{debug, trace};

pub fn put_uncompressed<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
) -> gyra_codec::error::Result<usize> {
    let mut data_buffer = Vec::new();
    let mut packet_buffer = Vec::new();

    /* Packet ID */
    VarInt::from(P::ID).encode(&mut data_buffer)?;
    packet.encode(&mut data_buffer)?;

    debug!(
        "sending {} bytes of a uncompressed packet ({}).",
        data_buffer.len(),
        P::ID
    );

    /* Packet Length */
    VarInt::from(data_buffer.len() as i32).encode(&mut packet_buffer)?;
    packet_buffer.append(&mut data_buffer);

    writer.write_all(&packet_buffer)?;

    trace!("[Client->Server] Packet data: {:02X?}", packet_buffer);

    Ok(packet_buffer.len())
}

/**
* Let's try to send a uncompressed packet as COMPRESSED PACKET!
* I still wanna know why mojang did this.
*/
pub fn put_compressed_uncompressed<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
) -> gyra_codec::error::Result<usize> {
    let mut packet_buffer = vec![];
    let mut data_buffer = vec![];

    /* Packet ID */
    VarInt::from(P::ID).encode(&mut packet_buffer)?;

    /* Packet Data */
    packet.encode(&mut packet_buffer)?;

    // 1. Packet Length
    VarInt::from(packet_buffer.len() as i32 + 1).encode(&mut data_buffer)?;

    // 2. Data Length
    VarInt::from(0).encode(&mut data_buffer)?;

    // 3. Packet Data
    packet_buffer.append(&mut data_buffer);

    writer.write_all(&data_buffer)?;

    Ok(packet_buffer.len())
}

pub fn put_compressed<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
    threshold: u32,
) -> gyra_codec::error::Result<usize> {
    let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let mut uncompressed_size = 0;
    let mut packet_buffer = Vec::new();
    let mut data_buffer = Vec::new();

    /* Packet ID */
    uncompressed_size += VarInt::from(P::ID).encode(&mut encoder)?;
    uncompressed_size += packet.encode(&mut encoder)?;

    if (uncompressed_size as u32) < threshold {
        trace!(
            "Packet size is less than threshold; sending {uncompressed_size} bytes uncompressed"
        );
        return put_compressed_uncompressed(writer, packet);
    }

    trace!("Packet size is greater than threshold, sending {uncompressed_size} bytes compressed.");

    let compressed_data = encoder.finish()?;
    let mut packet_size = compressed_data.len();

    /* FORMAT:
     * packet length
     * uncompressed length
     * compressed data *  */

    packet_size += VarInt::from(uncompressed_size as i32).encode(&mut data_buffer)?;

    VarInt::from(packet_size as i32).encode(&mut packet_buffer)?;
    packet_buffer.append(&mut data_buffer);

    writer.write_all(&packet_buffer)?;

    Ok(packet_buffer.len())
}

pub fn put<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
    threshold: Option<u32>,
) -> gyra_codec::error::Result<usize> {
    debug!(
        "[Client->Server] Sending packet with ID: 0x{:02X}/{:?}",
        P::ID,
        P::WHEN
    );

    if let Some(threshold) = threshold {
        put_compressed(writer, packet, threshold)
    } else {
        put_uncompressed(writer, packet)
    }
}
