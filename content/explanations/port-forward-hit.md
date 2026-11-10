## What happened

The packet reached your router's public address, matched the port forward you added, and was sent on to the server.

## The concept

A port forward is a standing instruction to the router: "packets for this port go to this device inside". It is the guard's list at the society gate. Without it the router has no way to know which device an uninvited packet is for.

It works only if the packet can reach the router in the first place. That needs the router's WAN address to be a public one.

## Why it matters

Port forwarding is the oldest and simplest way to reach a home server, and most guides assume it works. It also puts the service directly on the internet: anyone who finds the address and port can try to connect.

## How to fix it

Nothing is broken. To do it well:

- Forward only the ports you need.
- Keep the service updated and behind a strong password.
- Prefer a VPN for anything that only you need to reach.

## On a real line

Check first that your line has its own public address: compare the router's WAN address with a "what is my IP" site. If they differ, a forward on your router cannot work, however carefully you set it up. That is the next mission.

## Go deeper

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): NAT behaviour, including the mappings a forward creates by hand.
