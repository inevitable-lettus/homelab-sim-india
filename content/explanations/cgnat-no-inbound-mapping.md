## What happened

The packet was sent to the home's public address. It reached the ISP and stopped there. It never reached your router, so your port forward was never checked.

## The concept

There are not enough IPv4 addresses for every home to have its own. So many ISPs put their own NAT in front of a whole neighbourhood. Many homes then share one public address. This is called **carrier-grade NAT**, or **CGNAT**.

Think of a large housing society:

- The society has one postal address and one main gate. The management runs the main gate, not you.
- Your flat's door has a list of expected visitors. That list is your port forward.
- Visitors are stopped at the main gate first. You cannot edit the main gate's list.

The ISP's NAT works like your router's NAT. It lets a packet in only if something inside sent a packet out first. A packet nobody asked for matches nothing, so it is dropped.

You can spot CGNAT without any special tool:

1. Find the WAN address on your router's admin page. (The **WAN address** is the address the ISP gives your router's outside side.)
2. Open a "what is my IP" site.
3. If the two addresses are different, you are behind CGNAT. Often the WAN address is between `100.64.` and `100.127.`. That range is set aside for CGNAT.

## Why it matters

Most homelab guides say "forward a port". Behind CGNAT, that advice fails. It also fails silently: the router's settings page still shows the forward as active. People lose days here.

## How to fix it

You cannot open the ISP's gate yourself. Your options, cheapest first:

1. **Dial out instead.** Make the server open a connection to a relay, and meet it there. Mesh VPNs do this. This is the next mission.
2. **Use IPv6**, if both your home line and the network you are calling from have it. IPv6 has no NAT in the way.
3. **Rent a small server** that has a public address, and tunnel through it.
4. **Ask the ISP** for a public or static address. Some ISPs sell this as an add-on. Some do not offer it.

## On a real line

Do the line check before you spend an evening on port forwarding. If the two addresses are different, skip port forwarding. Go straight to a dial-out method.

## Go deeper

- [RFC 6598](https://www.rfc-editor.org/rfc/rfc6598): the shared address range `100.64.0.0/10`.
- [RFC 6888](https://www.rfc-editor.org/rfc/rfc6888): rules for carrier-grade NATs.
