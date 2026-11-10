## What happened

Your router has a correct port forward, but the router's WAN address is not a public one. Packets from the internet stop at the ISP before they can reach the forward.

## The concept

A port forward tells *your* router what to do with an inbound packet. Behind carrier NAT there is a second NAT, run by the ISP, between the internet and your router. It drops inbound packets that nothing inside asked for, so your router never sees them.

## Why it matters

The router's admin page will show the forward as saved and active. Nothing on your side reports a problem. The only sign is that it does not work from outside.

## How to fix it

Leave port forwarding and use a method where the server dials out: a mesh VPN, a tunnel to a rented server, or IPv6 if both ends have it.

## On a real line

Compare your router's WAN address with a "what is my IP" site. If they differ, a port forward cannot work on this line.

## Go deeper

- [RFC 6888](https://www.rfc-editor.org/rfc/rfc6888): requirements for carrier-grade NATs.
