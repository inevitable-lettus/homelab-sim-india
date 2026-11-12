## What happened

The packet reached the right device. But no program there is listening on that port, so the device refused it.

## The concept

- An **address** gets a packet to a device.
- A **port** is a number that gets the packet to one program on that device.

If no program is using the port, there is nobody to give the packet to.

## How to fix it

1. Check that the service is running.
2. Check that you dialled the port the service really uses.
3. If you forwarded a port on the router, check that the forward's inside port matches the service's port.

## On a real line

On the server, run `ss -tlnp`. It lists which programs are listening on which ports.
