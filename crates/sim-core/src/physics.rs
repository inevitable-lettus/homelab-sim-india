//! Network physics in integer units: nanoseconds, bytes, bits per second,
//! metres. No floats, so the same input always gives the same bytes out.
//!
//! These are textbook first-order models. The UI labels results as estimates.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Medium {
    #[default]
    Ethernet,
    Fibre,
    Wifi,
    Cellular,
    /// Long-distance carrier fibre between cities.
    Backbone,
}

impl Medium {
    /// Signal travel time in picoseconds per metre.
    /// Glass fibre has a refractive index near 1.47, so light covers a
    /// kilometre in about 4.9 microseconds. Copper pairs are close to that.
    /// Radio travels at the speed of light in air.
    pub const fn ps_per_m(self) -> u64 {
        match self {
            Medium::Fibre | Medium::Backbone => 4_900,
            Medium::Ethernet => 5_000,
            Medium::Wifi | Medium::Cellular => 3_336,
        }
    }
}

/// WireGuard adds a 20-byte IPv4 or 40-byte IPv6 header, 8 bytes of UDP and 32
/// bytes of its own. 1500 - 80 gives the well-known default tunnel MTU of 1420.
pub const fn tunnel_overhead_bytes(outer_is_ipv6: bool) -> u32 {
    if outer_is_ipv6 { 80 } else { 60 }
}

pub const WIREGUARD_DEFAULT_MTU: u32 = 1420;
pub const WIREGUARD_HANDSHAKE_BYTES: u32 = 176;
pub const DEFAULT_LINK_MTU: u32 = 1500;
pub const TCP_IPV4_HEADER_BYTES: u32 = 40;
pub const TCP_IPV6_HEADER_BYTES: u32 = 60;

pub fn propagation_ns(medium: Medium, length_m: u64) -> u64 {
    length_m.saturating_mul(medium.ps_per_m()) / 1_000
}

/// Time to clock a packet onto a link: bits divided by bit rate.
pub fn serialization_ns(bytes: u32, rate_bps: u64) -> Option<u64> {
    if rate_bps == 0 {
        return None;
    }
    let ns = u128::from(bytes) * 8 * 1_000_000_000 / u128::from(rate_bps);
    Some(u64::try_from(ns).unwrap_or(u64::MAX))
}

/// Average wait behind other traffic, from the M/D/1 queue formula
/// `wait = service * load / (2 * (1 - load))`. Load is in thousandths and is
/// capped at 99%, where the formula stops being meaningful.
pub fn queue_ns(serialization_ns: u64, load_permille: u16) -> u64 {
    let load = u128::from(load_permille.min(990));
    let wait = u128::from(serialization_ns) * load / (2 * (1_000 - load));
    u64::try_from(wait).unwrap_or(u64::MAX)
}

/// Rate left for one more flow on a link that is already `load` busy.
pub fn spare_rate_bps(rate_bps: u64, load_permille: u16) -> u64 {
    let spare = u128::from(rate_bps) * u128::from(1_000 - load_permille.min(990)) / 1_000;
    u64::try_from(spare).unwrap_or(u64::MAX)
}

/// Bytes on the wire for a payload split into full packets, each carrying
/// `header_bytes` of overhead inside a `mtu`-sized packet.
pub fn wire_bytes(payload_bytes: u64, mtu: u32, header_bytes: u32) -> Option<u64> {
    let per_packet = u64::from(mtu.checked_sub(header_bytes).filter(|p| *p > 0)?);
    let packets = payload_bytes.div_ceil(per_packet);
    Some(payload_bytes.saturating_add(packets.saturating_mul(u64::from(header_bytes))))
}

pub fn transfer_ms(wire_bytes: u64, rate_bps: u64) -> Option<u64> {
    if rate_bps == 0 {
        return None;
    }
    let ms = u128::from(wire_bytes) * 8 * 1_000 / u128::from(rate_bps);
    Some(u64::try_from(ms).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibre_is_about_five_microseconds_per_km() {
        assert_eq!(propagation_ns(Medium::Fibre, 1_000), 4_900);
        // Mumbai to Singapore is roughly 3,900 km in a straight line.
        assert_eq!(propagation_ns(Medium::Backbone, 3_900_000), 19_110_000);
    }

    #[test]
    fn a_full_packet_on_fast_ethernet_takes_120_microseconds() {
        assert_eq!(serialization_ns(1_500, 100_000_000), Some(120_000));
        assert_eq!(serialization_ns(1_500, 0), None);
    }

    #[test]
    fn queueing_grows_sharply_near_full_load() {
        assert_eq!(queue_ns(120_000, 0), 0);
        assert_eq!(queue_ns(120_000, 500), 60_000);
        assert_eq!(queue_ns(120_000, 900), 540_000);
        assert_eq!(queue_ns(120_000, 1_000), queue_ns(120_000, 990));
    }

    #[test]
    fn a_two_gigabyte_backup_on_a_20_mbit_upload() {
        let wire = wire_bytes(2_000_000_000, 1_500, 40).unwrap();
        assert_eq!(wire, 2_054_794_560);
        // About 13 minutes 42 seconds.
        assert_eq!(transfer_ms(wire, 20_000_000), Some(821_917));
        assert_eq!(wire_bytes(10, 40, 40), None);
    }

    #[test]
    fn wireguard_default_mtu_follows_from_the_overhead() {
        assert_eq!(DEFAULT_LINK_MTU - tunnel_overhead_bytes(true), WIREGUARD_DEFAULT_MTU);
        assert_eq!(tunnel_overhead_bytes(false), 60);
    }

    #[test]
    fn spare_rate_shrinks_with_load() {
        assert_eq!(spare_rate_bps(20_000_000, 250), 15_000_000);
    }
}
