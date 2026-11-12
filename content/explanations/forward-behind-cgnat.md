## What happened

Your router's port forward is correct. But the router's WAN address is not a public address. Packets from the internet stop at the ISP before they reach your forward.

## The concept

A port forward tells *your* router what to do with an incoming packet.

Behind carrier NAT, there is a second NAT between the internet and your router. The ISP runs it. It drops incoming packets that nothing inside asked for. So your router never sees them.

## Why it matters

The router's admin page shows the forward as saved and active. Nothing on your side reports a problem. The only sign is that it does not work from outside.

## How to fix it

Stop using port forwarding. Use a method where the server dials out instead:

- a mesh VPN,
- a tunnel to a rented server, or
- IPv6, if both ends have it.

## On a real line

1. Note your router's WAN address.
2. Note the address a "what is my IP" site shows.
3. If they are different, a port forward cannot work on this line.

## Go deeper

- [RFC 6888](https://www.rfc-editor.org/rfc/rfc6888): rules for carrier-grade NATs.
