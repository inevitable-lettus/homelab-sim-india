## What happened

The packet reached your router's public address. It matched the port forward you added. The router sent it on to the server.

## The concept

A **port forward** is a fixed rule on the router: "packets for this port go to this device inside". It is the guard's visitor list at the society gate. Without it, the router cannot know which device a packet nobody asked for is meant for.

It works only if the packet can reach the router first. For that, the router's WAN address must be a public address.

## Why it matters

Port forwarding is the oldest and simplest way to reach a home server. Most guides assume it works.

It also puts the service directly on the internet. Anyone who finds the address and port can try to connect.

## How to fix it

Nothing is broken. To do it safely:

- Forward only the ports you need.
- Keep the service updated, and protect it with a strong password.
- Use a VPN instead for anything only you need to reach.

## On a real line

First check that your line has its own public address: compare the router's WAN address with a "what is my IP" site. If they are different, a forward on your router cannot work, however carefully you set it up. That is the next mission.

## Go deeper

- [RFC 4787](https://www.rfc-editor.org/rfc/rfc4787): how NATs behave, including the rows a port forward creates by hand.
