//! # sojourn-ui — UI view-model + host (FA-10)
//!
//! The **headless, testable presentation logic** behind Sojourn's desktop UI. It maps
//! the cores' **read-only snapshots** to display data (the **view-model**) and submits
//! **journalled commands**, holding **no game logic and no authoritative state**
//! (Constitution IV). The egui renderer is a separate thin binary
//! (`sojourn-ui-desktop`); this crate has **no GUI dependency**, so the same core runs
//! headless and every bit of display logic is **unit-testable with no renderer**
//! (FR-UI-1506).
//!
//! Layout: pure primitives (`units` SI formatting, `trace` traceability rendering,
//! `command` plan→preview→commit, `table` virtualisation math, `widgets_data` the
//! bespoke-widget shapes), the `host` (an in-process `SimCore` owner exposing
//! status/events/snapshots/commands), the `read` sync layer, and `viewmodel/*` (one
//! pure `build()` per screen). The astrobiology view-model carries an **honesty
//! guard** (never a conclusive-positive the core has not set; no ground-truth input).

#![forbid(unsafe_code)]

pub mod command;
pub mod host;
pub mod modules;
pub mod read;
pub mod table;
pub mod trace;
pub mod units;
pub mod viewmodel;
pub mod widgets_data;

pub use command::{CommitOutcome, DraftPlan, Gate, Preview, TracedDelta};
pub use host::{Status, UiHost};
pub use trace::TraceRender;
pub use units::Quantity;
