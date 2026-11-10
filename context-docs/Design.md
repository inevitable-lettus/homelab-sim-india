# Design (UX)

Screens, behaviours and teaching rules. Motion lives in [Animations.md](Animations.md). **No visual tokens live here:** when the UI is built they go in `app/src/styles/tokens.css`, and motion values in `app/src/lib/motion.ts`.

## What this product is

A free, India-specific homelab learning simulator. Learners complete missions on a simulated network, use a real Linux terminal, and see exactly why things work or fail. The first track teaches **remote access on Indian ISP lines**, where CGNAT makes the usual advice wrong.

**Learners:** college students and existing hobbyists. Both use the same missions at different depths.

## Screen inventory

| Screen | Who | What it does |
|---|---|---|
| Landing | Everyone | States the promise in one line, plays one trace of a blocked connection once (with replay), and starts Mission 1 with no signup. |
| Track map | Everyone | Remote-access missions in order, with progress. Locked-looking future tracks are labelled "planned", with links to the curriculum. |
| Mission | Everyone | The core screen: brief, board, terminal or file editor, findings. Details below. |
| Debrief | Everyone | After a mission: what you did, the concept in plain language, what to do on a real line, next mission. |
| Line check | Everyone | Guided check of the learner's own connection. Feeds the sim for that session. Details below. |
| ISP profile | Everyone | One ISP's dated claims, confidence, sources and "last checked". |
| Concept page | Everyone | A shareable static page for one explanation. |
| Progress and settings | Everyone | Depth mode, electricity rate, export and import progress, clear data. |
| Credits and source | Everyone | Source link for the running version, licences, third-party list. |

Sandbox free-build is not in the first release. The mission board is built so it can become one later.

## Mission screen

Desktop: brief and findings on the left, board in the centre, terminal or editor docked below. Under 768px: one panel at a time with a bottom switcher (Brief, Board, Terminal, Findings). The board stays usable at 360px width.

A mission has:

1. **Scenario** in one or two lines, in an Indian household or hostel context.
2. **Goal**, which can be checked by the sim. "Open Jellyfin from your phone on mobile data."
3. **Constraints**, such as ISP profile, no static IP, or no monthly spend.
4. **Board**, pre-placed. Learners change what the mission is about, not everything.
5. **Terminal task**, where the mission needs one.
6. **Run**, which plays the connection attempt as a trace.
7. **Result**: win, or the exact hop that failed and why.

Learners should fail at least once in most missions, on purpose. The first obvious attempt (a port forward) failing behind CGNAT is the lesson.

## Depths

One set of missions and two depths, switchable at any time:

| | Guided (default) | Challenge |
|---|---|---|
| Brief | Goal and suggested approach | Goal and constraints only |
| Hints | Available step by step | One, on request |
| Explanations | Shown on failure | Collapsed, available after finishing |
| Extra constraints | None | Harder variant, such as no third-party tunnel |

Every mission must be written for both before it ships.

## Findings and verdict language

Findings are ranked: errors first, then warnings, then information. Show at most three at once, with the rest behind "more". Rules about things missing (no backups) appear only in the debrief or when the mission concerns them, so they do not nag beginners into overbuilding.

| State | Wording | Never say |
|---|---|---|
| Pass, official source | "Listed by the maker" | "Compatible" |
| Pass, community source | "Reported to work, N reports" | "Works" |
| Pass, teaching model | "In this model" | Any claim about a real product |
| Fail | The specific reason and hop | "Error" alone |
| Unknown | "We don't have data for this" | Silence, or a pass |
| Learner's own line | "Based on your line check" | Claims about the ISP in general |

ISP statements are always dated and qualified: "Usually behind CGNAT, reported 2026", never "Jio is CGNAT".

## Explanation format

Each explanation is written once in `content/explanations/` and reused:

- **What happened:** one sentence about this learner's situation.
- **The concept:** plain language, with an everyday Indian analogy where it helps. A society gate with one shared address works for CGNAT.
- **Why it matters:** the real consequence.
- **How to fix it:** options, cheapest first.
- **On a real line:** what to actually do outside the sim.
- **Go deeper:** official docs and community sources.

English only at launch. Short sentences. Define every term on first use. Reviewed like code.

## Line check

The learner's real answer beats our data.

1. Open the router page and find its WAN IP (help for common Indian ISP routers).
2. Open any "what is my IP" site.
3. Answer: same, different, or can't find it.
4. Optional: does the WAN IP start with `100.`, `10.`, or `192.168.`?
5. The sim sets the line to public, behind carrier NAT, or unknown.

**Never ask learners to type or paste an IP address.** The app stores only the outcome, on the device. "Can't find it" is a valid answer and yields `Unknown`.

## Remote-access track (draft)

Starting list. Detailed specs belong in `curriculum/`.

| # | Mission | Concept |
|---|---|---|
| 1 | Which address is yours? | LAN, WAN and public addresses; run the line check |
| 2 | Open a door | Port forwarding on a line with a public IP |
| 3 | The wall you can't see | CGNAT: why the same forward fails |
| 4 | Dial out to get in | Outbound connections, mappings, overlay VPN |
| 5 | Write the tunnel | WireGuard config in the real terminal, graded by the sim |
| 6 | Share a website | Tunnel connector for public access, and its trade-offs |
| 7 | The IPv6 side door | Inbound IPv6, the firewall, and clients without IPv6 |
| 8 | Rent a front door | Relay through a low-cost VPS |
| 9 | Power cut at 2 AM | Outage, UPS or inverter, auto-start, does access come back? |

## India-specific behaviour at launch

- **ISP realities:** profiles and line check, as above.
- **Power realities:** state tariff presets in ₹/kWh (editable), outage scenarios, UPS versus inverter behaviour, restore-on-power settings.
- **Not at launch:** rupee hardware prices and sellers (no open data source, goes stale), Hindi or Hinglish.

## Device and network constraints

Assume many learners use **phones, low-end Android and metered data**. This is an assumption to test in Phase 1.

- The core experience needs no large download.
- The terminal is optional per session and states its download size before starting. It falls back to the file editor.
- Touch: tap port, then tap port, to connect. Dragging is an extra, never the only way.
- Tap targets of at least 44px. No horizontal page scroll at 360px.

## Visual direction

Minimal and technical. Flat device cards with labelled port sockets and no decorative hardware art. Effort goes into clarity of the board and the trace. It must not read as AI-generated: avoid generic gradient hero pages, card grids of feature blurbs, and glow. Use the `impeccable` and `design-taste-frontend` skills when building UI.

Light and dark themes both supported. Colour never carries meaning alone: pass, fail and unknown each also have an icon and a word.

## Accessibility

The trace has a complete text form. Board actions work by keyboard and have screen-reader names. Contrast meets WCAG AA. Reduced motion is a first-class mode ([Animations.md](Animations.md)).

## Neutrality

No ISP, seller or sponsor can influence a verdict. Any future affiliate link or sponsorship must be disclosed and kept apart from verdicts.
