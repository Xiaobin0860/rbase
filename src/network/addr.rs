use std::net::{IpAddr, SocketAddr};

/// Convert an IPv4 address to a u32.
pub fn ipv4_to_u32(ip: &str) -> u32 {
    match ip.parse() {
        Ok(IpAddr::V4(ip)) => u32::from_be_bytes(ip.octets()),
        Ok(IpAddr::V6(ip)) => panic!("ipv6 not supported: {ip}"),
        _ => panic!("invalid ip: {ip}"),
    }
}

/// Convert a u32 to an IPv4 address.
pub fn u32_to_ipv4(ip: u32) -> String {
    let ip = ip.to_be_bytes();
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}
pub fn u32_to_ipv4_le(ip: u32) -> String {
    let ip = ip.to_le_bytes();
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

pub fn addr_v4(addr: &SocketAddr) -> (u32, u16) {
    match addr.ip() {
        IpAddr::V4(ip) => (u32::from_be_bytes(ip.octets()), addr.port()),
        IpAddr::V6(ip) => panic!("ipv6 not supported: {ip}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ip_convert_should_work() {
        assert_eq!(ipv4_to_u32("127.0.0.1"), 0x7f000001);
        assert_eq!("127.0.0.1", u32_to_ipv4(0x7f000001));
        assert_eq!("192.168.192.158", u32_to_ipv4(0xc0a8c09e));
    }
}
