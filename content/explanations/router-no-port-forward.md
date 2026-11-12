## What happened

The packet reached your router's WAN address. The router had no forward for that port. No device inside had asked for the packet. So the router dropped it.

## The concept

The router is the society's gate.

1. A visitor arrives and says "port 8096".
2. The guard has a list of flats that expect visitors on certain ports.
3. There is no entry for 8096. The guard cannot know which flat is meant, so the visitor is sent away.

A **port forward** is one entry in that list: "anything for port 8096 goes to `192.168.1.20`, port 8096".

## Why it matters

Every home router starts like this, and that is a good default. Nothing gets in unless you allow it.

## How to fix it

1. Give the server a fixed home address. Otherwise, after a restart, the forward may point at the wrong device.
2. Add a port forward on the router for the service's port, pointing at the server's home address.

## On a real line

In your router's admin page, look for "Port forwarding", "Virtual server" or "NAT". Forward only the ports you need.

## Go deeper

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): how NATs handle UDP, including blocking incoming packets.
