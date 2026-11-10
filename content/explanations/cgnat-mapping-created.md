## What happened

A packet went *out* from the home. The ISP's carrier NAT replaced its source with a shared public address and added a row to its table. Packets coming back to that row are now allowed in.

## The concept

A NAT, whether your router's or the ISP's, has one rule: replies are welcome, strangers are not. It tells them apart with a table. Each outbound packet writes a row: "this inside address and port is talking to that outside address and port". A packet arriving from that outside address, for that port, is a reply and goes through.

So the gate that blocks every inbound connection will hold a door open for as long as the inside keeps talking. If the server dials out to a relay on the internet, and the phone dials out to the same relay, the relay can pass packets between the two connections. Neither side ever accepted an inbound connection. This is how mesh VPNs get through carrier NAT.

Rows do not last forever. A NAT removes a UDP row after it has been idle for a while, often between 30 seconds and a few minutes. So the tunnel sends a tiny *keepalive* packet every 25 seconds or so to stop the row from expiring.

## Why it matters

This is the idea behind almost every way of reaching a home server on an Indian broadband line: mesh VPNs, tunnel services and rented relays all start with the server dialling out.

## How to fix it

Nothing is broken. This is the row doing its job.

## On a real line

A mesh VPN does all of this for you. Install it on the server and the phone, sign both in to the same account, and use the address the VPN gives the server. Real mesh VPNs also try to connect the two devices directly, and fall back to a relay when they cannot. This model always uses the relay.

## Go deeper

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): NAT behaviour for UDP, including mapping timeouts.
- [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445): ICE, the standard method for finding a path between two devices behind NATs.
