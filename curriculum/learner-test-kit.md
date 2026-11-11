# Learner test kit (Phase 6)

Licensed CC BY-SA 4.0. See `curriculum/LICENSE.md`.

A script for watching about five real learners use Missions 1 to 3, and for checking whether they can explain carrier NAT (CGNAT) afterwards. Nobody has tested this product with a beginner yet. This kit exists so that the first test is the same for every learner, and so its result can decide whether the project continues as planned.

**What this kit cannot do:** it cannot run itself. A person has to sit with each learner. Until that happens, Phase 6 has no findings.

## What we want to find out

| # | Question | Decided by |
|---|---|---|
| 1 | After Missions 1 to 3, can a learner explain in their own words why a port forward fails behind CGNAT? | The explain-back, scored with the rubric below |
| 2 | Can they finish Missions 1 to 3 without help? | Task completion and help count |
| 3 | Where do they get stuck, and is it the lesson or the interface? | Observation notes |
| 4 | Does it work on the device they actually own? | Device notes |
| 5 | Do they believe the simulator? ("In this model", "We don't have data for this") | Closing questions |

## Who to test with

Five people, one at a time, about 40 minutes each. Aim for this mix:

- Three college students who have never set up a server. At least two should use their own phone, not a laptop.
- Two hobbyists who run something at home already. At least one should be on a line where port forwarding did not work for them.

Do not test with people who helped build this, or with network engineers.

## Before each session

- Use the public site, in the learner's own browser, on the learner's own device and connection. Do not use a build on your machine.
- Open **Settings** on their device and choose **Clear all data**, so everyone starts from the same place.
- Leave the depth on **Guided**. For a hobbyist who finishes early, switch to **Challenge** for Mission 3 and note it.
- Have this kit, a timer, and one note sheet per learner (the template is at the end).

### Consent and privacy

Say this, and get a yes, before starting:

> "I am testing a learning tool, not you. Nothing you do can be wrong. I will take notes on what you do and say. I will not record your name, your face, your screen, or anything about your internet connection. You can stop at any time."

Rules for the facilitator:

- Record no names. Use `L1` to `L5`.
- **Never write down an IP address**, an ISP account detail, or a router password, even if the learner shows you one. The product never asks for these either.
- No screen or voice recording unless the learner asks for it themselves. Notes on paper or in a local file are enough.
- Notes stay with the project owner. Only the summary table goes into `context-docs/Handoff.md`.

## Session script

Read the parts in quotes aloud. Do not explain networking at any point, even if asked. If asked, say: "What do you think? Try it and see."

### 0. Warm-up (3 minutes)

> "Have you ever tried to reach something at your home from outside, like a camera, a game server or files on a computer? What happened?"

Then the **before** question. Write the answer down as close to word for word as you can.

> "A friend says: 'I set up port forwarding on my router exactly like the video said, and I still can't reach my server from my phone on mobile data.' What could be going on?"

Do not react to the answer.

### 1. Landing page (2 minutes)

Give them the address and nothing else.

> "Have a look at this page. Tell me what you think this site is for. Then do whatever you would do next."

Observe: do they read the headline, do they watch the small trace, do they find **Start Mission 1** without help.

### 2. Mission 1: Which address is yours? (8 minutes)

> "Please do this mission. Think aloud as you go: say what you are looking at and what you expect to happen."

Task counts as done when the debrief appears.

Observe:

- Do they press **Run** before changing anything?
- Do they notice that the packet's "from" address changes at the router and at the mobile network?
- Do they tap the boxes on the board to compare addresses?
- On a phone: do they find the **Brief / Board / Findings** switcher, and do they know which panel they are on?

### 3. Mission 2: Open a door (8 minutes)

> "Now the next one."

Observe:

- Do they run the failing attempt first, and do they read why it failed?
- Can they fill in the port forward form? Which field do they get wrong first?
- When it works, do they look at the reply leg, the round-trip time and the film transfer time?

### 4. Mission 3: The wall you can't see (10 minutes)

> "And the third."

This is the mission the product stands on. Observe closely:

- Do they see that the packet stops at the **Home ISP** box and not at the router? Ask nothing. Watch where they look.
- Do they try to fix it by changing the port forward? (Expected, and useful. Let them.)
- Do they run the second attempt, "Ask a what-is-my-IP site", and compare its answer with the router's WAN address?
- Do they open **Why, and what to do**? How far do they read?
- Which answers do they pick, and in what order?

If a learner is stuck for more than three minutes with no new idea, give **one** nudge and count it: "Is there a hint on the page?" If still stuck after two more minutes, tell them the step, mark the task as **helped**, and move on.

### 5. Explain-back (5 minutes)

Close the laptop lid or ask them to put the phone face down. Then ask the **after** question, the same one as before:

> "Same friend, same problem: port forwarding is set up exactly right, and it still does not work from mobile data. What would you tell them now?"

Then, only after they have finished:

> "How could your friend check whether that is what is happening on their line?"
>
> "What could they do instead?"

Write the answers down word for word. Score them later, not in the room.

### 6. Closing questions (4 minutes)

> "What was the most confusing moment?"
>
> "Was there a point where you did not believe what the site showed you?"
>
> "The site says things like 'In this model'. What do you think that means?"
>
> "Would you do the next mission? Why, or why not?"
>
> "What device and what kind of internet do you normally use for something like this?"

Thank them. Do not explain the answers until all sessions for the day are over, in case learners talk to each other.

## Scoring the explain-back

Score the **after** answer against five points. Give a point only if the learner said it without being led. Their words do not have to be technical: "the internet company's own router" counts for point 1.

| Point | The learner says, in any words | 1 if said |
|---|---|---|
| A | There is a second NAT, or a shared address, **at the ISP**, outside the home | |
| B | So the address the router has is **not a public address** (the WAN address and the address the internet sees are different) | |
| C | Packets coming in are dropped **at the ISP, before they reach the router**, so the port forward never gets a chance | |
| D | A way to **check**: compare the router's WAN address with a what-is-my-IP site, or look for a `100.64` to `100.127` address | |
| E | A way **around** it that starts from inside: a tunnel or mesh VPN that dials out, or asking the ISP for a public address | |

| Score | Reading |
|---|---|
| A, B and C all present | **Can explain CGNAT.** This is the bar |
| Two of A, B, C | Partly. Note which one is missing: it points at the animation or text that is not landing |
| One or none | Cannot explain it yet |

D and E are extra. Mission 3 teaches D. Mission 4 teaches E, so a learner who stopped after Mission 3 is not expected to have E.

Also score the **before** answer the same way. A learner who already scores A, B and C before starting tells us nothing about the product. Note it and count them separately.

Common wrong models to listen for, and write down when heard:

- "The router's firewall is blocking it." (They did not see where the packet stopped.)
- "The port is wrong." (They are still debugging the forward.)
- "Mobile data blocks it." (They saw the mobile network's NAT and blamed the wrong side.)
- "The ISP blocks that port." (Close, but it is not port blocking. Nothing inbound has a mapping.)

## What decides the gate

Phase 6 is a gate. Write the result in `context-docs/Handoff.md` and let the owner decide.

| Result across the learners who did not already know | Suggested reading |
|---|---|
| Four or five reach the bar | The core bet holds. Continue |
| Two or three reach the bar | The idea works but something is not landing. Fix what the notes point at, and test again with new learners before building more missions |
| None or one reaches the bar | Stop and rethink the approach before more is built |

Separately, any of these is a finding to fix no matter what the scores are:

- A learner could not finish a mission without being told the step.
- A learner on a phone could not use the board.
- A learner read "In this model" or "We don't have data for this" as the site being broken.
- The page did not load, or was too slow to use, on a learner's own connection.

## Note sheet (one per learner)

```
Learner:            L_      Group: student / hobbyist
Device and browser:
Connection:         home broadband / mobile data / campus Wi-Fi   (no addresses, no account details)
Depth:              guided / challenge

BEFORE answer (word for word):

Score before:       A_ B_ C_ D_ E_

Landing: understood what the site is for?   yes / partly / no
         found Start without help?          yes / no

Mission 1   done alone / done with a nudge / helped      minutes: __
  notes:
Mission 2   done alone / done with a nudge / helped      minutes: __
  first wrong field in the forward form:
  notes:
Mission 3   done alone / done with a nudge / helped      minutes: __
  saw the packet stop at the ISP?           yes / no
  tried to fix the forward first?           yes / no
  ran the what-is-my-IP attempt?            yes / no
  opened "Why, and what to do"?             yes / no
  notes:

AFTER answer (word for word):

How to check (word for word):

What to do instead (word for word):

Score after:        A_ B_ C_ D_ E_
Wrong models heard:

Most confusing moment:
Did not believe the site when:
"In this model" means, to them:
Would do the next mission?   yes / no, because:

Interface problems seen (what, where, on which device):
```

## Summary table for Handoff

Copy this into `context-docs/Handoff.md` under Phase 6 when the sessions are done.

```
| Learner | Group | Device | Before (A B C) | After (A B C) | M1 | M2 | M3 | Main problem seen |
|---|---|---|---|---|---|---|---|---|
| L1 | | | | | | | | |
| L2 | | | | | | | | |
| L3 | | | | | | | | |
| L4 | | | | | | | | |
| L5 | | | | | | | | |

Reached the bar (of those who did not already know): _ of _
Decision by the owner: continue / change / stop
```
