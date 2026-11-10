## What happened

The packet left the home with the router's WAN address as its source, not the server's or laptop's private address.

## The concept

A home has many devices and usually one address the outside can see. The router solves this with *network address translation*, or NAT. On the way out it replaces the private source address and port with its own, and writes the swap down in a table. When the reply comes back, it looks the swap up and sends the reply to the right device inside.

The table row only exists because something inside spoke first. That one fact explains most of this track.

## Why it matters

Outbound connections work without any setup. Inbound ones have no row to match, so they need something else: a port forward, or a connection that was opened from the inside.

## On a real line

Nothing to do. Every home router does this already.

## Go deeper

- [RFC 3022](https://www.rfc-editor.org/rfc/rfc3022): traditional IP network address translation.
- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): how NATs should treat UDP, including how long they remember a mapping.
