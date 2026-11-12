## What happened

The tunnel does send keepalive packets. But it sends them less often than a NAT on its path forgets an idle connection. Between two keepalives, the NAT deletes its row, and incoming packets stop arriving.

## The concept

Each NAT on the path has its own **idle timeout**: how long it waits before deleting an unused row. The keepalive interval must be shorter than the *shortest* timeout on the path. Otherwise the row is gone before the next keepalive arrives.

## Why it matters

The tunnel works while you are using it, and fails after it has been quiet. That makes the fault look random.

## How to fix it

Lower the interval. 25 seconds is the usual choice. It is under the 30-second timeout that strict NATs use.

## On a real line

You usually cannot find out your ISP's timeout. Use 25 seconds and move on.

## Go deeper

- [WireGuard quick start](https://www.wireguard.com/quickstart/): "NAT and firewall traversal persistence".
