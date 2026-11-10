## What happened

The packet reached your router's WAN address. The router had no forward for that port, and no device inside had asked for the packet, so the router dropped it.

## The concept

The router is the society's gate. A visitor arrives and says "port 8096". The guard has a list of flats that are expecting visitors on particular ports. With no entry for 8096, the guard cannot know which flat is meant, and sends the visitor away.

A *port forward* is an entry in that list: "anything for port 8096 goes to `192.168.1.20`, port 8096".

## Why it matters

This is the default state of every home router, and it is a good default. Nothing gets in unless you say so.

## How to fix it

Add a port forward on the router for the service's port, pointing at the server's home address. Give the server a fixed home address first, or the forward will point at the wrong device after a restart.

## On a real line

Look for "Port forwarding", "Virtual server" or "NAT" in your router's admin page. Only forward the ports you need.

## Go deeper

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): NAT behaviour for UDP, including filtering of inbound packets.
