# Animations

Motion in this product is **instructional**. It shows cause and effect in a network: where a packet went, what changed it, and where it stopped. If an animation doesn't teach or confirm something, it doesn't ship.

Values below are what Phase 5 built (2026-11-11). Tune them on real devices, and keep the real numbers in `app/src/lib/motion.ts`, never inline.

## Rules

1. **The core decides, the animation replays.** All network animation is playback of a `Trace` from the Rust core ([Architecture.md](Architecture.md)). No animation code may decide whether a packet gets through.
2. **Motion is scrubbable.** Learners can play, pause, step forward and back, and change speed. The lesson is often in one hop.
3. **Every step has a text line.** The step log is the canonical record. Animation is a view of it.
4. **One thing at a time.** One moving packet by default, with a hard limit of three on screen. Comparisons play in sequence.
5. **Only `transform` and `opacity`.** No animating layout, SVG geometry attributes or filters.
6. **No per-frame React state.** Playback writes to refs and DOM directly.
7. **Same trace, same playback.** Given a trace and a speed, frames are reproducible, which is what makes them testable.

## Pipeline

```mermaid
graph LR
  Core["sim-core trace()"] --> Trace["Trace JSON"]
  Trace --> Timeline["buildTimeline()"]
  Timeline --> Player["TimelinePlayer"]
  Player --> Overlay["SVG/DOM overlay"]
  Player --> Log["Step log"]
  Player --> Highlights["Node and edge highlights"]
```

`buildTimeline(trace, tokens)` is a pure function that returns a list of timed segments. It is unit-tested without a browser.

`TimelinePlayer` (`app/src/lib/player.ts`) owns the clock and exposes `play`, `pause`, `seek`, `showStep`, `stepForward`, `stepBack`, `toEnd`, `reset` and `setSpeed`. A renderer subscribes to frames and writes to the DOM; React subscribes only to step and state changes. Packet position comes from `getPointAtLength` on the React Flow edge's path, and is applied as a `transform` on one overlay element above the board. CSS transitions cover one-off changes. There is no animation library.

**Time on screen comes from the core.** A hop's duration is its share of the slowest hop in the trace (the core's `total_ns`), held between `hopMin` and `hopMax`, so a 900 km backbone hop visibly takes longer than a LAN hop. When queueing is more than 8% of a hop's time, the packet waits at the near end for that share and its label says "waiting in the queue". A trace longer than `cap` at 1x is scaled down evenly, so proportions survive. A hop with no timing from the core uses `hop`. None of this decides anything: the numbers a learner reads are in the step log, straight from the core.

The overlay element lives inside React Flow's viewport (`ViewportPortal`), so it is positioned in flow coordinates and follows pan and zoom with no re-reading. The Phase 1 spike measured 0 px of drift.

**The packet's label has two homes.** On a wide board it rides above the packet on a short stem, high enough to clear the row of cards, counter-scaled by `1 / zoom`, and nudged sideways to stay inside the board. On a narrow (stacked) board a label that travels would cover the cards above the packet, so it sits in a fixed strip at the top of the board and only the packet moves. `fitView` keeps that strip free. A NAT or forwarding table appears under its card on a wide board; on a narrow board the same row is in the step log.

**Cables have slack.** A wired cable is a quadratic curve that sags, and sags more when its cards are dragged closer than they were laid out. Radio links are straight and dotted. The path is recomputed when a card moves, which is a redraw on input and not an animation. Cards cannot be dragged on a narrow board, so a finger scrolls the page instead.

## Tokens (starting values)

| Token | Value | Used for |
|---|---|---|
| `dur.instant` | 80 ms | Press feedback |
| `dur.fast` | 160 ms | Hover, small state changes |
| `dur.base` | 240 ms | Panels, finding appearing |
| `dur.hop` | 520 ms | A hop the core could not time |
| `dur.hopMin`, `dur.hopMax` | 170 ms, 900 ms | Bounds for a timed hop at 1x |
| `dur.forward` | 150 ms | Crossing a box that only forwards |
| `dur.rewrite` | 560 ms | Header value changing at a NAT, or a tunnel wrapping |
| `dur.appear`, `dur.deliver` | 260 ms, 520 ms | First and last step of a leg |
| `dur.drop` | 900 ms | Packet stopping and failing |
| `dur.legGap` | 420 ms | Between legs (setup, request, reply) |
| `dur.cap` | 12 s | Longest trace at 1x before even scaling |
| `dur.win` | 700 ms | Mission success moment |
| `ease.out` | `cubic-bezier(0.22, 1, 0.36, 1)` | Things arriving |
| `ease.inOut` | `cubic-bezier(0.65, 0, 0.35, 1)` | Packet travel |
| `ease.in` | `cubic-bezier(0.55, 0, 1, 0.45)` | Things leaving |
| `speed` | 0.5x, 1x, 2x | Playback |

## Catalogue

| Moment | What the learner sees | Teaches |
|---|---|---|
| Packet hop | A labelled packet travels along a cable to the next node | Path, one hop at a time |
| **NAT rewrite** | Packet pauses at the router; the source address visibly changes: old value struck through above, new value below. Never cross-fade them in the same place | NAT changes addresses |
| **Mapping created** | An outbound packet adds a row to the NAT table at that node | Why replies can return |
| **CGNAT wall** | An inbound packet reaches the ISP's carrier NAT, finds no mapping row, and stops there, short of the home router | The forward at home never had a chance |
| Port forward hit | Inbound packet matches a highlighted forward rule | What a forward does |
| Tunnel build | A connection draws outward from home to the relay first | Outbound-first |
| Tunnel carry | Packet enters a sleeve around the link; the outer addresses show, then it unwraps | Encapsulation |
| IPv6 direct | Packet passes the NAT nodes by a direct path, then meets the firewall | No NAT, firewall still applies |
| Firewall decision | Rule row highlights, allow or deny | Which rule decided |
| DNS lookup | Short query out and answer back before the main packet starts | Names resolve first |
| Drop | Packet stops at the exact hop, marked, with the reason opening beside it | Precise failure location |
| Service boot | Dependencies check in order: OK, degraded or failed | Startup order |
| Power cut | Mains line goes out, unprotected devices dim, UPS or inverter takes over, runtime drains | What survives an outage |
| Power restored | Machines return only if restore-on-power is set; tunnel reconnects | Unattended recovery |
| Finding appears | Slides 8px and fades in; involved nodes pulse once | Links finding to the board |
| Mission win | Delivered packet settles, one restrained confirmation | Completion |
| Grade from files | The learner's file lines light up as the sim reads them into the config | Your file changed the network |

The NAT rewrite, mapping and CGNAT wall are the most important animations in the first track, so they get the most polish. Labels must be readable at 1x at 360px wide: use short values such as `192.168.1.20`, and collapse ports when space is tight.

## Reduced motion

With `prefers-reduced-motion: reduce`, or the in-app setting:

- Packets do not travel. The current step's node and edge are highlighted, and the step log advances.
- Address changes appear as a static before and after.
- Autoplay is off. Learners step by hand.
- The landing trace does not play by itself. It shows the board and a Replay button that steps.

Every lesson must be fully understandable in this mode. This is a requirement, not a degraded fallback.

## Performance

- Target 60 fps on desktop, and at least 30 fps on a low-end Android phone. The built app holds 60 fps under 6x CPU throttling and 58 fps under 20x, at 1280x800 and at 360x740 with a 3x pixel ratio, on the 27-step Mission 4 trace (headless Chrome on an Apple M4, `node app/e2e/measure-playback.mjs`). Throttling slows script only, so a real phone is still untested.
- One overlay layer for packets. Apply `will-change: transform` only during playback.
- Stop the clock when paused or when the tab is hidden.
- Never run traces or evaluate rules inside a frame callback. Compute first, then play.
- Long sequences cap at about 12 seconds at 1x. Beyond that, split them or skip repeated hops.

## Avoid

- Decorative particles, ambient pulses, background motion, parallax.
- Bouncy or elastic easing on technical content.
- Infinite animations, except a paused-state indicator.
- Page transition effects that delay reaching the mission.
- Linear presentation-style fades.
- Blocking input during playback. Learners can always interrupt.

## Testing

- `app/src/lib/timeline.test.ts` unit tests `buildTimeline` on real mission traces.
- `app/e2e/missions.spec.ts` (Playwright, desktop and 360px) runs a failure and a win, asserts the step log equals the core's trace step for step (the test loads the same WASM in Node to get the expected trace), steps a whole trace in reduced motion, and lets one trace autoplay to its end.
- Not built: visual snapshots in light and dark. Screens were checked by eye in both themes.
