## What happened

The phone dialled a private address from outside the home. The packet left the phone. But no network on the internet knows where that address is, so the packet was dropped.

## The concept

Addresses that start with `192.168.`, `10.`, or `172.16.` to `172.31.` are **private** addresses. Any home or office may use them, and almost all do. The same address exists in millions of places, so the internet will not carry packets sent to one.

It is like writing only "Flat 20" on a letter, with no building, street or city.

## Why it matters

The address you see on the server itself is almost always private. It is the right address at home and the wrong address everywhere else.

## How to fix it

1. Dial the home's public address instead.
2. Make sure each gate on the way in will let the packet through.

## On a real line

An app works on home Wi-Fi but stops when you switch to mobile data? Check whether it uses an address starting with `192.168.` or `10.`.

## Go deeper

- [RFC 1918](https://www.rfc-editor.org/rfc/rfc1918): private address ranges.
