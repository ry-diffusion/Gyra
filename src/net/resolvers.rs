use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::Resolver;
use bevy::log::{info, trace, warn};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

pub fn resolve(address: impl ToString) -> io::Result<SocketAddr> {
    let mut address = address.to_string();
    let resolver = Resolver::new(ResolverConfig::cloudflare(), ResolverOpts::default())?;

    trace!("Resolving record for: {address}");
    let query = format!("_minecraft._tcp.{address}");

    trace!("Querying record: {query}");

    if let Ok(lookup) = resolver.srv_lookup(query) {
        let entry = lookup
            .iter()
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid SRV record"))?;
        trace!("SRV record: {entry:#?}");

        let incomplete_addr = (entry.target().to_string(), entry.port());

        trace!("Incomplete address: {incomplete_addr:#?}");
        let mut addrs = incomplete_addr.to_socket_addrs()?;
        let addr = addrs
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid address"))?;

        info!("Resolved {address} -> {addr:#?}");
        Ok(addr)
    } else {
        if !address.contains(':') {
            address = format!("{address}:25565");
        }

        // the server may not support SRV records, so we'll just try to connect to the address directly
        warn!("No SRV record found, attempting to resolve address directly");
        let mut addrs = address.to_socket_addrs()?;
        let addr = addrs
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid address"))?;
        info!("Resolved {address} -> {addr:#?}");
        Ok(addr)
    }
}
