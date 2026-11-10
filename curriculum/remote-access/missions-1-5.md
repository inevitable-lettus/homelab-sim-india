# Remote-access track: Missions 1 to 5 (short specs)

Licensed CC BY-SA 4.0. See [../LICENSE.md](../LICENSE.md).

These are the working specs the code was built against. Phase 2 expands them into full curriculum pages. The mission files in `missions/<id>/mission.yaml` are the source of truth for wording, layout and goals; this page says what each mission is *for*.

All five share one cast and one board, so each mission changes one thing:

| Box | What it is |
|---|---|
| Phone on mobile data | The device outside the home. Always behind the mobile network's carrier NAT |
| Mobile network, Internet | The path from the phone |
| What-is-my-IP site | A public website, used to see the home's public address |
| Home ISP | Public in Missions 1 and 2, carrier NAT from Mission 3 on. A teaching model, not a real provider |
| Home router | One LAN, NAT, port forwards |
| Jellyfin server | An old laptop at `192.168.1.20`, Jellyfin on TCP 8096 |

Addresses come from documentation ranges (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`) and the shared range `100.64.0.0/10`. Distances and link speeds are a teaching model and are labelled as estimates in the UI.

## 1. Which address is yours? (`which-address`)

- **Concept:** a home has three addresses: LAN, WAN and public.
- **Learner does:** dials the server's home address from the phone and watches it die on the mobile network. Dials the WAN or public address and sees the packet reach the router. Runs the what-is-my-IP attempt and compares.
- **Intended failure:** the `192.168` address from outside. Reason: `private-address-not-routable`.
- **Win:** a packet from the phone reaches the home router, and the learner answers that WAN and public are the same on this line.
- **Checks:** `reaches` on the router, and a `line_type` answer that the core derives from the world.
- **Challenge depth:** same goals, no step-by-step brief, one hint.

## 2. Open a door (`open-a-door`)

- **Concept:** port forwarding on a line with a public address.
- **Learner does:** adds a forward on the home router for TCP 8096 to `192.168.1.20:8096`.
- **Intended failure:** no forward, so the router drops the packet (`router-no-port-forward`). Wrong port, wrong home address and wrong inside port each fail for their own reason.
- **Win:** the phone opens Jellyfin and the reply returns. The result shows the round-trip time and how long a 1.5 GB film would take over the slowest link.
- **Debrief only:** the service is now open to the whole internet.

## 3. The wall you can't see (`the-wall`)

- **Concept:** carrier-grade NAT. The same forward that worked in Mission 2 fails, and the router's settings page gives no sign of it.
- **Learner does:** runs the same attempt with the forward already in place. Watches the packet stop at the Home ISP, never reaching the router. Runs what-is-my-IP and sees two rewrites.
- **Intended failure:** the whole mission. Nothing the learner can change on the board makes the forward work.
- **Win:** the learner names the box that stopped the packet (Home ISP) and says that WAN and public addresses differ.
- **Checks:** `blocked_at` and `line_type` answers, both derived from the trace and the world, not stored in the mission.

## 4. Dial out to get in (`dial-out`)

- **Concept:** an outbound packet leaves a row in every NAT it crosses, and replies ride back in on it. A mesh VPN uses this.
- **Learner does:** joins the server and the phone to the mesh, then dials the server's mesh address.
- **Intended failures:** dialling the public address still stops at the ISP. Joining only one device gives `overlay-peer-not-joined`.
- **Win:** the phone opens Jellyfin through the relay, ten minutes after the server last sent anything by hand. The trace shows both devices dialling out first.
- **Model limit, stated in the debrief:** traffic always goes through the relay. Real mesh VPNs often manage a direct path.

## 5. Write the tunnel (`write-the-tunnel`)

- **Concept:** the same idea, built by hand with WireGuard and a rented server that has a public address.
- **Learner does:** in the real terminal (or the file editor) generates a key pair, writes `/root/wg0.conf` for the home server, and registers the public key with the hub.
- **Graded from the files:** the config parses; its private key matches the registered public key; the peer's endpoint is the hub's public address and port; `AllowedIPs` covers the tunnel network; `PersistentKeepalive` is set and shorter than every NAT timeout on the path.
- **Intended failures:** no keepalive (works at first, fails after ten idle minutes), wrong endpoint, `AllowedIPs` too narrow, a public key that does not match.
- **Win:** the phone reaches Jellyfin through the hub ten minutes after the tunnel was last used.
