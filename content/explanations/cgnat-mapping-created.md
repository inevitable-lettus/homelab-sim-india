## What happened

A packet went *out* from the home. The ISP's carrier NAT did two things:

1. It replaced the packet's source address with a shared public address.
2. It added a row to its table for this connection.

Packets that come back and match that row are now allowed in.

## The concept

**NAT** (network address translation) is a gate that swaps addresses on packets. Your router has one. On some lines, the ISP has one too. That second one is called **carrier NAT**.

Every NAT follows one rule: replies get in, strangers do not.

It tells them apart with a **table**:

- Each packet going out writes a **row**: "this inside address and port is talking to that outside address and port".
- A packet coming in from that outside address, to that port, matches the row. It is a reply, so it gets in.
- A packet that matches no row is a stranger. It is dropped.

So a NAT that blocks every new incoming connection still keeps a path open, as long as the inside keeps talking. This is how mesh VPNs get past carrier NAT:

1. The server dials out to a **relay** (a computer on the internet that passes packets along).
2. The phone also dials out to the same relay.
3. The relay passes packets between the two connections.

Neither side ever accepted an incoming connection.

Rows do not last forever. A NAT deletes a UDP row after it has been idle (unused) for a while, often between 30 seconds and a few minutes. So the tunnel sends a tiny packet about every 25 seconds. This is called a **keepalive**. It stops the row from being deleted.

## Why it matters

Almost every way to reach a home server on an Indian broadband line uses this idea. Mesh VPNs, tunnel services and rented relays all start with the server dialling out.

## How to fix it

Nothing is broken. The row is doing its job.

## On a real line

A mesh VPN does all of this for you:

1. Install the mesh VPN on the server and on the phone.
2. Sign both in to the same account.
3. Use the address the VPN gives the server.

Real mesh VPNs also try to connect the two devices directly. If they cannot, they fall back to a relay. This simulator always uses the relay.

## Go deeper

RFCs are the official documents that define how the internet works.

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): how NATs handle UDP, including how long rows last.
- [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445): ICE, the standard way to find a path between two devices that are both behind NATs.
