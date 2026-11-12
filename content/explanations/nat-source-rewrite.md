## What happened

The packet left the home with the router's WAN address as its source. It did not carry the server's or laptop's private address.

## The concept

A home has many devices. It usually has only one address that the outside can see. The router solves this with **network address translation**, or **NAT**:

1. On the way out, the router replaces the private source address and port with its own.
2. It writes this swap down as a row in a table.
3. When the reply comes back, the router looks up the row and sends the reply to the right device inside.

The row exists only because something inside spoke first. That one fact explains most of this track.

## Why it matters

Outgoing connections work with no setup. Incoming connections have no row to match. So they need something else: either a port forward, or a connection that was opened from inside.

## On a real line

Nothing to do. Every home router already does this.

## Go deeper

- [RFC 3022](https://www.rfc-editor.org/rfc/rfc3022): traditional network address translation.
- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): how NATs should handle UDP, including how long they remember a row.
