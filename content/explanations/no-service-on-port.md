## What happened

The packet reached the right device, but no program there is listening on that port, so the device refused it.

## The concept

An address gets a packet to a device. A *port* gets it to one program on that device. If no program has claimed the port, there is nobody to hand the packet to.

## How to fix it

Check that the service is running, and that you dialled the port it actually uses. If you forwarded a port on the router, check that the forward's inside port matches the service.

## On a real line

On the server, `ss -tlnp` lists which programs are listening on which ports.
