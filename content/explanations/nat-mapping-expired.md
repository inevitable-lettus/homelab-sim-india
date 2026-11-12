## What happened

The path in was open earlier. Then nothing used it for a while. A NAT on the way deleted its row for this connection. The next packet found the path closed.

## The concept

A NAT remembers each connection as a row in a table. It deletes the row once the connection has been idle (unused) long enough.

For UDP, that can be as short as 30 seconds. (UDP is the kind of traffic most tunnels use.) After the row is deleted, an incoming packet looks like a stranger again, and is dropped.

## Why it matters

This is the classic fault: "it worked when I set it up, and it was dead the next morning". The tunnel was fine while you tested it, because your testing kept it busy.

## How to fix it

Make the device behind NAT send a small packet at regular times. This packet is called a **keepalive**.

- In WireGuard, add `PersistentKeepalive = 25` on the device that is behind NAT.
- Mesh VPNs do this automatically.

## On a real line

Set the keepalive on the home side, not only on the phone. The home server must stay reachable even while nobody is using it.

## Go deeper

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): section 4.3 covers how long rows last.
- [WireGuard quick start](https://www.wireguard.com/quickstart/): see "NAT and firewall traversal persistence".
