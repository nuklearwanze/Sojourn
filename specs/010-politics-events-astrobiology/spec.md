# Feature Specification: Politics, Events, Milestones & Astrobiology (FA-09)

**Feature Branch**: `010-politics-events-astrobiology`
**Created**: 2026-06-14
**Status**: Draft
**Input**: User description: "Build Sojourn's politics, events, milestones and astrobiology systems — the non-combat competitive and narrative layer … faction relationships and prestige; public/political mood …; policy and treaties …; a COSPAR-style planetary-protection regime …; a seeded-plus-state-driven event system …; the astrobiology question honestly …; an AI world …; ~120 scored historic firsts …; selectable Grand Goals … Everything data-driven and seed-respecting. … Do not choose a tech stack."

## Overview

This slice is **the source of the game's tension in a game with no weapons**. With combat explicitly out of scope (Constitution IX), competitive pressure must come from three real forces, each modelled as honest mechanical pressure rather than set dressing:

1. **The race for historic firsts** — ~120 scored milestones where being *first in the world* beats being *first in your faction*, tracked globally so the player races an AI world.
2. **The politics of money and approval** — public and political mood that shifts with successes and failures (loss-of-crew most of all) and drives budgets, approvals, valuations, policy and treaties.
3. **The slow, honest unveiling of whether life exists elsewhere** — a per-game seeded astrobiology ground truth on candidate worlds, resolved only through a staged, probabilistic, mission-driven evidence process against abiotic competitors, with scientific consensus forming over time — never a binary "life found" popup.

Around these sit the **event system** (the seeded, state-driven scheduler that feeds the Slice 1 interrupt-and-pause loop), the **planetary-protection regime** (COSPAR-style categories, Special Regions, forward/back contamination), the **AI world** (non-player factions that research, build, fly, contract, partner and race), and the **Grand Goals** (Pathfinder, Homestead, Prospector, Seeker) that turn all of it into selectable win/scoring conditions.

Per the established slice architecture (the FA-04 C1 / FA-06 R1 / FA-08 decoupling), this layer is composed from upstream outputs but does **not** hard-depend on the upstream gameplay crates: missions/launches/landings (FA-04/07/08), research/tech maturity and the science tide (FA-05), budgets/valuations/markets/supply (FA-06), site planetary-protection categories and candidate-world astrobiology priors (FA-03), and loss-of-crew (FA-08) all flow in as **composed values / opaque inputs** the host assembles. This layer produces prestige, mood, approvals, policy state, contamination verdicts, events (interrupts), astrobiology consensus, milestone claims and the final score. There is **no combat and no aliens**: any discovered life is microbial/chemical and a **science object, never an actor**.

## Clarifications

### Session 2026-06-14

- Q: How do the per-faction astrobiology posteriors aggregate into the community consensus, and when is a candidate's question conclusively resolved? → A: **Weighted aggregate + confidence band** — consensus is a prestige/science-output-weighted aggregate of the per-faction posteriors; a candidate is *conclusive* when the consensus crosses a fixed confidence band (≥ 0.9 positive / ≤ 0.1 negative) **and** sample-return-tier evidence exists.
- Q: When two+ factions achieve the same unclaimed world-first on the same tick, how is the single winner chosen? → A: **Highest current prestige** wins; ties broken by lowest faction id (deterministic, seed-stable).
- Q: How is the loss of a body's pristine astrobiology value modelled on a Special-Region breach? → A: **Graded by overage** — pristine-value degradation is proportional to how far over the bioburden limit the lander is, scaled further by crash vs soft landing (small breach partially confounds future evidence; gross breach effectively ruins it).
- Q: How does the event system schedule events over time? → A: **Daily Bernoulli hazard** — per event source, a daily (cadence-tick) Bernoulli draw with a multiplicative-hazard probability, one seeded stream per named source (the proven FA-08 pattern).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The race for historic firsts (Priority: P1) 🎯 MVP

A player (or an AI faction) accomplishes something notable — the first fully-reusable orbital flight, the first kilogram of lunar-derived propellant sold, the first crewed Mars landing. The system recognises the achievement, decides whether it is a **world-first** (nobody, player or AI, has done it before) or merely a **faction-first** (a first for this faction but the world already has it), records it once, and awards prestige weighted by the milestone's historic significance. The player can see which firsts remain unclaimed and races the AI world for the world-first bonus.

**Why this priority**: This is the scoring spine and the competitive heart of a combat-free game. It is the single system that, on its own, makes the game a *race* rather than a sandbox — the natural pressure that replaces combat. It is the MVP.

**Independent Test**: Feed the layer a sequence of dated achievements (some by the player, some by AI factions); assert each milestone is awarded world-first to exactly the first global claimant and faction-first to later claimants of the same milestone within their own faction, that prestige accrues with the milestone's weight, that an already-world-claimed milestone cannot be world-claimed again, and that the whole ledger is bit-identical across two runs of the same seed + decision script.

**Acceptance Scenarios**:

1. **Given** the "first crewed Mars landing" milestone is unclaimed, **When** the player's mission achieves it, **Then** it is recorded as a world-first to the player with full prestige, and the milestone is globally retired for world-first.
2. **Given** an AI faction already holds the world-first for "first lunar-polar ice ground-truth", **When** the player later achieves the same thing, **Then** the player receives the (smaller) faction-first credit and no world-first.
3. **Given** a milestone with prerequisite conditions (e.g. "first kg of lunar-derived propellant *sold*" requires a recorded sale), **When** the conditions are not all met, **Then** the milestone is not awarded.
4. **Given** two factions claim the same world-first on the same simulated day, **When** the tie is resolved, **Then** it is resolved deterministically (documented, seed-stable rule) and recorded once.

---

### User Story 2 - The politics of money and approval (Priority: P1)

The player runs an agency or company whose fortunes rise and fall with public and political mood. A spectacular first lifts mood and loosens budgets; a loss-of-crew is severe and long-lasting, freezing approvals and shrinking appropriations or valuation for years. Mood feeds concrete levers: appropriation modifiers (agencies), private valuation and contract access (companies), and approval timelines for major programs.

**Why this priority**: This is the "politics of money and approval" — the second of the three tension sources. It makes every success and failure echo beyond the mission, and it is what makes loss-of-crew (FA-08) *matter* at the strategic level. Without it, prestige is a number with no teeth.

**Independent Test**: Drive the mood model with a scripted sequence of outcomes (firsts, routine successes, a launch failure, a loss-of-crew, an economic downturn) and assert mood moves in the correct direction with the correct magnitude and persistence, that loss-of-crew produces a deeper and longer depression than a routine failure, and that the resulting budget/valuation/approval modifiers are the documented function of mood — all deterministic and reconcilable to their inputs.

**Acceptance Scenarios**:

1. **Given** a faction at baseline mood, **When** it achieves a world-first, **Then** its public/political mood rises and its appropriation/valuation modifier improves for a sustained, decaying period.
2. **Given** a crewed program, **When** a loss-of-crew occurs, **Then** mood drops sharply, crewed-flight approval is frozen for a multi-year recovery window, and the budget/valuation modifier is materially reduced — a deeper, longer effect than a robotic launch failure.
3. **Given** a major program awaiting approval, **When** mood is low, **Then** the approval is delayed or denied per the documented mood→approval mapping; when mood is high, approval is faster.
4. **Given** an economic cycle input (boom/recession), **When** it shifts, **Then** appropriations/valuations move accordingly without violating conservation of the underlying economy inputs.

---

### User Story 3 - The astrobiology question, answered honestly (Priority: P1)

The player wonders whether life exists on a candidate world (Mars subsurface, Europa/Enceladus oceans, Titan, Ceres brines, Venus clouds). The truth is fixed per game by the seed within scientifically plausible bounds, but is **never directly observable**. The player learns about it only by flying missions that return staged evidence — orbital biosignature hints, then in-situ chemistry, then microscopy/metabolism, then sample return and independent confirmation — each stage giving probabilistic evidence that competes with abiotic explanations. A scientific consensus forms over time and missions; a conclusive result (positive *or* conclusive negative) is a top-tier milestone and reshapes politics and planetary protection.

**Why this priority**: This is the third tension source and the most "research-process" system in the game — the honest "are we alone here?" question that gives the late game its meaning and defines the *Seeker* Grand Goal. Constitution VIII (educational honesty) makes its honest treatment non-negotiable: no binary popups, no misinformation.

**Independent Test**: Seed a game; assert the per-candidate ground truth is drawn from the FA-03 sourced priors and is statistically faithful across many seeds; assert the ground truth is not exposed by any query before evidence; feed a staged sequence of mission evidence and assert **each faction's posterior** belief (and the community consensus that aggregates them) moves toward the truth probabilistically (with possible false positives/negatives at early stages and factions able to disagree publicly), that abiotic competitors can explain away early hints, that no single stage yields certainty, and that "conclusive" status requires sample-return-tier evidence and the **community consensus** crossing the documented confidence band (≥ 0.9 / ≤ 0.1) — all deterministic per seed.

**Acceptance Scenarios**:

1. **Given** a new game, **When** ground truth is drawn, **Then** each candidate world's life-present flag is seeded from its sourced prior, most games are mostly negative, and the truth is queryable only as "unknown" until evidence accrues.
2. **Given** a candidate with positive ground truth, **When** only an orbital biosignature hint has been collected, **Then** each faction's posterior rises but the community consensus remains inconclusive, an abiotic competitor hypothesis remains viable, and factions weighting the evidence differently may **publicly disagree**.
3. **Given** accumulating in-situ and sample-return evidence, **When** the community consensus (the prestige-weighted aggregate of per-faction posteriors) crosses the documented confidence band (≥ 0.9 / ≤ 0.1), **Then** the question is marked conclusively resolved (positive or negative), a top-tier milestone fires, and planetary-protection stringency for that body tightens.
4. **Given** a candidate with negative ground truth, **When** evidence accrues, **Then** early stages may show a false-positive hint but continued evidence converges on a conclusive negative, never a false "life confirmed".

---

### User Story 4 - The event system (seeded + state-driven) (Priority: P2)

While time advances under warp, the world interrupts the player when something matters: a launch failure, a stuck valve or comms loss, a solar storm, a funding crisis or windfall, a political shake-up, a rival's milestone, a discovery, a Pu-238 supply shock, a key hire or loss. Events are not pure RNG: a low-TRL, under-tested, over-subscribed-ops craft *earns* a higher anomaly probability from its state, and the seeded scheduler makes the same seed + decisions reproduce the same drama. Each event feeds the Slice 1 interrupt-and-pause loop at its configured class.

**Why this priority**: Events are the delivery mechanism that turns the other systems into felt, moment-to-moment pressure and connects everything to the FA-01 core loop. It is P2 because the scoring/politics/astrobiology systems define *what* matters; the event system decides *when the player is told*.

**Independent Test**: Run the scheduler over a fixed horizon for two factions with different state (one mature/well-tested, one low-TRL/over-subscribed) and assert the riskier state yields a strictly higher realised anomaly rate, that every event is emitted at the correct interrupt class and is acknowledgeable, that the event stream is bit-identical across two runs of the same seed + decisions and across stepping patterns, and that no event produces a physics-violating outcome.

**Acceptance Scenarios**:

1. **Given** two craft identical except maturity/test/ops-load, **When** the anomaly hazard is evaluated, **Then** the riskier craft has the higher event probability (state-driven, multiplicative hazard), seeded and reproducible.
2. **Given** a configured interrupt class (e.g. anomaly, discovery, budget vote), **When** an event of that class fires, **Then** it pauses/interrupts per the FA-01 loop and is acknowledgeable; log-only classes do not pause.
3. **Given** the same seed and decision script, **When** the simulation is run twice with different stepping patterns, **Then** the full event log is identical.
4. **Given** a rival AI faction hits a milestone, **When** it occurs, **Then** a "rival milestone" event notifies the player.

---

### User Story 5 - Policy & treaties with real consequences (Priority: P2)

The world has rules: launch licensing and range access, nuclear-launch approval (for NTP/RTG), planetary-protection stringency, export controls (ITAR-like effects on partnerships and component sourcing), and debris/sustainability regulation. These are real levers with consequences — a mission needing an un-granted nuclear-launch approval is gated or penalised; export controls raise a faction's avionics costs or block a partnership. World policy can drift over time, and some factions can lobby to tighten or relax it.

**Why this priority**: Policy is where politics becomes mechanical friction on the player's plans, and it is the seam through which planetary protection (US6) and the AI world (US7) exert pressure. P2 because it modifies and gates the core gameplay rather than constituting it.

**Independent Test**: Define a policy state; submit missions/partnerships that do and do not satisfy each lever; assert the satisfying ones proceed and the non-satisfying ones are gated or penalised per the documented rule; lobby a lever and assert its level changes deterministically; advance time and assert policy drift follows the seeded/scheduled model.

**Acceptance Scenarios**:

1. **Given** nuclear-launch approval is not held, **When** a mission requiring it is attempted, **Then** it is gated or incurs the documented penalty until approval is obtained.
2. **Given** an export-control regime, **When** a partnership or component-sourcing depends on a restricted transfer, **Then** the restriction raises cost or blocks the arrangement per policy.
3. **Given** a faction lobbies to relax planetary-protection stringency, **When** the lobby resolves, **Then** the stringency level changes by the documented amount and the change persists/drifts deterministically.
4. **Given** launch licensing/range access constraints, **When** a launch is manifested, **Then** licensing status affects its availability or cost.

---

### User Story 6 - Planetary protection: forward & back contamination (Priority: P2)

The Solar System's bodies carry COSPAR-style protection categories (I–V); Special Regions on Mars (possible liquid water) and ocean worlds (Europa/Enceladus) carry strict forward-contamination rules — bioburden limits, sterilisation cost and mass, restricted access. Crashing a non-sterile lander into a Special Region is a real, tempting, consequential mistake: a science and reputation penalty that can **ruin the body's pristine astrobiology value for everyone, including future you**. Sample return from a potentially-habitable world requires a back-contamination containment chain (receiving facilities, restricted-Earth-return).

**Why this priority**: Planetary protection is the constraint that makes astrobiology (US3) and policy (US5) bite: it is how cutting corners poisons your own evidence and your reputation. P2 because it is a consequence layer over missions and the astrobiology question.

**Independent Test**: Assign categories/Special-Region flags to bodies and sites (composed from FA-03); send a sterile and a non-sterile lander to a Special Region and assert only the non-sterile one triggers forward contamination with the documented science/reputation penalty and a degradation of that body's pristine astrobiology value; attempt a sample return with and without a compliant containment chain and assert the non-compliant one is gated or penalised for back-contamination risk.

**Acceptance Scenarios**:

1. **Given** a Special Region with a bioburden limit, **When** a lander exceeding the limit reaches it (especially via a crash), **Then** forward contamination is recorded with a science penalty, a reputation/political cost, and a loss of that body's pristine astrobiology value.
2. **Given** a compliant, sterilised lander meeting the bioburden limit, **When** it operates in the Special Region, **Then** no forward-contamination penalty is incurred (sterilisation cost/mass having been paid).
3. **Given** a sample return from a potentially-habitable world, **When** the back-contamination containment chain is not in place, **Then** the return is gated or penalised; when it is in place, it proceeds.
4. **Given** a body whose pristine value has been ruined, **When** astrobiology evidence is later sought there, **Then** the contamination is reflected as degraded/confounded evidence value.

---

### User Story 7 - The AI world (competitors & partners) (Priority: P2)

Non-player factions (always-AI CNSA plus the other unplayed organisations, and minor agencies) run simplified versions of the same systems: they research (advancing the global science tide), build, fly, bid on contracts, partner, hit milestones and suffer accidents. They provide the milestone race, a contract/partnership market, licensing/buy options and narrative texture. They obey the same physics and plausibility rules — no cheating into impossible tech — and difficulty tunes their funding and competence, never their physics.

**Why this priority**: The AI world is the *opponent* in the race (US1) and a partner/market for the economy. P2 because the firsts/politics systems define the contest; the AI world populates it.

**Independent Test**: Run the AI world over a horizon and assert AI factions claim firsts, advance the science tide and generate contracts/partnerships; assert no AI faction acquires a capability beyond what the plausibility rules permit; assert that raising difficulty increases AI funding/competence but never grants physics-violating performance; assert AI behaviour is deterministic per seed.

**Acceptance Scenarios**:

1. **Given** an AI faction with a capability profile, **When** the horizon advances, **Then** it pursues and can claim milestones, contributing to the world-first race against the player.
2. **Given** the global science tide, **When** AI factions research, **Then** the tide advances per the documented model (AI progress is visible as rising baseline capability).
3. **Given** a difficulty setting, **When** it is raised, **Then** AI funding/competence increases but no AI faction gains physics-violating tech.
4. **Given** the same seed and decisions, **When** the AI world is run twice, **Then** AI actions and outcomes are identical.

---

### User Story 8 - Grand Goals as win/scoring conditions (Priority: P3)

At the start the player picks a Grand Goal — **Pathfinder** (exploration firsts & science), **Homestead** (a settlement that survives a 5-year resupply embargo), **Prospector** (≥ X t/yr off-Earth resources sold at profit), or **Seeker** (resolve the astrobiology question for ≥ 3 candidate worlds) — and can change it later with a penalty. At the configured horizon (25/50/100-yr runs) the run resolves to a **pass/fail verdict on the selected Grand Goal** (its primary condition met or not) **plus a secondary composite score** (prestige + milestones + Grand-Goal progress) that ranks the run and breaks ties. Soft-fail states (agency gutting, bankruptcy, loss-of-crew spiral) are handled as continue-in-observer/rebuild rather than hard game-over.

**Why this priority**: Grand Goals are the meta-layer that gives the run a shape and a verdict; they compose the outputs of every other story. P3 because they are the capstone — meaningful only once the systems they score exist.

**Independent Test**: Select each Grand Goal in turn, drive the relevant systems to/through its threshold, and assert the goal's progress and completion are computed deterministically from the documented criteria (exploration firsts for Pathfinder, embargo-survival index for Homestead via composed FA-07/08 inputs, profitable off-Earth tonnage for Prospector via composed FA-06 inputs, conclusive astrobiology resolutions for Seeker via US3); assert changing goal applies the documented penalty; assert the final score at the horizon is a deterministic function of its inputs.

**Acceptance Scenarios**:

1. **Given** the Seeker goal, **When** three candidate worlds reach conclusive astrobiology resolution, **Then** the goal's primary condition is met and scored.
2. **Given** the Prospector goal, **When** profitable off-Earth resource throughput crosses the threshold, **Then** the goal condition is met (composed from the FA-06 economy).
3. **Given** a goal change mid-game, **When** it is applied, **Then** the documented penalty is taken and the new goal's scoring takes effect.
4. **Given** the configured horizon is reached, **When** the game scores, **Then** a deterministic **pass/fail verdict on the selected Grand Goal** plus a **secondary composite score** (prestige + milestones + Grand-Goal progress) is produced, and soft-fail states are reflected rather than ending the run.

---

### Edge Cases

- **Simultaneous world-first claims**: two factions achieve the same unclaimed world-first on the same simulated tick — resolved by a documented, seed-stable deterministic tiebreak; recorded exactly once as world-first.
- **Retroactive prerequisite invalidation**: a milestone whose prerequisite later becomes false (e.g. the "sale" is reversed) — firsts are permanent once validly awarded (assumption FR-PEA-107).
- **Astrobiology false positive at sample-return tier**: the strongest evidence stage must still be reconcilable; the model must never let a conclusive-positive be declared for a negative-ground-truth world (conclusive-positive requires positive ground truth; false *hints* are allowed only at pre-conclusive stages).
- **Contaminating a world before any evidence**: forward contamination of a pristine candidate before its question is resolved permanently degrades the achievable evidence quality there (you can foreclose your own *Seeker* goal).
- **Loss-of-crew with no active crewed program**: a loss-of-crew input arriving for a faction whose program state is inconsistent is handled without panic; the mood effect applies to the responsible faction.
- **AI faction insolvency / agency gutting**: an AI faction hitting a soft-fail state continues in a degraded mode rather than vanishing, so the milestone ledger and contracts remain consistent.
- **Policy lever drift past bounds**: lobbying or drift cannot push a policy level outside its defined range.
- **Mood saturation**: repeated successes/failures cannot push mood outside its bounded range; effects saturate, not overflow.
- **Grand-Goal completion after horizon**: progress achieved after the scoring horizon does not retroactively change the recorded score.

## Requirements *(mandatory)*

### Functional Requirements

#### Historic firsts & prestige (US1) — FR-PEA-1xx

- **FR-PEA-101**: The system MUST maintain a data-driven catalogue of ~120 historic firsts, each with an id, an era/category, a human-readable description, a **score/prestige weight**, machine-checkable **award conditions** (the achievement that claims it), and a `source` justifying it as a real historic-significance milestone.
- **FR-PEA-102**: For each milestone the system MUST track a **world-first** claim (the first global claimant, player or AI) and **faction-first** claims (the first time each faction achieves it), awarding the full weight for world-first and a documented lesser weight for faction-first.
- **FR-PEA-103**: A milestone MUST be awarded only when **all** its award conditions are satisfied by composed mission/economy/world inputs; unmet conditions MUST NOT award it.
- **FR-PEA-104**: A world-first MUST be retired globally on first award and MUST NOT be re-awarded as world-first; later achievers receive faction-first credit only.
- **FR-PEA-105**: Simultaneous same-tick world-first claims MUST be resolved by awarding the world-first to the claimant with the **highest current prestige**, breaking remaining ties by **lowest faction id** — deterministic, seed-stable, recorded exactly once.
- **FR-PEA-106**: Prestige MUST accrue per faction from awarded firsts (and other documented sources such as science output and reliability) and MUST be queryable; prestige MUST be traceable to the firsts/events that produced it.
- **FR-PEA-107**: Once validly awarded, a first MUST be permanent (not revoked by later state changes) unless explicitly defined otherwise.

#### Politics, public mood, budgets & approvals (US2) — FR-PEA-2xx

- **FR-PEA-201**: The system MUST maintain a bounded **public/political mood** state per faction that responds to composed outcome inputs (firsts, routine successes, launch/anomaly failures, loss-of-crew, economic cycle) with documented direction, magnitude and decay.
- **FR-PEA-202**: Loss-of-crew MUST produce a **severe and long-lasting** mood depression — deeper and longer than a routine robotic failure — and MUST freeze crewed-flight approval for a documented multi-year recovery window.
- **FR-PEA-203**: Mood MUST drive concrete, documented levers: appropriation modifiers (agencies), private valuation/contract access (companies), and approval timelines/denials for major programs.
- **FR-PEA-204**: All mood effects MUST be data-parameterised (no magic numbers in code) and MUST saturate within bounds rather than overflow.
- **FR-PEA-205**: Approval requests for major programs MUST resolve (grant / delay / deny) as the documented function of current mood and policy, deterministically.
- **FR-PEA-206**: Budget/valuation modifiers produced by this layer MUST be expressed as factors over the composed FA-06 economy inputs and MUST NOT create or destroy value outside that economy's conservation rules.

#### Astrobiology — seeded ground truth & staged evidence (US3) — FR-PEA-3xx

- **FR-PEA-301**: At game start the system MUST draw a **per-candidate seeded ground-truth** life-present flag for each candidate world, using the FA-03 sourced `presence_prob` priors, on the per-game seed; across many seeds the realised positive fraction MUST match the priors within tolerance.
- **FR-PEA-302**: Ground truth MUST NOT be directly observable through any query; it MUST be discoverable only through accrued evidence (the per-faction posteriors and the community consensus aggregating them), all of which begin at "unknown".
- **FR-PEA-303**: Evidence MUST be **staged** (orbital biosignature hint → in-situ chemistry → microscopy/metabolism → sample return & independent confirmation), each stage contributing probabilistic evidence, with **abiotic competitor hypotheses** that can explain away pre-conclusive evidence.
- **FR-PEA-304**: No single stage MUST yield certainty; a **conclusive** resolution (positive or negative) MUST require sample-return-tier evidence **and** the **community consensus** crossing a fixed confidence band — **≥ 0.9 for conclusive-positive, ≤ 0.1 for conclusive-negative** (band values data-driven). Neither condition alone suffices.
- **FR-PEA-305**: The model MUST permit false positives and false negatives at **pre-conclusive** stages but MUST NEVER declare a conclusive-positive for a negative-ground-truth world (a conclusive-positive implies positive ground truth).
- **FR-PEA-306**: A conclusive resolution MUST emit a top-tier milestone (US1) and tighten planetary-protection stringency (US6) for the affected body, and MUST be reflected in mood/politics (US2).
- **FR-PEA-308**: Each faction MUST hold its **own posterior** over each candidate (updated from the evidence it has access to, with its own weighting), and the **community consensus** MUST be a **prestige/science-output-weighted aggregate** of those per-faction posteriors (more reputable factions sway the consensus more; weights data-driven). Factions MAY publicly disagree before consensus is conclusive, and the disagreement state MUST be queryable. Conclusive resolution (FR-PEA-304/306) is defined on the community consensus, not on any single faction's belief.
- **FR-PEA-307**: Forward contamination (US6) of a candidate MUST degrade the achievable evidence quality/value there (confounded evidence), and the layer MUST expose this so the *Seeker* goal can account for it.

#### Event system — seeded + state-driven (US4) — FR-PEA-4xx

- **FR-PEA-401**: The system MUST schedule events from a data-driven catalogue of event classes (launch success/failure, anomaly, solar/radiation storm, funding crisis/boom, political shake-up, rival milestone, discovery, supply shock, personnel event), each with a `source` and an interrupt/log classification feeding the FA-01 loop.
- **FR-PEA-402**: Event probabilities MUST be **state-driven** — derived from composed state (maturity/TRL, test heritage, ops oversubscription, environment) via a multiplicative-hazard model so a riskier configuration earns a strictly higher probability — and MUST be evaluated as a **daily (cadence-tick) Bernoulli draw per named event source on its own seeded stream** (the FA-08 pattern), so the same seed + decisions reproduce the same events.
- **FR-PEA-403**: Each fired event MUST be emitted at its configured interrupt class and MUST be acknowledgeable; log-only classes MUST NOT pause the loop.
- **FR-PEA-404**: The complete event stream MUST be bit-identical across two runs of the same seed + decision script and across different stepping patterns.
- **FR-PEA-405**: No event MUST produce a physics-violating or plausibility-violating outcome (Constitution I/II); events modify state within sourced bounds only.

#### Policy & treaties (US5) — FR-PEA-5xx

- **FR-PEA-501**: The system MUST maintain a data-driven set of policy/treaty levers (launch licensing & range access, nuclear-launch approval, planetary-protection stringency, export controls, debris/sustainability), each with a bounded level and a `source`.
- **FR-PEA-502**: A mission/partnership that fails to satisfy a required lever (e.g. missing nuclear-launch approval, a restricted export transfer) MUST be **gated or penalised** per the documented rule; satisfying ones MUST proceed.
- **FR-PEA-503**: Policy levels MUST be able to **drift** over time (seeded/scheduled) and MUST be **lobbyable** by factions, with deterministic, bounded outcomes.
- **FR-PEA-504**: Planetary-protection stringency set here MUST be the value consumed by the planetary-protection regime (US6), keeping a single source of truth.
- **FR-PEA-505**: Export controls MUST modify partnership feasibility and component-sourcing cost as a documented factor over composed FA-06 inputs.

#### Planetary protection (US6) — FR-PEA-6xx

- **FR-PEA-601**: The system MUST associate each body/site with a COSPAR-style protection **category (I–V)** and a **Special Region** flag with a **bioburden limit**, composed from FA-03 site data and the active stringency (US5), each carrying a `source`.
- **FR-PEA-602**: A lander exceeding the bioburden limit reaching a Special Region (especially via a crash) MUST record **forward contamination** with the documented science penalty, reputation/political cost (US2), and a loss of that body's **pristine astrobiology value** (US3) that is **graded by the bioburden overage** (how far over the limit) and **scaled by crash vs soft landing** — a small breach partially confounds future evidence; a gross breach effectively ruins it (degradation function data-driven).
- **FR-PEA-603**: A compliant sterilised lander meeting the limit MUST incur **no** forward-contamination penalty (the sterilisation cost/mass having been paid as a composed input).
- **FR-PEA-604**: A sample return from a potentially-habitable world MUST require a **back-contamination containment chain**; without it the return MUST be gated or penalised.
- **FR-PEA-605**: Forward/back contamination outcomes MUST be deterministic given the composed mission facts and the active categories.

#### The AI world (US7) — FR-PEA-7xx

- **FR-PEA-701**: Non-player factions MUST pursue milestones, advance the global science tide, and generate contracts/partnership offers, contributing to the world-first race and the economy/market, using an **abstracted, heuristic, seeded behaviour model** over composed capability estimates — **not** a full mirror of the player's FA-04…08 systems.
- **FR-PEA-702**: AI factions MUST obey the same plausibility rules as the player — they MUST NOT acquire physics- or plausibility-violating capability (no AI cheating into impossible tech).
- **FR-PEA-703**: Difficulty MUST tune AI **funding and competence only**, never their physics or plausibility envelope.
- **FR-PEA-704**: AI faction behaviour and outcomes MUST be deterministic for a given seed + decision context.

#### Grand Goals & scoring (US8) — FR-PEA-8xx

- **FR-PEA-801**: The system MUST offer the four Grand Goals (Pathfinder, Homestead, Prospector, Seeker), selectable at start and changeable mid-game with a documented penalty.
- **FR-PEA-802**: Each Grand Goal's progress and completion MUST be computed deterministically from documented criteria over composed inputs: exploration firsts/science (Pathfinder), embargo-survival index (Homestead, composed FA-07/08), profitable off-Earth tonnage (Prospector, composed FA-06), conclusive astrobiology resolutions for ≥ 3 worlds (Seeker, US3).
- **FR-PEA-803**: At the configured horizon (25/50/100-yr) the game MUST resolve to a **pass/fail verdict on the selected Grand Goal** (its primary condition met or not) as the primary result, **plus a secondary composite score** (prestige + milestones + Grand-Goal progress) used to rank the run and break ties — both deterministic functions of their inputs.
- **FR-PEA-804**: Soft-fail states (agency gutting, private bankruptcy, loss-of-crew spiral) MUST be reflected in scoring and MUST allow the run to continue in observer/rebuild mode rather than hard-ending.

#### Cross-cutting: determinism, data, decoupling, encyclopedia — FR-PEA-9xx

- **FR-PEA-901**: All stochastic outcomes in this layer (astrobiology ground truth, event scheduling, AI decisions, policy drift, lobbying, contamination rolls) MUST derive from the per-game seed via explicit named streams; the layer MUST contain no wall-clock or unseeded randomness (Constitution III).
- **FR-PEA-902**: This layer MUST depend only on the simulation core; all upstream gameplay facts (missions/launches/landings, research/tech maturity & science tide, budgets/valuations/markets/supply, site PP categories, candidate-world priors, loss-of-crew) MUST enter as **composed values / opaque inputs** the host assembles (the FA-04/06/08 decoupling), adding no new cross-gameplay-crate dependency.
- **FR-PEA-903**: Every quantitative data entry (milestone weights, mood coefficients, event base rates, policy bounds, PP categories/limits, astrobiology priors usage, Grand-Goal thresholds, AI tuning) MUST carry a `source` and MUST be schema- and source-validated in CI (Constitution I/V).
- **FR-PEA-904**: The layer MUST be fully testable headless with no UI dependency, and its state MUST round-trip through save/load bit-identically (Constitution IV; versioned, hash-pinned data).
- **FR-PEA-905**: There MUST be **no combat, weapons, sabotage, or alien actor** anywhere in this layer; discovered life MUST be represented purely as a science object, never as an agent or antagonist (Constitution IX).
- **FR-PEA-906**: Discovered results, bodies, policies and milestones SHOULD expose encyclopedia ("Sojournal") content with sources that updates as belief-state/world advance (Constitution VIII), consistent with the existing FA-03 Sojournal surface.

### Key Entities

- **Faction (political view)**: an organisation's relationship state (cooperation↔rivalry with each other faction), partnership/consortium membership, funding-model class (appropriation vs revenue), prestige score, and current public/political mood.
- **Milestone / First**: a catalogued historic achievement — id, era/category, description, score weight, award conditions, world-first claimant + tick, per-faction faction-first claimants, `source`.
- **Prestige record**: per-faction accumulated prestige with provenance (which firsts/events contributed).
- **Mood state**: per-faction bounded political/public mood with decay, plus the derived budget/valuation/approval modifiers; a global political-weather/economic-cycle context.
- **Policy/Treaty lever**: a named world rule (licensing, nuclear-launch, PP stringency, export control, debris) with a bounded level, drift behaviour, lobby state, and `source`.
- **Planetary-protection category**: per body/site COSPAR category I–V, Special-Region flag, bioburden limit, and current pristine-value state, with `source`.
- **Contamination record**: a forward- or back-contamination event with body/site, cause (e.g. crash), severity, and the science/reputation/astrobiology-value consequences.
- **Event**: a catalogued class with seeded + state-driven trigger, interrupt/log classification, and payload feeding the FA-01 loop.
- **Astrobiology candidate**: a candidate world with composed FA-03 prior, the seeded hidden ground-truth flag, tier (subsurface/ocean/atmospheric/brine), the accrued staged evidence, abiotic-competitor hypotheses, the **per-faction posteriors** and the **community-consensus aggregate** (plus a public-disagreement state), and conclusive-resolution status.
- **AI faction behaviour state**: an AI organisation's capability profile, current goals/targets, funding/competence (difficulty-tuned), and pending actions — an **abstracted, heuristic, seeded** model, all seeded.
- **Grand Goal**: the selected goal type, its progress against documented criteria, change history/penalty, its **pass/fail verdict**, and its contribution to the secondary composite score.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For any seed + decision script, the milestone ledger (world-first/faction-first claimants and prestige) is **bit-identical across two runs** and across stepping patterns (determinism gate).
- **SC-002**: A given world-first is awarded to **exactly one** global claimant (the first), and every later achiever of that milestone receives faction-first credit only — verified across multi-faction races including same-tick ties.
- **SC-003**: A loss-of-crew event produces a mood depression and budget/valuation reduction that is **measurably deeper and longer-lasting** (documented multiplier and recovery window) than a routine robotic launch failure, and freezes crewed-flight approval for the defined multi-year window.
- **SC-004**: Across ≥ 1,000 seeds, the fraction of candidate worlds drawn with positive astrobiology ground truth matches the sourced priors **within a defined tolerance**, and no query exposes any candidate's ground truth before evidence accrues.
- **SC-005**: No astrobiology candidate is ever marked **conclusively positive** unless its seeded ground truth is positive; conclusive resolution requires sample-return-tier evidence **and** the prestige-weighted community consensus crossing the confidence band (≥ 0.9 / ≤ 0.1) — no single-stage binary result.
- **SC-006**: For two craft differing only in maturity/test-heritage/ops-load, the riskier craft has a **strictly higher** realised anomaly/event rate over a fixed horizon, and the entire event stream is reproducible from seed + decisions.
- **SC-007**: A mission lacking a required policy approval (e.g. nuclear-launch) is **gated or penalised** per the active policy in 100% of cases; satisfying missions proceed; tightening PP stringency raises the sterilisation cost/mass requirement that compliant missions must meet.
- **SC-008**: Crashing a non-sterile (over-limit) lander into a Special Region triggers a forward-contamination record with the documented science/reputation penalty and degrades that body's pristine astrobiology value in 100% of cases; a compliant lander triggers none.
- **SC-009**: Over a full-horizon run, AI factions claim firsts and advance the science tide, and **no AI faction ever acquires a capability outside the plausibility envelope**; raising difficulty increases AI funding/competence outputs without changing their physics.
- **SC-010**: Each of the four Grand Goals has a deterministic, documented **pass/fail verdict** computed from composed inputs, and the secondary composite score at the configured horizon is reproducible from seed + decisions.
- **SC-011**: All FA-09 data files pass schema + `source`-presence validation in CI, the layer's state round-trips through save/load bit-identically, and the FA-01…08 suites remain green with no new cross-gameplay-crate dependency.
- **SC-012**: The layer sustains the full world (all ten factions + AI world, ~120 firsts, the event scheduler, the policy/PP/astrobiology state) at high time-warp within the core's tick-time budget.

## Assumptions

- **Composed-value decoupling (architecture)**: consistent with FA-04 (C1) / FA-06 (R1) / FA-08, this layer depends only on the simulation core; upstream gameplay facts arrive as composed values the host assembles. This is assumed, not re-litigated.
- **Firsts are permanent**: a validly awarded world-/faction-first is not revoked by later state changes (FR-PEA-107).
- **Milestone catalogue scope**: FR-PEA-101's "~120 firsts" is the catalogue *target*; this slice authors a **representative sourced subset spanning all eras** with the full structure (id/era/weight/conditions/source), completable in data without code change (the FA-05 tech-node-subset precedent). The success criteria do not require the full 120 at this slice.
- **Single source of truth for PP stringency**: the policy lever (US5) sets the stringency the PP regime (US6) consumes (FR-PEA-504).
- **Astrobiology priors are owned by FA-03**: the `presence_prob` priors live in `data/world/astrobiology.ron`; FA-09 *draws* the seeded ground truth from them and runs the evidence process, rather than redefining the priors.
- **AI fidelity (confirmed)**: AI factions use an abstracted, heuristic, seeded behaviour model over composed capability estimates — not a full mirror of the player's FA-04…08 systems (FR-PEA-701).
- **Astrobiology consensus is per-faction (confirmed)**: each faction holds its own posterior; the community consensus is their documented aggregate, factions may publicly disagree, and conclusive resolution is defined on the consensus (FR-PEA-308).
- **Scoring shape (confirmed)**: the run resolves to a pass/fail verdict on the selected Grand Goal plus a secondary composite score for ranking/tie-break (FR-PEA-803).
- **Mood/budget coupling is a modifier, not a generator**: this layer scales the FA-06 economy via documented factors and never creates/destroys value outside the economy's conservation rules (FR-PEA-206).
- **Event hazard model (confirmed)**: events use a daily (cadence-tick) Bernoulli draw per named source on its own seeded stream, with a multiplicative-hazard probability modulated by composed state — the proven FA-08 pattern (FR-PEA-402).
- **Soft-fail, not game-over**: soft-fail states continue the run in observer/rebuild mode (per OVERVIEW §9); hard game-over is out of scope.
- **Difficulty tunes harshness, not physics**: difficulty affects political/economic harshness, event base rates and AI funding/competence — never physics or plausibility (OVERVIEW §10).
- **Sources**: all quantitative entries cite real COSPAR policy, historic-firsts significance, published astrobiology priors, and documented political/funding dynamics; speculative endgame firsts are gated behind the FA-05 Breakthrough system.

## Dependencies

- **FA-01 (core)**: the deterministic kernel, seeded streams, the interrupt-and-pause loop and event/command plumbing this layer feeds.
- **FA-03 (world)**: candidate-world astrobiology priors, site planetary-protection categories/Special-Region data, body facts, and the Sojournal surface — composed in.
- **FA-05 (research)**: tech/maturity and the global science tide that drive AI progress, event hazards and breakthrough-gated firsts — composed in.
- **FA-06 (economy)**: budgets, valuations, contract/partnership market, and strategic supply (e.g. Pu-238) that mood/policy modify and Prospector scores — composed in.
- **FA-07 (bases) / FA-08 (crew)**: mission/landing/embargo facts and loss-of-crew events that claim firsts, drive mood, trigger contamination, and score Homestead — composed in.
