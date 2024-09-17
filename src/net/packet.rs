use std::io::Write;
use flate2::write::ZlibEncoder;
use log::{info, trace, warn};
use gyra_codec::coding::Encoder;
use gyra_codec::packet::Packet;
use gyra_codec::variadic_int::VarInt;

pub fn put_uncompressed<P: Packet>(writer: &mut impl Write, packet: &P) -> crate::error::Result<()> {
    let mut data_buffer = Vec::new();
    let mut packet_buffer = Vec::new();

    /* Packet ID */
    VarInt::from(P::ID).encode(&mut data_buffer)?;
    packet.encode(&mut data_buffer)?;

    /* Packet Length */
    VarInt::from(data_buffer.len() as i32).encode(&mut packet_buffer)?;
    packet_buffer.append(&mut data_buffer);

    writer.write_all(&packet_buffer)?;


    trace!("[Client->Server] Packet data: {:02X?}", packet_buffer);

    Ok(())
}

pub fn put_compressed<P: Packet>(writer: &mut impl Write, packet: &P, threshold: u32) -> crate::error::Result<()> {
    let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let mut uncompressed_size = 0;
    let mut packet_buffer = Vec::new();
    let mut data_buffer = Vec::new();

    /* Packet ID */
    uncompressed_size += VarInt::from(P::ID).encode(&mut encoder)?;
    uncompressed_size += packet.encode(&mut encoder)?;

    if (uncompressed_size as u32) < threshold {
        warn!("Packet size is less than threshold, sending {uncompressed_size} bytes uncompressed");
        return put_uncompressed(writer, packet);
    }


    info!("Packet size is greater than threshold, sending {uncompressed_size} bytes compressed.");

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

    Ok(())
}

pub fn put<P: Packet>(writer: &mut impl Write, packet: &P, threshold: Option<u32>) -> crate::error::Result<()> {
    info!("[Client->Server] Sending packet with ID: 0x{:02X}/{:?}", P::ID, P::WHEN);

    if let Some(threshold) = threshold {

        put_compressed(writer, packet, threshold)?;
    } else {
        put_uncompressed(writer, packet)?;
    }

    Ok(())
}