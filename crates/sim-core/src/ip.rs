//! Address helpers. Networks are written as strings in data ("192.168.1.0/24").

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// An address with a prefix length. A bare address means a single host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct IpNet {
    pub addr: IpAddr,
    pub prefix: u8,
}

fn mask32(prefix: u8) -> u32 {
    match prefix {
        0 => 0,
        p => u32::MAX << (32 - u32::from(p.min(32))),
    }
}

fn mask128(prefix: u8) -> u128 {
    match prefix {
        0 => 0,
        p => u128::MAX << (128 - u32::from(p.min(128))),
    }
}

impl IpNet {
    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.addr, ip) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => {
                let mask = mask32(self.prefix);
                u32::from(net) & mask == u32::from(ip) & mask
            }
            (IpAddr::V6(net), IpAddr::V6(ip)) => {
                let mask = mask128(self.prefix);
                u128::from(net) & mask == u128::from(ip) & mask
            }
            _ => false,
        }
    }

    /// The network address plus `n`.
    pub fn host(&self, n: u32) -> IpAddr {
        match self.addr {
            IpAddr::V4(net) => IpAddr::V4(Ipv4Addr::from((u32::from(net) & mask32(self.prefix)).wrapping_add(n))),
            IpAddr::V6(net) => {
                IpAddr::V6(Ipv6Addr::from((u128::from(net) & mask128(self.prefix)).wrapping_add(u128::from(n))))
            }
        }
    }

    /// The `index`-th /64 inside an IPv6 prefix, or `None` for IPv4.
    pub fn subnet64(&self, index: u32) -> Option<IpNet> {
        match self.addr {
            IpAddr::V6(net) => Some(IpNet {
                addr: IpAddr::V6(Ipv6Addr::from((u128::from(net) & mask128(self.prefix)) + (u128::from(index) << 64))),
                prefix: 64,
            }),
            IpAddr::V4(_) => None,
        }
    }
}

impl fmt::Display for IpNet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

impl From<IpNet> for String {
    fn from(net: IpNet) -> Self {
        net.to_string()
    }
}

impl FromStr for IpNet {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr, prefix) = match s.trim().split_once('/') {
            Some((addr, prefix)) => (addr.trim(), Some(prefix.trim())),
            None => (s.trim(), None),
        };
        let addr: IpAddr = addr.parse().map_err(|_| format!("`{s}` is not an IP address or network"))?;
        let max = if addr.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            None => max,
            Some(p) => p
                .parse::<u8>()
                .ok()
                .filter(|p| *p <= max)
                .ok_or_else(|| format!("`{s}` has an invalid prefix length"))?,
        };
        Ok(IpNet { addr, prefix })
    }
}

impl TryFrom<String> for IpNet {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// What kind of address this is, as far as reachability is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddrClass {
    Loopback,
    /// RFC 1918 (10/8, 172.16/12, 192.168/16) and IPv6 unique-local.
    Private,
    /// RFC 6598 shared address space, 100.64.0.0/10. Used inside carrier NAT.
    CarrierNat,
    LinkLocal,
    /// Routable on the internet. Documentation ranges count, since the sim uses them.
    Public,
}

pub fn classify(ip: IpAddr) -> AddrClass {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            if o[0] == 127 {
                AddrClass::Loopback
            } else if o[0] == 10 || (o[0] == 172 && (16..32).contains(&o[1])) || (o[0] == 192 && o[1] == 168) {
                AddrClass::Private
            } else if o[0] == 100 && (64..128).contains(&o[1]) {
                AddrClass::CarrierNat
            } else if o[0] == 169 && o[1] == 254 {
                AddrClass::LinkLocal
            } else {
                AddrClass::Public
            }
        }
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            if v6.is_loopback() {
                AddrClass::Loopback
            } else if first & 0xfe00 == 0xfc00 {
                AddrClass::Private
            } else if first & 0xffc0 == 0xfe80 {
                AddrClass::LinkLocal
            } else {
                AddrClass::Public
            }
        }
    }
}

pub fn routable(ip: IpAddr) -> bool {
    classify(ip) == AddrClass::Public
}

/// "ip:port", with brackets for IPv6.
pub fn sock(ip: IpAddr, port: u16) -> String {
    match ip {
        IpAddr::V4(v4) => format!("{v4}:{port}"),
        IpAddr::V6(v6) => format!("[{v6}]:{port}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn parses_and_contains() {
        let lan: IpNet = "192.168.1.0/24".parse().unwrap();
        assert!(lan.contains(ip("192.168.1.20")));
        assert!(!lan.contains(ip("192.168.2.20")));
        assert_eq!(lan.host(20), ip("192.168.1.20"));
        assert_eq!("10.8.0.2".parse::<IpNet>().unwrap().prefix, 32);
        assert!("10.8.0.2/33".parse::<IpNet>().is_err());
        assert!("jellyfin".parse::<IpNet>().is_err());
    }

    #[test]
    fn classifies_the_ranges_the_track_teaches() {
        assert_eq!(classify(ip("192.168.1.20")), AddrClass::Private);
        assert_eq!(classify(ip("172.31.0.1")), AddrClass::Private);
        assert_eq!(classify(ip("172.32.0.1")), AddrClass::Public);
        assert_eq!(classify(ip("100.64.12.34")), AddrClass::CarrierNat);
        assert_eq!(classify(ip("100.127.255.255")), AddrClass::CarrierNat);
        assert_eq!(classify(ip("100.128.0.1")), AddrClass::Public);
        assert_eq!(classify(ip("203.0.113.50")), AddrClass::Public);
        assert_eq!(classify(ip("2001:db8::1")), AddrClass::Public);
        assert_eq!(classify(ip("fd00::1")), AddrClass::Private);
    }

    #[test]
    fn ipv6_subnets() {
        let prefix: IpNet = "2001:db8:a::/48".parse().unwrap();
        let lan = prefix.subnet64(1).unwrap();
        assert_eq!(lan.to_string(), "2001:db8:a:1::/64");
        assert_eq!(lan.host(0x20), ip("2001:db8:a:1::20"));
    }
}
