## What happened

The packet was addressed to the home's public address. It reached the ISP and stopped there. It never got to your router, so your port forward was never even looked at.

## The concept

There are not enough IPv4 addresses for every home to have its own. So many ISPs put a NAT of their own in front of a whole neighbourhood and share one public address between many homes. This is *carrier-grade NAT*, or CGNAT.

Think of a large society with one postal address and a main gate run by the management, not by you. Your own flat's door has a list of expected visitors (your port forward). But visitors are stopped at the main gate, and the main gate's list is not yours to edit.

The ISP's NAT works like your router's. It lets a packet in only if something inside sent one out first. An uninvited packet matches nothing and is dropped.

You can spot CGNAT without any special tool: the WAN address on your router is different from the address a "what is my IP" site shows. Often the WAN address starts with `100.64.` to `100.127.`, a range reserved for this.

## Why it matters

Most homelab guides say "forward a port". Behind CGNAT that advice fails, and it fails silently: the router's settings page shows the forward as active. People lose days here.

## How to fix it

You cannot open the ISP's gate yourself. The options, cheapest first:

- **Dial out instead.** Have the server open a connection to a relay, and meet it there. Mesh VPNs do this. This is the next mission.
- **Use IPv6**, if both your home line and the network you are calling from have it. IPv6 has no NAT in the way.
- **Rent a small server** with a public address and tunnel through it.
- **Ask the ISP** for a public or static address. Some sell this as an add-on and some do not offer it.

## On a real line

Do the line check before you spend an evening on port forwarding. If the two addresses differ, skip forwarding and go straight to a dial-out method.

## Go deeper

- [RFC 6598](https://www.rfc-editor.org/rfc/rfc6598): the shared address space `100.64.0.0/10`.
- [RFC 6888](https://www.rfc-editor.org/rfc/rfc6888): requirements for carrier-grade NATs.
