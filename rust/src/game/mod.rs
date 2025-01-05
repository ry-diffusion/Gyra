use gyra_net::codec::packet::When;
use gyra_net::proto::Protocol;
use gyra_net::transport::Transport;
use log::info;

pub struct NetworkGame {
    pub transport: Transport,
}

impl NetworkGame {
    pub fn connect(addr: impl ToString) -> std::io::Result<Self> {
        Ok(NetworkGame {
            transport: Transport::connect(addr)?,
        })
    }

    pub fn transport_state(&self) -> When {
        self.transport.state
    }

    pub fn poll_login(&mut self) -> gyra_net::Result<()> {
        let packet = self.transport.poll_packet()?;
        match packet {
            Protocol::LoginSuccess(packet) => {
                info!("Login successful");
                self.transport.state = When::Play;
                self.transport.stream.set_nonblocking(true)?;
                Ok(())
            }

            Protocol::SetCompression(packet) => {
                info!(
                    "Server is compressing packets, threshould = {}",
                    packet.threshold.0
                );

                self.transport.server_compress_threshold = Some(packet.threshold.into());
                Ok(())
            }

            _ => {
                info!("Received unexpected packet: {:?}", packet);
                Err(gyra_net::error::Error::UnexpectedPacket)
            }
        }
    }
}
