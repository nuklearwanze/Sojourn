# Contract: Observation & Prospecting Commands + Events (FR-WORLD-304/502/703)

World mutations are **journaled commands** routed through FA-02's `Command::ModulePayload {module:
"world", kind, payload}` → `SimModule::on_command` — **no kernel change**. Payloads are postcard
DTOs. All effects are deterministic and seeded; replay reproduces them bit-identically.

## Commands (`WorldCommand`)

### `Observe { faction, target, scope, class, quality }`
- `target`: `BodyId` or `SiteRef`; `scope`: a property key or `All`; `class`: remote-sensing |
  in-situ | sample-grade; `quality`: instrument quality level.
- **Validation (trust-the-caller, FR-WORLD-304)**: structural only — target exists, class/quality
  valid, property sensible for the target. Entitlement (being there with an instrument) is enforced
  by the mission slices that will *issue* these commands. Invalid commands are rejected
  deterministically (no state change, no panic).
- **Effect**: for each sensed property, draw `ε ~ N(0, σ²(class,quality))` from the `obs-noise`
  stream keyed by `(faction, target, property, observation-seq)`; apply the Gaussian precision-add
  update (see `belief-model.md`); clamp variance to the class floor; append to the faction's belief
  change log; emit `survey-milestone` when a documented certainty threshold is crossed.
- **Honesty**: moves the *acting faction's* belief only; never writes truth; never widens variance
  (information never decreases); cannot reach below the class floor.

### `Prospect { faction, field, effort }`
- **Effect**: draw a detection count from the field's `detection_model(effort)` and sample each new
  body from the field distributions via the `prospect` stream; allocate a collision-free
  `id ≥ 2³¹` from the persisted monotonic counter; record the **Generated Body** (a per-world fact);
  narrow the discoverer's belief about it; republish the `generated-bodies` view to astro; emit
  `body-catalogued` per new body.
- **Determinism (SC-004)**: identical seed + command script ⇒ identical ids, orbits and properties;
  aggregate over many seeds matches the field's sourced distributions within documented tolerance.

## Events (data-registry additions to `data/kernel/event-classes.ron`)

| Class | Policy | Emitted when |
|---|---|---|
| `body-catalogued` | `LogOnly` | a prospecting draw creates a new small body |
| `survey-milestone` | `LogOnly` | a faction's belief about a property crosses a documented certainty threshold |

Both are non-interrupting (discovery is significant but not pause-worthy); FA-05 (research) and
FA-10 (UI) consume them later. Adding them is a data change — no kernel code.

## Streams (declared in the world manifest)

| Stream | Used by | Keying |
|---|---|---|
| `world-seed` | world creation (truth seeding incl. astrobiology) | per `(target, property)` |
| `obs-noise` | `Observe` | per `(faction, target, property, seq)` |
| `prospect` | `Prospect` | per `(field, draw-seq)` |

Keying is order-independent and replay-stable (BLAKE3-derived sub-streams, FA-01 RNG pattern), so
command interleaving across modules never changes outcomes.
