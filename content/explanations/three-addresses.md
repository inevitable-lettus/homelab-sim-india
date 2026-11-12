## What happened

Your server, your router and the internet each know your home by a different address. The address you dial decides how far the packet gets.

## The concept

Think of a flat in a housing society.

- The **flat number** is the server's home address, such as `192.168.1.20`. It is a **private** address: it only means something inside the building. Millions of homes use the same numbers.
- The **society's gate** is the router. Its outside side has a **WAN address**. The ISP gives it this address.
- The **postal address** that the outside world writes on a letter is the **public address**. A "what is my IP" site shows it to you.

On some lines, the WAN address and the public address are the same. Then the gate opens straight onto the street. On other lines, they are different. That means there is one more gate, and you do not control it. Mission 3 is about that second gate.

## Why it matters

Every remote-access guide starts with "find your IP address". But there are three addresses. Using the wrong one is the most common first mistake. A phone on mobile data cannot use the flat number, because the phone is not inside the building.

## How to fix it

Dial an address that the calling device can actually reach.

- From inside the home: the server's home address works.
- From outside: you need the public address, and a way through each gate.

## On a real line

1. Open your router's admin page. Find the WAN or Internet IP.
2. On a device connected to that router, open any "what is my IP" site.
3. Compare the two. If they match, your line has its own public address. If they are different, there is a NAT between your router and the internet.
4. Record the result with the line check in this app. It never asks for the addresses themselves.

## Go deeper

- [RFC 1918](https://www.rfc-editor.org/rfc/rfc1918): the private address ranges `10.0.0.0/8`, `172.16.0.0/12` and `192.168.0.0/16`.
- [RFC 6598](https://www.rfc-editor.org/rfc/rfc6598): the range `100.64.0.0/10`, set aside for ISPs that share public addresses.
