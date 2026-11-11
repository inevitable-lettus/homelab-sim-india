# Data backlog

Every piece of data the sandbox ([Sandbox.md](Sandbox.md)) and the remote-access track need, written so agents can work through it slowly, one item at a time. Created 2026-11-11.

Schema shapes are in [Sandbox.md §4](Sandbox.md#4-data-packs-schema-v4). Until S1 lands, the schema is the spec, not code: **agents may research and draft packs before S1, but files only go into `data/` once `npm run data` can validate them.** Drafts go in `data-drafts/<category>/<id>.yaml` (gitignored is fine; or a branch), and move across when S1 is in.

## Rules for every data agent

Read these before each item. They come from the hard rules in `CLAUDE.md`.

1. **One pack per change.** One YAML file, one commit, signed off (`git commit -s`). Commit message: `Add <category> pack: <label>`.
2. **Every value has a source and a retrieval date, or is `unknown`.** `retrieved_at` is the ISO date you read the page. Never fill a value from memory, from another pack, or by reasoning "it's probably the same as the previous model".
3. **Source order, best first:**
   1. `official`: maker's spec page, datasheet, user manual, Lenovo PSREF, Apple Tech Specs, Dell/HP spec sheets, the software project's own docs.
   2. `review`: a named independent reviewer's measurement (power draw measured at the wall, for example). Name the page, not the site's home page.
   3. `community`: forum thread, wiki, issue tracker, Reddit. Only when nothing better exists, and only with a durable link.
4. **Exact model or exact configuration.** Laptops and mini PCs come in many configurations under one name. One pack = one configuration (CPU, RAM, storage as sold). Put the model number in `aka`. If the source covers several configurations, write the values that are common to all and leave the rest `unknown`, or make one pack per configuration.
5. **`unknown` is a good answer.** A pack with ten honest unknowns is better than one guess. The sandbox shows unknowns and asks for help, which is how the data gets better.
6. **No prices, no seller or affiliate links, no tracking parameters** (`utm_`, `tag=`, `affid`, `ref=`) in any source URL. The data build rejects them.
7. **No copied text.** Facts only, in our own words. `data/` is CC BY-SA 4.0 and maker text is not.
8. **India availability (`india`)** needs an official India page for `sold_new` (maker's India store or support page, or an authorised distributor named by the maker). `used_common` is `community` basis and needs a durable page, never a single marketplace listing.
9. **Never invent a real ISP claim.** ISP profiles follow the Phase 3 rule: dated, sourced, with a confidence, or `unknown`.
10. **Check before marking done:** `npm run data` passes, the pack's id matches the file name, every source opens, and the item's checkbox below is ticked in the same commit with the date.

### Status marks

`[ ]` not started · `[~]` drafted, waiting for S1 or for review · `[x]` merged (add the date). Priorities: **P0** needed for S5 templates, **P1** launch, **P2** after launch, **P3** nice to have.

## A. Schema-level reference data

| | Item | Priority | What | Where to look |
|---|---|---|---|---|
| [ ] | `data/reference/media.yaml` | P0 | Typical bitrates for `h264_720p`, `h264_1080p`, `hevc_1080p`, `hevc_2160p_8bit`, `hevc_2160p_10bit`, `av1_1080p`, `av1_2160p`; audio ignored | Jellyfin and Plex documentation on bandwidth and transcoding |
| [ ] | UPS usable fraction constant | P0 | A defensible usable share of a lead-acid UPS battery's rated Wh, with inverter loss, for `UPS_USABLE_PERMILLE` | Battery makers' depth-of-discharge guidance; UPS makers' runtime charts (back-calculate from two) |
| [ ] | Laptop usable fraction constant | P0 | `LAPTOP_USABLE_PERMILLE` | OS battery-reserve docs; laptop makers' battery guidance |
| [ ] | Battery always-charging guidance | P0 | Sourced statement for the `battery-always-charging` explanation: wear and swelling with sustained full charge, and charge-limit features | Apple battery and optimised charging docs, Lenovo charge-threshold docs, Dell/HP equivalents, Battery University (review basis) |
| [ ] | PSU hold-up and inverter transfer | P2 | Why transfer time and waveform matter to desktop PSUs | ATX design guide; UPS maker FAQs |

## B. Archetypes (teaching models, unsourced values allowed)

Revise the three existing archetypes to schema v4 in S1. Add these (all **P0** unless noted). Values must be typical and plausible, and the label must say it is a teaching model.

| | Id | Models |
|---|---|---|
| [ ] | `old-laptop` (revise) | x86 laptop, 8 GB, 1 GbE, battery, Wi-Fi 5 |
| [ ] | `old-laptop-2013` | Dual-core x86, 4 GB DDR3, 2.5" SATA bay, 100 Mbit or 1 GbE, BIOS |
| [ ] | `consumer-laptop-2018` | Quad-core x86, 8 GB, M.2 slot, no Ethernet port, Wi-Fi 5 |
| [ ] | `apple-silicon-laptop` | aarch64, 8 GB soldered, no Ethernet, 2× USB-C 40G, fanless, battery |
| [ ] | `single-nic-mini-pc` (revise) | Low-power x86 quad-core, 16 GB, 1 GbE, M.2 + 2.5" |
| [ ] | `corporate-tiny-pc` | 6th–8th gen x86, 8 GB, 1 GbE, M.2 + 2.5", AC power recovery in BIOS |
| [ ] | `single-board-computer` (revise) | aarch64, 4 GB, 1 GbE, SD card, USB 3, 5 V USB-C power |
| [ ] | `old-android-phone` | aarch64, 3 GB, no Ethernet, Wi-Fi 5, battery, `android_bootloader` |
| [ ] | `old-desktop-tower` | x86, 8 GB, 1 GbE, 2× SATA 3.5", AC power recovery |
| [ ] | `two-bay-nas` | P1: aarch64 or x86 NAS appliance, 2× 3.5" |
| [ ] | `isp-combo-router` | ONT + router, 4× 1 GbE, Wi-Fi 5, limited admin, port forwarding unknown |
| [ ] | `retail-router` | 4× 1 GbE, Wi-Fi 6, full admin, DNS setting, DHCP disable |
| [ ] | `unmanaged-switch-5` / `-8` | 5 / 8 ports, 1 GbE |
| [ ] | `line-interactive-ups-600va` | ~360 W, small lead-acid battery, stepped sine |
| [ ] | `home-inverter-900va` | External battery, sine |
| [ ] | `dc-mini-ups-12v` | 12 V and 9 V outputs, small lithium pack |
| [ ] | `hdd-3-5`, `hdd-2-5`, `ssd-sata`, `ssd-nvme` | Generic drives with `capacity_gb` set per template |
| [ ] | `usb-sata-enclosure-uas`, `usb-nvme-enclosure` | |
| [ ] | `usb-gbe-adapter`, `usb-2-5gbe-adapter` | |
| [ ] | `usb-c-hub-ethernet` | USB-C hub with 1 GbE and USB-A |
| [ ] | `smart-tv`, `streaming-stick`, `phone-client` | Client archetypes with typical codec support |

## C. Real computers

All `category: computer`. Fields: every `compute`, `network`, `ports`, `storage`, `power`, `battery` value, `form`, `released`, `india`, and `os_support` entries for every OS in section E that the sources speak to.

Typical sources: Apple Tech Specs (support.apple.com), Lenovo PSREF, Dell and HP spec sheets and service manuals, Raspberry Pi product briefs and datasheets, Asahi Linux feature-support pages, Notebookcheck (review basis) for measured power.

### C1. Apple (P0 for the M1 Air, P1 others)

| | Id (suggested) | Notes for the agent |
|---|---|---|
| [ ] | `macbook-air-m1-2020-8gb-256` | P0. The template device. `nic: []`, USB-C 40G ×2, fanless, battery Wh, soldered RAM and SSD. `os_support`: macOS (full), `fedora-asahi-remix` (partial; list gaps), generic Linux (none) |
| [ ] | `macbook-air-m1-2020-16gb` | P1 |
| [ ] | `macbook-air-m2-2022` | P1. Check Asahi support status at retrieval date |
| [ ] | `macbook-air-m3-2024` | P2. Asahi status likely `none` at time of writing; source it |
| [ ] | `mac-mini-m1-2020` | P1. Apple publishes Mac mini power consumption: use it. Has Ethernet |
| [ ] | `mac-mini-m2-2023`, `mac-mini-m4-2024` | P2 |
| [ ] | `macbook-pro-13-2015` | P1. Intel, common used. Thunderbolt 2, no Ethernet |
| [ ] | `macbook-air-13-2017` | P1. Intel, common used |

### C2. Used business laptops (P1, P0 for one)

| | Id | Notes |
|---|---|---|
| [ ] | `thinkpad-t480` | P0 candidate for the old-laptop template. PSREF has everything except power draw |
| [ ] | `thinkpad-t440`, `thinkpad-t450`, `thinkpad-t460`, `thinkpad-t470` | |
| [ ] | `thinkpad-x250`, `thinkpad-x260`, `thinkpad-x270` | |
| [ ] | `thinkpad-e14-gen1` | |
| [ ] | `latitude-e7450`, `latitude-e7470`, `latitude-5490`, `latitude-7490`, `latitude-e6430` | |
| [ ] | `elitebook-840-g3`, `elitebook-840-g5`, `probook-440-g5` | |

### C3. Common consumer laptops in India (P2)

Consumer models vary by SKU. Only add one when a source gives the exact SKU's specs. Families to look at: Dell Inspiron 15 3000 series, Dell Vostro 3000 series, HP 15 and Pavilion 15, Lenovo IdeaPad 320/330/S145, Acer Aspire 3/5, ASUS VivoBook 15. Until then the `consumer-laptop-2018` archetype covers them.

### C4. Used corporate tiny PCs (P1)

| | Id | Notes |
|---|---|---|
| [ ] | `optiplex-3040-micro`, `optiplex-3050-micro`, `optiplex-3060-micro`, `optiplex-3070-micro` | |
| [ ] | `optiplex-7040-micro`, `optiplex-7050-micro` | |
| [ ] | `thinkcentre-m700-tiny`, `thinkcentre-m710q-tiny`, `thinkcentre-m720q-tiny`, `thinkcentre-m920q-tiny` | One of these is the P0 candidate for `mini-pc-proxmox` |
| [ ] | `elitedesk-800-g3-mini`, `prodesk-600-g3-mini`, `elitedesk-800-g4-mini` | |

### C5. New mini PCs (P1)

Intel N100 / N150 class boxes sold on Indian marketplaces. Pick models where the maker publishes a spec page: e.g. Beelink, GMKtec, ASUS NUC, Intel NUC (older, used). One pack per configuration. Note `video.encode` for QSV codecs from Intel's own processor pages.

### C6. Single-board computers (P0 for one Pi)

| | Id | Notes |
|---|---|---|
| [ ] | `raspberry-pi-4-4gb`, `raspberry-pi-4-8gb`, `raspberry-pi-4-2gb` | P0: one of these for `pi-hole-whole-house`. Official India distributors for `india` |
| [ ] | `raspberry-pi-5-4gb`, `raspberry-pi-5-8gb`, `raspberry-pi-5-16gb` | P1. Note the Pi 5 has no hardware H.264 encoder: source it |
| [ ] | `raspberry-pi-zero-2-w` | P2 |
| [ ] | `orange-pi-5`, `radxa-rock-5b` | P2. `os_support` mostly community |

### C7. Phones as servers (P2)

Archetype first. Real phones only where postmarketOS or a well-documented Termux setup is sourced for that exact model.

### C8. NAS appliances (P2)

Synology two-bay models and TerraMaster equivalents sold in India.

## D. Network gear

### D1. ISP-supplied routers and ONTs (P0 for at least one, P1 rest)

The most India-specific data in the sandbox. Many learners cannot change DNS, forward ports, or use bridge mode on the unit their ISP gave them, and nobody documents it in one place. Fields: everything in `RouterCaps`, plus `supplied_by` (ISP profile id).

For each ISP below, first find **which units are currently supplied** (it changes, and differs by plan and circle), then make one pack per unit. Expect `community` basis for most admin-feature claims, with the firmware version and date in `note`.

| | ISP | Notes |
|---|---|---|
| [ ] | JioFiber / JioAirFiber | Jio-branded home gateways. Admin access, port forwarding, DNS setting, IPv6 |
| [ ] | Airtel Xstream Fiber | Several makers' units under Airtel branding |
| [ ] | BSNL FTTH (and its franchise partners) | ONT models vary by franchise |
| [ ] | ACT Fibernet | |
| [ ] | Excitel | |
| [ ] | Tata Play Fiber | |
| [ ] | Hathway | |
| [ ] | Local cable operators | Archetype only: unknown everything |

### D2. Retail routers (P1)

Popular in India, with a maker spec page: TP-Link Archer C6, Archer AX23, Archer AX55; Mercusys Halo / MR-series; Tenda; Xiaomi. Fields from the maker's spec and emulator/manual pages (TP-Link publishes web-UI emulators: those count as `official`).

### D3. Switches and access points (P2)

One or two real unmanaged switches (TP-Link TL-SG105 / TL-SG108). Archetypes suffice otherwise.

## E. Operating systems and hypervisors (`data/os/`)

All **P0** unless noted. Fields: `arches`, `firmware`, `listed_devices_only`, `requires`, `base_ram_mb`, `provides`. Source: the project's own system requirements page.

| | Id | Notes |
|---|---|---|
| [ ] | `ubuntu-server-lts` | |
| [ ] | `debian-stable` | |
| [ ] | `macos` | One pack for current macOS. `native_runtimes: []`: containers need a VM runtime. Virtualization framework accelerates aarch64 guests only on Apple silicon |
| [ ] | `fedora-asahi-remix` | `listed_devices_only: true`. Device support lives in each Mac pack |
| [ ] | `proxmox-ve` | x86_64 only officially; requires virtualization |
| [ ] | `raspberry-pi-os-lite` | |
| [ ] | `windows-11` | P1. TPM and CPU requirements are not modelled; say so in a note |
| [ ] | `windows-10` | P2. End of support date in `notes` |
| [ ] | `truenas-scale`, `openmediavault`, `unraid` | P1. Unraid is paid: `paid_features` is on services, so add a note |
| [ ] | `home-assistant-os` | P1. `kind: appliance` |
| [ ] | `dietpi`, `casaos` (on Debian), `umbrel` | P2 |
| [ ] | `android-termux` | P2. What Termux can and cannot run (no Docker without root) |
| [ ] | `postmarketos` | P3. `listed_devices_only: true` |

## F. Container runtimes (`data/runtimes/`)

| | Id | Priority | Notes |
|---|---|---|---|
| [ ] | `docker-engine` | P0 | Linux native, host networking, device passthrough |
| [ ] | `docker-desktop-mac` | P0 | Via VM; host-networking support has changed across versions: source the current state; licence terms for personal use |
| [ ] | `orbstack` | P0 | Via VM, macOS. Licence for personal use |
| [ ] | `colima` | P1 | Via VM, macOS |
| [ ] | `podman` | P1 | Linux native; Podman machine on macOS as a separate pack `podman-machine-mac` |
| [ ] | `docker-desktop-windows` | P1 | WSL2 backend |
| [ ] | `proxmox-lxc` | P1 | Modelled as runtime for LXC guests |

## G. Services (`data/services/`)

Fields: `install` (methods × OS families × arches), `ram_min_mb`, `ram_rec_mb`, `storage_min_gb`, `ports`, `needs`, `provides`, `paid_features`, `licence`. Source: each project's install and requirements docs, and its container image's published arches.

| | Id | Priority | Watch for |
|---|---|---|---|
| [ ] | `jellyfin` | P0 | Native macOS build exists; transcoding APIs per OS (VideoToolbox, VAAPI, QSV) |
| [ ] | `pi-hole` | P0 | DHCP server feature needs host networking; static IP |
| [ ] | `immich` | P0 | Container only; RAM needs; machine learning part is heavy |
| [ ] | `tailscale` | P0 | Provides `mesh_vpn`. Free plan limits as a dated note |
| [ ] | `wireguard` | P0 | Provides `vpn_endpoint` |
| [ ] | `cloudflared` | P1 | Provides `tunnel_connector`; Cloudflare account terms note |
| [ ] | `plex` | P1 | Remote streaming and hardware transcoding paid features: source the current policy and date |
| [ ] | `adguard-home` | P1 | |
| [ ] | `nextcloud` | P1 | |
| [ ] | `home-assistant` (container) | P1 | Discovery needs host networking |
| [ ] | `vaultwarden` | P1 | |
| [ ] | `syncthing` | P1 | |
| [ ] | `samba` | P1 | Provides `file_share` |
| [ ] | `uptime-kuma` | P2 | |
| [ ] | `paperless-ngx` | P2 | |
| [ ] | `navidrome` | P2 | |
| [ ] | `qbittorrent` | P2 | |
| [ ] | `forgejo` | P2 | |
| [ ] | `nginx-proxy-manager`, `caddy` | P2 | Provides `reverse_proxy` |
| [ ] | `frigate` | P3 | Needs an accelerator; CPU-heavy |
| [ ] | `minecraft-java` | P2 | RAM-heavy; popular with students |
| [ ] | `portainer` | P3 | |

## H. Power

### H1. Tariffs (`data/power/tariffs/`)

Residential (LT domestic) tariffs from each state regulator's (SERC's) latest tariff order or the DISCOM's published schedule. Fields: slabs, fixed charge, duty, free-unit scheme with its condition, effective date. **One file per DISCOM.** Note when a state has several DISCOMs with different tariffs.

| | State / DISCOM | Priority |
|---|---|---|
| [ ] | Karnataka: BESCOM | P0 |
| [ ] | Maharashtra: MSEDCL | P0 |
| [ ] | Delhi: BSES Rajdhani, BSES Yamuna, Tata Power-DDL (one tariff order; free-unit scheme) | P0 |
| [ ] | Tamil Nadu: TNPDCL | P1 |
| [ ] | Telangana: TGSPDCL | P1 |
| [ ] | Kerala: KSEB | P1 |
| [ ] | West Bengal: WBSEDCL; Kolkata: CESC | P1 |
| [ ] | Gujarat: state DISCOMs; Ahmedabad and Surat: Torrent Power | P1 |
| [ ] | Uttar Pradesh: UPPCL DISCOMs | P1 |
| [ ] | Mumbai: Adani Electricity, Tata Power, BEST | P1 |
| [ ] | Andhra Pradesh, Rajasthan, Punjab (free-unit scheme), Haryana, Madhya Pradesh | P2 |
| [ ] | Bihar, Odisha, Assam, Goa, Chandigarh, Jharkhand, Chhattisgarh, Uttarakhand, Himachal Pradesh, J&K | P3 |

### H2. Outage scenarios (`data/power/outages/`)

| | Id | Priority | Kind |
|---|---|---|---|
| [ ] | `evening-cut-2h` | P0 | teaching |
| [ ] | `overnight-6h` | P0 | teaching |
| [ ] | `short-blips` (10 × 1 minute a day) | P0 | teaching |
| [ ] | `scheduled-maintenance-day` (8 h) | P1 | teaching |
| [ ] | State reliability data | P3 | real: only from an official reliability report (regulator or DISCOM), per state |

### H3. UPS, inverters, DC mini UPS (`category: ups | inverter | dc_ups`)

Prefer models whose makers publish a runtime chart.

| | Items | Priority |
|---|---|---|
| [ ] | APC Back-UPS sold in India (BX600C-IN, BX1100C-IN, BVX series) | P0: one; P1: rest |
| [ ] | Microtek, Luminous, V-Guard line-interactive UPS | P1 |
| [ ] | Home inverters: Luminous, Microtek, V-Guard (inverter only; battery separate) | P1 |
| [ ] | Inverter batteries: common tubular 150 Ah class, as `kind: archetype` battery entries unless a maker sheet is used | P1 |
| [ ] | DC mini UPS for routers and ONTs (12 V / 9 V outputs) sold in India | P0: one; P1: two more |

## I. Storage and adapters

| | Items | Priority |
|---|---|---|
| [ ] | NAS-rated HDDs sold in India (WD Red Plus, Seagate IronWolf) at two capacities | P1 |
| [ ] | One common SATA SSD and one NVMe SSD | P2 (archetypes cover it) |
| [ ] | USB 3 SATA enclosures with UAS stated by the maker | P1 |
| [ ] | USB Ethernet adapters sold in India: 1 GbE and 2.5 GbE, with OS support from the maker's driver page (TP-Link UE300 / UE306 class) | P0: one 1 GbE |
| [ ] | USB-C hubs with Ethernet | P2 |
| [ ] | Apple's own USB-C adapters with Ethernet, if any are sold | P3 |

## J. Clients (`category: client`)

| | Items | Priority |
|---|---|---|
| [ ] | Streaming sticks sold in India (Fire TV Stick models, Chromecast with Google TV / Google TV Streamer): codecs from maker spec pages, Jellyfin/Plex app availability from app stores | P1 |
| [ ] | Android TV / Google TV platform as an archetype with codec support left `unknown` per TV | P1 |

## K. ISP profiles (from Phase 3, needed by the sandbox too)

Fixed-line: Jio, Airtel, BSNL, ACT (Phase 3, **P0**), then Excitel, Tata Play Fiber, Hathway (P1). Mobile networks for the away side: Jio, Airtel, Vi, BSNL mobile: IPv4 CGNAT, IPv6 availability (P1). Follow the existing ISP claim rule exactly.

## L. Content for the sandbox

| | Item | Priority |
|---|---|---|
| [ ] | `content/steps.yaml` entries for every reason in [Sandbox.md §7.5](Sandbox.md#75-reasons-summary) | P0 (with S2/S3) |
| [ ] | Long explanations for every ★ reason, six headings each | P0 |
| [ ] | Six templates in [Sandbox.md §12](Sandbox.md#12-templates) | P0 (S5) |
| [ ] | "About these results" text ([Sandbox.md §15](Sandbox.md#15-model-limits-stated-to-the-learner)) | P0 |
| [ ] | GitHub issue form `.github/ISSUE_TEMPLATE/device-data.yml` for "Help add this data": pack id, field, source URL, retrieved date, basis | P0 |
| [ ] | `CONTRIBUTING.md` section: how to add a device in one file, with a worked example | P1 |

## Suggested agent order

1. **A** reference items, and **E** + **F** (small, unblock most rules).
2. **G** P0 services.
3. P0 hardware: `macbook-air-m1-2020-8gb-256`, `thinkpad-t480`, one ThinkCentre Tiny, one Raspberry Pi 4, one USB GbE adapter, one APC UPS, one DC mini UPS, one ISP-supplied router.
4. **H1** P0 tariffs, **H2** P0 outages.
5. Then down the priorities, one file at a time.
