## What happened

The phone dialled a private address from outside the home. The packet left the phone, but no network on the internet knows where that address is, so it was dropped.

## The concept

Addresses that start with `192.168.`, `10.` or `172.16.` to `172.31.` are *private*. Any home or office may use them, and almost all do. Because the same address exists in millions of places, the internet refuses to carry packets addressed to one. It is like writing "Flat 20" on a letter with no building, street or city.

## Why it matters

The address you see on the server itself is almost always private. It is the right address at home and the wrong one everywhere else.

## How to fix it

Dial the home's public address instead, then make sure each gate on the way in will pass the packet.

## On a real line

If an app works on home Wi-Fi and stops the moment you switch to mobile data, check whether it is pointed at an address starting with `192.168.` or `10.`.

## Go deeper

- [RFC 1918](https://www.rfc-editor.org/rfc/rfc1918): address allocation for private internets.
