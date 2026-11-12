## What happened

This end of the tunnel is behind NAT, and it has no keepalive set. When the tunnel goes quiet, the NAT forgets it. Then nothing can reach this device through the tunnel until the device sends something again.

## The concept

WireGuard is silent by design. It sends nothing when there is nothing to send. That saves battery. But behind a NAT it is a problem, because the NAT keeps the path open only while packets flow.

A **keepalive** is a tiny packet sent at regular times to keep the path open.

## How to fix it

On the device that is behind NAT, add `PersistentKeepalive = 25` to the `[Peer]` section.

## On a real line

A home server behind an ISP's NAT always needs this. A server with its own public address does not.

## Go deeper

- [WireGuard quick start](https://www.wireguard.com/quickstart/): "NAT and firewall traversal persistence".
