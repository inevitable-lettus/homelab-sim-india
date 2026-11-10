## What happened

The path in was open earlier, but nothing used it for a while. A NAT on the way removed its row for the connection, and the next packet found the door shut.

## The concept

A NAT remembers each connection in a table row and deletes the row once it has been idle for long enough. For UDP, which tunnels use, that can be as short as 30 seconds. After that an inbound packet looks like a stranger again.

## Why it matters

This is the classic "it worked when I set it up and was dead the next morning" fault. The tunnel was fine while you tested it, because your testing kept it busy.

## How to fix it

Make the side that is behind NAT send a small packet regularly. In WireGuard this is `PersistentKeepalive = 25` on the peer that sits behind NAT. Mesh VPNs do it automatically.

## On a real line

Set the keepalive on the home side, not only on the phone. The home server is the one that must stay reachable while nobody is using it.

## Go deeper

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): section 4.3 covers mapping timeouts.
- [WireGuard quick start](https://www.wireguard.com/quickstart/): see "NAT and firewall traversal persistence".
