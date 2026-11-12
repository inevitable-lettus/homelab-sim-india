//! Reads a `wg-quick` config file, the way `/etc/wireguard/wg0.conf` is
//! written. Mistakes come back with their line numbers. The private key is
//! only ever used to work out the public key, and is never put in a message.

use std::net::IpAddr;

use crate::artifacts::Problem;
use crate::ip::IpNet;
use crate::model::Endpoint;
use crate::x25519;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conf {
    /// Worked out from `PrivateKey`. The private key itself is not kept.
    pub public_key: String,
    pub addresses: Vec<IpNet>,
    pub listen_port: Option<u16>,
    pub mtu: Option<u32>,
    pub peers: Vec<ConfPeer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfPeer {
    pub public_key: String,
    pub endpoint: Option<Endpoint>,
    pub allowed_ips: Vec<IpNet>,
    pub keepalive_s: Option<u32>,
}

/// Settings `wg-quick` accepts that this model has no use for.
const INTERFACE_IGNORED: &[&str] = &["dns", "table", "preup", "postup", "predown", "postdown", "saveconfig", "fwmark"];

const KEY_SHAPE: &str = "A WireGuard key is 44 characters and ends in =";

#[derive(Default)]
struct PeerDraft {
    line: u32,
    public_key: Option<String>,
    endpoint: Option<Endpoint>,
    allowed_ips: Vec<IpNet>,
    keepalive_s: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Interface,
    Peer,
    /// A section we do not know. Its lines are skipped, after one complaint.
    Other,
}

/// The public key for a private key, both as WireGuard writes them. This is
/// what `wg pubkey` prints.
pub fn public_key(private: &str) -> Option<String> {
    let private = x25519::decode_key(private.trim())?;
    Some(x25519::encode_key(&x25519::public_key(&private)))
}

pub fn parse(text: &str) -> Result<Conf, Vec<Problem>> {
    let mut problems: Vec<Problem> = Vec::new();
    let mut problem = |line: u32, message: String| problems.push(Problem { line, message });

    let mut section = Section::None;
    let mut interface_line: Option<u32> = None;
    let mut public: Option<String> = None;
    let mut private_seen = false;
    let mut addresses = Vec::new();
    let mut listen_port = None;
    let mut mtu = None;
    let mut peers: Vec<PeerDraft> = Vec::new();

    for (index, raw) in text.lines().enumerate() {
        let line = index as u32 + 1;
        let content = raw.split('#').next().unwrap_or("").trim();
        if content.is_empty() {
            continue;
        }
        if content.starts_with('[') {
            let name = content.trim_start_matches('[').trim_end_matches(']').trim().to_ascii_lowercase();
            if !content.ends_with(']') {
                problem(line, "A section name needs a closing bracket, like [Interface]".to_owned());
            }
            section = match name.as_str() {
                "interface" => {
                    if interface_line.is_some() {
                        problem(line, "There is already an [Interface] section. A file has one".to_owned());
                    }
                    interface_line.get_or_insert(line);
                    Section::Interface
                }
                "peer" => {
                    peers.push(PeerDraft { line, ..PeerDraft::default() });
                    Section::Peer
                }
                _ => {
                    problem(line, format!("[{name}] is not a section. Use [Interface] and [Peer]"));
                    Section::Other
                }
            };
            continue;
        }
        let Some((key, value)) = content.split_once('=') else {
            problem(line, "Expected a setting written as Name = value".to_owned());
            continue;
        };
        let (name, value) = (key.trim(), value.trim());
        let key = name.to_ascii_lowercase();
        if value.is_empty() {
            problem(line, format!("{name} has no value"));
            // Said once is enough: do not also report the key as missing.
            private_seen |= section == Section::Interface && key == "privatekey";
            continue;
        }
        match section {
            Section::None => problem(line, format!("{name} comes before any section. Start with [Interface]")),
            Section::Other => {}
            Section::Interface => match key.as_str() {
                "privatekey" => {
                    if private_seen {
                        problem(line, "PrivateKey is set twice".to_owned());
                    }
                    private_seen = true;
                    match x25519::decode_key(value) {
                        Some(private) => public = Some(x25519::encode_key(&x25519::public_key(&private))),
                        None => problem(line, format!("PrivateKey is not a key. {KEY_SHAPE}")),
                    }
                }
                "address" => {
                    for part in list(value) {
                        match part.parse::<IpNet>() {
                            Ok(net) => addresses.push(net),
                            Err(_) => problem(line, format!("Address {part} is not an address like 10.8.0.2/24")),
                        }
                    }
                }
                "listenport" => match value.parse::<u16>() {
                    Ok(port) if port > 0 => listen_port = Some(port),
                    _ => problem(line, format!("ListenPort {value} is not a port from 1 to 65535")),
                },
                "mtu" => match value.parse::<u32>() {
                    Ok(bytes) if (576..=65535).contains(&bytes) => mtu = Some(bytes),
                    _ => problem(line, format!("MTU {value} is not a size from 576 to 65535 bytes")),
                },
                _ if INTERFACE_IGNORED.contains(&key.as_str()) => {}
                "publickey" | "endpoint" | "allowedips" | "persistentkeepalive" | "presharedkey" => {
                    problem(line, format!("{name} belongs under [Peer], not [Interface]"));
                }
                _ => problem(line, format!("{name} is not a setting [Interface] has")),
            },
            Section::Peer => {
                let Some(peer) = peers.last_mut() else {
                    continue;
                };
                match key.as_str() {
                    "publickey" => {
                        if peer.public_key.is_some() {
                            problem(line, "PublicKey is set twice for this peer".to_owned());
                        }
                        match x25519::decode_key(value) {
                            Some(_) => peer.public_key = Some(value.to_owned()),
                            None => problem(line, format!("PublicKey is not a key. {KEY_SHAPE}")),
                        }
                    }
                    "presharedkey" => {
                        if x25519::decode_key(value).is_none() {
                            problem(line, format!("PresharedKey is not a key. {KEY_SHAPE}"));
                        }
                    }
                    "allowedips" => {
                        for part in list(value) {
                            match part.parse::<IpNet>() {
                                Ok(net) => peer.allowed_ips.push(net),
                                Err(_) => {
                                    problem(line, format!("AllowedIPs {part} is not a range like 10.8.0.0/24"));
                                }
                            }
                        }
                    }
                    "endpoint" => match endpoint(value) {
                        Some(found) => peer.endpoint = Some(found),
                        None => problem(line, format!("Endpoint {value} is not written as address:port")),
                    },
                    "persistentkeepalive" => match value.to_ascii_lowercase().as_str() {
                        "off" | "0" => peer.keepalive_s = None,
                        seconds => match seconds.parse::<u16>() {
                            Ok(seconds) => peer.keepalive_s = Some(u32::from(seconds)),
                            Err(_) => problem(
                                line,
                                format!("PersistentKeepalive {value} is not a number of seconds from 1 to 65535"),
                            ),
                        },
                    },
                    "privatekey" | "address" | "listenport" | "mtu" | "dns" => {
                        problem(line, format!("{name} belongs under [Interface], not [Peer]"));
                    }
                    _ => problem(line, format!("{name} is not a setting [Peer] has")),
                }
            }
        }
    }

    match interface_line {
        None => problem(1, "There is no [Interface] section".to_owned()),
        Some(line) => {
            if !private_seen {
                problem(line, "[Interface] has no PrivateKey".to_owned());
            }
            if addresses.is_empty() {
                problem(line, "[Interface] has no Address, so the tunnel has no address of its own".to_owned());
            }
        }
    }
    for peer in &peers {
        if peer.public_key.is_none() {
            problem(peer.line, "This [Peer] has no PublicKey".to_owned());
        }
    }

    problems.sort_by_key(|p| p.line);
    if !problems.is_empty() {
        return Err(problems);
    }
    let Some(public_key) = public else {
        // Unreachable: a missing key is a problem above.
        return Err(vec![Problem { line: 1, message: "[Interface] has no PrivateKey".to_owned() }]);
    };
    Ok(Conf {
        public_key,
        addresses,
        listen_port,
        mtu,
        peers: peers
            .into_iter()
            .filter_map(|peer| {
                Some(ConfPeer {
                    public_key: peer.public_key?,
                    endpoint: peer.endpoint,
                    allowed_ips: peer.allowed_ips,
                    keepalive_s: peer.keepalive_s,
                })
            })
            .collect(),
    })
}

fn list(value: &str) -> impl Iterator<Item = &str> {
    value.split(',').map(str::trim).filter(|part| !part.is_empty())
}

/// `host:port`, or `[v6]:port`.
fn endpoint(value: &str) -> Option<Endpoint> {
    let (host, port) = value.rsplit_once(':')?;
    let port = port.parse::<u16>().ok().filter(|port| *port > 0)?;
    if let Some(rest) = host.strip_prefix('[') {
        let inner = rest.strip_suffix(']')?;
        inner.parse::<IpAddr>().ok().filter(IpAddr::is_ipv6)?;
        return Some(Endpoint { host: inner.to_owned(), port });
    }
    let plain = !host.is_empty() && host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    plain.then(|| Endpoint { host: host.to_owned(), port })
}

#[cfg(test)]
mod tests {
    use super::*;

    // A key pair made with Node's crypto module, to check against an
    // implementation that is not ours.
    const PRIVATE: &str = "EPXV4Q6FxuX4RT/KMQEiBWRYOGE5t/32aTjX944kZ18=";
    const PUBLIC: &str = "8zk2Okc5d/z9exmtbj/2cGf2/lZdwOfXFU7s4sItJjo=";
    const HUB: &str = "W9J3Hqm8EgeN6UewWxIP6ccD2CQZ8cC6fWREezgTW1U=";

    fn good() -> String {
        format!(
            "# Home server\n[Interface]\nPrivateKey = {PRIVATE}\nAddress = 10.8.0.2/24\nDNS = 1.1.1.1\n\n\
             [Peer]\nPublicKey = {HUB}  # the hub\nEndpoint = 192.0.2.50:51820\n\
             AllowedIPs = 10.8.0.0/24, fd00::/64\nPersistentKeepalive = 25\n"
        )
    }

    fn lines(text: &str) -> Vec<(u32, String)> {
        parse(text).expect_err("should not parse").into_iter().map(|p| (p.line, p.message)).collect()
    }

    #[test]
    fn public_key_matches_another_implementation() {
        assert_eq!(public_key(PRIVATE).as_deref(), Some(PUBLIC));
        assert_eq!(public_key(&format!("  {PRIVATE}\n")).as_deref(), Some(PUBLIC));
        assert_eq!(public_key("not a key"), None);
    }

    #[test]
    fn a_complete_file_parses() {
        let conf = parse(&good()).expect("parses");
        assert_eq!(conf.public_key, PUBLIC);
        assert_eq!(conf.addresses, vec!["10.8.0.2/24".parse().unwrap()]);
        assert_eq!(conf.listen_port, None);
        assert_eq!(conf.mtu, None);
        assert_eq!(conf.peers.len(), 1);
        let peer = &conf.peers[0];
        assert_eq!(peer.public_key, HUB);
        assert_eq!(peer.endpoint, Some(Endpoint { host: "192.0.2.50".to_owned(), port: 51820 }));
        assert_eq!(peer.allowed_ips, vec!["10.8.0.0/24".parse().unwrap(), "fd00::/64".parse().unwrap()]);
        assert_eq!(peer.keepalive_s, Some(25));
    }

    #[test]
    fn names_are_not_case_sensitive_and_keepalive_can_be_off() {
        let text = good()
            .replace("[Interface]", "[interface]")
            .replace("PrivateKey", "privatekey")
            .replace("PersistentKeepalive = 25", "persistentkeepalive = off");
        let conf = parse(&text).expect("parses");
        assert_eq!(conf.peers[0].keepalive_s, None);
        let conf = parse(&good().replace("= 25", "= 0")).expect("parses");
        assert_eq!(conf.peers[0].keepalive_s, None);
    }

    #[test]
    fn endpoints_take_names_and_bracketed_ipv6() {
        assert_eq!(
            endpoint("hub.example.net:51820"),
            Some(Endpoint { host: "hub.example.net".to_owned(), port: 51820 })
        );
        assert_eq!(endpoint("[2001:db8::1]:51820"), Some(Endpoint { host: "2001:db8::1".to_owned(), port: 51820 }));
        assert_eq!(endpoint("192.0.2.50"), None);
        assert_eq!(endpoint("192.0.2.50:0"), None);
        assert_eq!(endpoint("192.0.2.50:99999"), None);
        assert_eq!(endpoint("2001:db8::1:51820"), None);
        assert_eq!(endpoint(":51820"), None);
    }

    #[test]
    fn mistakes_carry_their_line_numbers() {
        let text = "Address = 10.8.0.2/24\n[Interface]\nPrivateKey = <paste it here>\nAddress = ten\nListenPort = 0\n\
             Colour = blue\nEndpoint = 192.0.2.50:51820\n[Peer]\nAllowedIPs = 10.8.0.0/33\nEndpoint = 192.0.2.50\n\
             PersistentKeepalive = soon\nno equals sign\n[Server]\nAnything = goes\n";
        let found = lines(text);
        let at: Vec<u32> = found.iter().map(|(line, _)| *line).collect();
        assert_eq!(at, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
        assert!(found[0].1.contains("before any section"));
        assert!(found[1].1.contains("no Address"));
        assert!(found[2].1.contains("PrivateKey is not a key"));
        assert!(found[6].1.contains("belongs under [Peer]"));
        assert!(found[7].1.contains("no PublicKey"));
        assert!(found[12].1.contains("not a section"));
    }

    #[test]
    fn an_empty_file_and_a_missing_key_are_problems() {
        assert_eq!(lines(""), vec![(1, "There is no [Interface] section".to_owned())]);
        let found = lines("[Interface]\nAddress = 10.8.0.2/24\n");
        assert_eq!(found, vec![(1, "[Interface] has no PrivateKey".to_owned())]);
    }

    #[test]
    fn the_private_key_never_appears_in_a_message() {
        // A real key in the wrong place, and a broken one in the right place.
        let broken = &PRIVATE[..43];
        let text =
            format!("[Interface]\nPrivateKey = {broken}\nAddress = 10.8.0.2/24\n[Peer]\nPrivateKey = {PRIVATE}\n");
        for (_, message) in lines(&text) {
            assert!(!message.contains(broken), "{message}");
        }
    }
}
