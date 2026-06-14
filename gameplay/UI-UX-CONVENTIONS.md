# Sojourn — UI/UX Conventions (shared system)

This is the shared UI/UX contract every gameplay increment builds against. Each increment doc (`01-…` … `10-…`) specifies its own screen and subscreens in detail; this document defines the **common grammar** so the twelve screens feel like one instrument, and maps the **full screen → subscreen tree** the playable game needs (the current screens are placeholder templates and must gain this depth).

The renderer is `sojourn-ui-desktop` (egui/eframe, `egui_extras::TableBuilder` for virtualised tables). The view-model is `sojourn-ui` (headless, unit-tested). **All interaction logic that decides consequences lives below the renderer** (in `sojourn-game` / the core); the renderer only collects intents, shows core-computed previews, and submits.

The visual reference is the mission-console template already in the repo (`ui-references/`): deep ink surfaces, bracketed instrument panels with equipment codes + status LEDs, reticle-framed map, restrained phosphor glow, mono-forward console type, sharp edges, SI units only, full-bleed System Map with floating HUD.

---

## 1. The persistent shell

Unchanged in structure from FA-10, now made *live* in GP-00:

- **Top bar:** brand · faction callsign (ESA) · simulated date + `T+NNNd` + **run-state** (PAUSED/RUNNING) · time-warp control (pause, 1×, 1 h, 1 d, 10 d, …) · the **six currencies** (Funds, Propellant, Mass→orbit, Crew-time, Ops-cap, Pol-cap) each drilling to its ledger · alerts bell with pending-interrupt count.
- **Left nav:** the twelve screens in three groups (Operate / Develop / Discover), each opening its subscreens (§3).
- **Centre work area:** the active screen / subscreen.
- **Right inspector:** the pinned object's detail + its **context actions** (the verbs that open plan→preview→commit drafts).
- **Bottom event ticker:** the chronological feed, filterable by class, with ⏸ markers on pausing classes.

The shell reads `ShellVM` (status + recent events) and renders interrupts as a modal review (GP-00). It owns no game state.

---

## 2. Interaction grammar (applies to every screen)

1. **Plan → preview → commit for every irreversible action.** The screen collects a *draft Intent*; the player hits **Preview**; `sojourn-game` returns a **core-computed `Preview`** (the traced consequence deltas + what becomes unrecoverable + any failed `Gate`s); the player **Commits**; the renderer submits the batch and surfaces the `CommitOutcome`. Reversible actions (layer toggles, sorts, allocation sliders before commit, lobbying nudges) are `Direct`. This is the existing `command.rs` flow (`ActionKind`, `DraftPlan`, `Preview`, `Gate`, `CommitOutcome`) — wire each new verb to it; never submit an irreversible command without a shown preview.

2. **Traceability everywhere.** Any derived number renders with a trace affordance (the dotted-underline "ⓘ"): hovering/opening shows the `TraceRender` tree down to sourced inputs (e.g. Δv = Isp·g₀·ln(m₀/m_f)). Numbers the UI cannot trace are a bug, not a display choice (Principle VIII).

3. **Belief vs truth, always honest.** Anything the player only *believes* (surveyed grades, astrobiology evidence, AI intentions) shows its **uncertainty** and never leaks ground truth. The astrobiology view-model's honesty guard is the template: never a conclusive positive the core has not set.

4. **Progressive disclosure.** Each screen has an overview tier and detail subscreens; `Disclosure` levels keep the first read calm and the depth one click away. Empty states (fresh ESA: no fleet, no bases, UL 0) read as deliberate "nothing yet — here's how to start," not as broken panels.

5. **Pause-friendly & keyboard-rich.** Nothing forces real-time action; opening a draft does not advance time. Every primary verb has a hotkey; tables are keyboard-navigable; the map is mouse-first with keyboard focus.

6. **SI units only**, colour-blind-safe palette, legible 1280×720 → 4K, virtualised tables for thousands of bodies / large ledgers at high time-warp.

7. **Interrupt review.** When the core pauses on an interrupt, the player reviews the event(s), takes any decision the event offers, and acknowledges; the run-state returns to PAUSED with the queue visible. Pause policy per event class is configurable (GP-09).

---

## 3. The full screen → subscreen map

The twelve top-level screens stay; each grows the subscreens below (introduced by the increment shown). "Inspector" = the right-rail detail for the selected object on that screen.

**S1 System Map** (GP-00 shell → GP-04 live → GP-05 layers)
- Full-bleed heliocentric/planetary map (real ephemerides), floating HUD: layer toggles, tracked-objects list, focus/frame control, scale.
- Layers: trajectories · transport graph · **resources** · comms/DSN · PP zones · science (GP-05).
- Inspector: body/site/craft detail; site shows surveyed properties + uncertainty (GP-05).

**S2 Trajectory & Manoeuvre Planner** (GP-04)
- Subscreens: **Porkchop / window solver** · **Manoeuvre-node editor** · **Low-thrust arc planner** · **Δv budget** (ladder vs selected vehicle).
- Flow: pick departure/arrival → nodes → Δv ladder → plan→preview→commit burn (gated, auto-pause at node).

**S3 Research & Development** (GP-02)
- Subscreens: **Science portfolio** (domain UL bars + world tide + allocation sliders) · **Engineering programmes** (TRL ladders, campaigns, leads, overrun risk) · **Tech-tree graph** (sourced nodes, capability paths, gates) · **Domain detail** (UL curve, synergy, publish policy) · **Programme detail** (gate requirements, test campaign, dead-end risk).
- Inspector: selected domain or programme.

**S4 Vehicle Designer** (GP-03)
- Subscreens: **Component catalogue** (gated by researched maturity) · **Stage composer** (stages + redundancy blocks + mission reqs) · **Derived performance & trace** · **Realism flags** · **Compare** · **Production queue**.
- Flow: compose → live derived figures → flags → register production (gated).

**S5 Operations / Fleet** (GP-04 → GP-07)
- Subscreens: **Asset register** (virtualised fleet table) · **Craft inspector** (state vector, propellant, health, crew, comms lag) · **Standing orders** · **Comms / DSN passes** (ops-capacity) · **Missions** (the mission threads, GP-04).
- Crew rows gain dose/health/EDL (GP-07).

**S6 Economy & Contracts** (GP-01)
- Subscreens: **Budget & appropriations** (timeline, fiscal calendar, allocation) · **Resource ledger by location** (Δv-addressed) · **Launch market** (buy launch) · **Contracts / RFP board** (bid) · **Facilities** · **Partnerships/consortia**.
- Inspector: selected line / contract.

**S7 Bases & Construction** (GP-06)
- Subscreens: **Sites** (surveyed, gated by GP-05) · **Found base** · **Module catalogue & layout** · **Build queue** (delivery routed through logistics) · **Emergent properties** (power margin, ECLSS closure, sustainability, shielding) · **Self-sufficiency / embargo**.
- Inspector: selected base / module.

**S8 Personnel** (GP-02 → GP-07)
- Subscreens: **Roster** (scientists/engineers/PMs/astronauts, traits) · **Recruit** (hire/poach) · **Training** · **Astronaut careers** (radiation dose, health, eligibility — GP-07) · **Assignments**.

**S9 World / Politics** (GP-08)
- Subscreens: **Milestone race** (firsts ledger, world-first vs faction-first) · **Mood & approval** (→ budget/valuation) · **Policy & treaties** (levers, gating, drift, lobby) · **AI faction standings** · **Planetary protection** (categories, contamination grading).

**S10 Science Returns & Astrobiology** (GP-08)
- Subscreens: **Candidate worlds** (staged evidence meter, honesty-guarded) · **Candidate detail** (evidence stages, abiotic competitors, consensus, sample-return gate) · **Incoming data** (instruments → UL deltas) · **Discoveries log**.

**S11 Sojournal** (GP-09)
- Subscreens: **Browser / search** · **Entry** (source-cited) · **Cross-links** (every entity & number deep-links here — the "why this number" layer).

**S12 Alerts / Event Log** (GP-00 feed → GP-09 config)
- Subscreens: **Full feed** (filter by class) · **Interrupt review** · **Pause-policy config** (which classes pause) · **Watch conditions** (player-defined).

Plus a shell-level **Grand Goal & Score** surface (GP-08) reachable from the top bar / S9, and a **Continue / New game** entry (GP-09) that runs the ESA bootstrap.

---

## 4. Bespoke widget inventory

Reuse and extend the widgets the view-model already shapes (`widgets_data.rs`): **porkchop field**, **Δv ladder**, **TRL ladder**, **Understanding bars** (with world-tide ghost), **resource-by-location ledger**, **logistics-graph view**, **base schematic**, **astrobiology evidence meter**. The playability programme adds: **allocation sliders** (normalised splits that must sum to 1, with live remainder), **budget/appropriation timeline**, **component palette** (maturity-gated), **derived-performance trace panel**, **mission timeline** (the thread's stages), **dose/health gauges**, **milestone-race ladder**, **mood→budget curve**, **Grand-Goal / score card**, and the **plan→preview→commit gate panel** (the shared consequence-preview component every verb pops). Each widget is shaped in `sojourn-ui` and rendered in `sojourn-ui-desktop`; the renderer computes nothing.

---

## 5. Per-increment UI deliverable shape

Every increment's design doc specifies, for its screen(s): the **overview tier**, each **subscreen** (purpose, data shown, the view-model struct it reads, interactions), the **inspector** content, the **verbs** and their **plan→preview→commit** drafts (which Intent, which Preview deltas, which Gates), the **empty/edge states**, and the **hotkeys**. The renderer work is "thin": add the egui layout + wire to `sojourn-ui` view-model builders and `sojourn-game` intents. New view-model builders and widget shapes are unit-tested headlessly before any renderer work.
