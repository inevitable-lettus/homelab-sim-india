## What happened

This end of the tunnel is behind NAT and has no keepalive set. Once the tunnel goes quiet, the NAT forgets it and nothing can reach this device through the tunnel until it speaks again.

## The concept

WireGuard is silent by design: it sends nothing when there is nothing to send. That is good for battery and bad behind a NAT, because the NAT only keeps the path open while packets flow.

## How to fix it

Add `PersistentKeepalive = 25` to the `[Peer]` section on the device that is behind NAT.

## On a real line

A home server behind an ISP's NAT always needs this. A server with its own public address does not.

## Go deeper

- [WireGuard quick start](https://www.wireguard.com/quickstart/): "NAT and firewall traversal persistence".
