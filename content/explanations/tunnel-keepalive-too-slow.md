## What happened

The tunnel does send keepalive packets, but less often than a NAT on its path forgets an idle connection. Between two keepalives the NAT removes its row and inbound packets stop arriving.

## The concept

Each NAT on the path has its own idle timeout. The keepalive interval has to be shorter than the *shortest* of them, or the row is gone before the next keepalive arrives.

## Why it matters

The tunnel will work whenever you are actively using it and fail after it has been quiet. That makes the fault look random.

## How to fix it

Lower the interval. 25 seconds is the usual choice because it is under the 30-second timeout that strict NATs use.

## On a real line

You usually cannot find out your ISP's timeout. Use 25 seconds and move on.

## Go deeper

- [WireGuard quick start](https://www.wireguard.com/quickstart/): "NAT and firewall traversal persistence".
