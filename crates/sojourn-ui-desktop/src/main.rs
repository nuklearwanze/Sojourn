//! Sojourn desktop renderer (FA-10) — the thin egui/eframe shell over the
//! `sojourn-ui` view-model, styled to the "flight-console / instrument-bezel"
//! direction (`ui-references/ui-template.html`): deep-ink surfaces, phosphor-green
//! accents, bracketed panels, mono-forward console type. It owns no game logic and no
//! authoritative state: it hosts the core via `UiHost`, pulls the view-model and
//! paints it; every action routes back as a journalled command.

use eframe::egui;
use egui::{Color32, FontId, Pos2, Rect, RichText, Stroke, Vec2, pos2, vec2};
use sojourn_ui::host::{
    BaseCatalogueView, CraftRow, DomainView, EconomyView, MapBody, MilestoneRow, PersonnelRow,
    SojournalEntry, TransferOption, UiHost, VehicleReport, WorldView,
};
use sojourn_ui::viewmodel as vm;
use std::path::PathBuf;

/// AU in metres (heliocentric scale reference for the map).
const AU_M: f64 = 1.495_978_707e11;

// ---- Palette (from ui-references/ui-template.html design tokens) -------------
const VOID: Color32 = Color32::from_rgb(0x07, 0x0a, 0x0f);
const PANEL: Color32 = Color32::from_rgb(0x0d, 0x12, 0x19);
const PANEL2: Color32 = Color32::from_rgb(0x12, 0x18, 0x23);
const INSET: Color32 = Color32::from_rgb(0x07, 0x0b, 0x11);
const LINE: Color32 = Color32::from_rgb(0x1d, 0x26, 0x32);
const LINE_BRIGHT: Color32 = Color32::from_rgb(0x2e, 0x3d, 0x50);
const INK: Color32 = Color32::from_rgb(0xe1, 0xe8, 0xf2);
const INK_DIM: Color32 = Color32::from_rgb(0x85, 0x93, 0xa6);
const INK_FAINT: Color32 = Color32::from_rgb(0x56, 0x62, 0x73);
const SIGNAL: Color32 = Color32::from_rgb(0x3e, 0xe0, 0xa0); // phosphor go/live/selected
const PLOT: Color32 = Color32::from_rgb(0x5a, 0xa8, 0xff); // trajectories / data
const WARN: Color32 = Color32::from_rgb(0xff, 0xc1, 0x4d);
const SUN: Color32 = Color32::from_rgb(0xff, 0xd2, 0x7a);

fn repo_root() -> PathBuf {
    let mut starts: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        starts.push(dir.to_path_buf());
    }
    for start in starts {
        let mut dir: &std::path::Path = &start;
        loop {
            if dir.join("data/kernel").is_dir() {
                return dir.to_path_buf();
            }
            match dir.parent() {
                Some(p) => dir = p,
                None => break,
            }
        }
    }
    PathBuf::from(".")
}

/// Apply the console theme (sharp edges, deep ink, phosphor accents, mono-forward).
fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(INK);
    v.panel_fill = VOID;
    v.window_fill = PANEL;
    v.window_stroke = Stroke::new(1.0, LINE_BRIGHT);
    v.extreme_bg_color = INSET;
    v.faint_bg_color = PANEL2;
    let z = egui::Rounding::ZERO;
    v.window_rounding = z;
    v.menu_rounding = z;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.rounding = z;
        w.bg_stroke = Stroke::new(1.0, LINE);
    }
    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, INK_DIM);
    v.widgets.inactive.bg_fill = PANEL2;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, INK_DIM);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x19, 0x21, 0x2d);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, LINE_BRIGHT);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, INK);
    v.widgets.active.bg_fill = Color32::from_rgb(0x1c, 0x2a, 0x24);
    v.widgets.active.fg_stroke = Stroke::new(1.0, SIGNAL);
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(0x3e, 0xe0, 0xa0, 36);
    v.selection.stroke = Stroke::new(1.0, SIGNAL);
    v.hyperlink_color = PLOT;
    style.visuals = v;
    style.spacing.item_spacing = vec2(8.0, 6.0);
    style.spacing.button_padding = vec2(9.0, 4.0);
    ctx.set_style(style);
    ctx.set_pixels_per_point(1.25);
}

/// A console label (uppercase, spaced, faint, mono).
fn lbl(text: &str) -> RichText {
    RichText::new(text.to_uppercase())
        .monospace()
        .size(9.0)
        .color(INK_FAINT)
}

/// An instrument-panel frame (deep panel fill + line bezel).
fn panel_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, LINE))
        .inner_margin(egui::Margin::same(9.0))
}

/// Draw L-shaped tactical brackets at the four corners of a rect.
fn brackets(p: &egui::Painter, r: Rect, len: f32, stroke: Stroke) {
    let cs = [
        (r.left_top(), vec2(1.0, 0.0), vec2(0.0, 1.0)),
        (r.right_top(), vec2(-1.0, 0.0), vec2(0.0, 1.0)),
        (r.left_bottom(), vec2(1.0, 0.0), vec2(0.0, -1.0)),
        (r.right_bottom(), vec2(-1.0, 0.0), vec2(0.0, -1.0)),
    ];
    for (c, a, b) in cs {
        p.line_segment([c, c + a * len], stroke);
        p.line_segment([c, c + b * len], stroke);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Map,
    Trajectory,
    Operations,
    Bases,
    Research,
    Vehicle,
    Economy,
    Personnel,
    World,
    Astro,
    Journal,
    Alerts,
}

impl Screen {
    /// The S-code, label, and nav group for every screen (matches the template).
    const NAV: &'static [(&'static str, &'static str, Screen, &'static str)] = &[
        ("OPERATE", "", Screen::Map, ""), // group header sentinel
        ("S1", "System Map", Screen::Map, "OPERATE"),
        ("S2", "Trajectory", Screen::Trajectory, "OPERATE"),
        ("S5", "Operations", Screen::Operations, "OPERATE"),
        ("S7", "Bases", Screen::Bases, "OPERATE"),
        ("S3", "Research & Dev", Screen::Research, "DEVELOP"),
        ("S4", "Vehicle Designer", Screen::Vehicle, "DEVELOP"),
        ("S6", "Economy", Screen::Economy, "DEVELOP"),
        ("S8", "Personnel", Screen::Personnel, "DEVELOP"),
        ("S9", "World / Politics", Screen::World, "DISCOVER"),
        ("S10", "Astrobiology", Screen::Astro, "DISCOVER"),
        ("S11", "Sojournal", Screen::Journal, "DISCOVER"),
        ("S12", "Event Log", Screen::Alerts, "DISCOVER"),
    ];
}

struct App {
    host: UiHost,
    screen: Screen,
    warp_days_per_frame: u64,
    running: bool,
    // System Map UI-ephemeral state (never in the deterministic save).
    map_bodies: Vec<MapBody>,
    map_pulled_tick: Option<u64>,
    map_px_per_au: f64,
    map_pan: Vec2,
    selected: Option<u32>,
    // R&D screen UI-ephemeral state.
    domains: Vec<DomainView>,
    domains_pulled_tick: Option<u64>,
    sel_domain: Option<String>,
    // Vehicle Designer UI-ephemeral state.
    vehicle: Option<VehicleReport>,
    vehicle_pulled_tick: Option<u64>,
    // Economy UI-ephemeral state.
    economy: Option<EconomyView>,
    economy_pulled_tick: Option<u64>,
    // World/Politics UI-ephemeral state.
    world: Option<WorldView>,
    world_pulled_tick: Option<u64>,
    // Remaining screens (static catalogues pulled once; live ones per tick).
    transfers: Option<Vec<TransferOption>>,
    bases: Option<BaseCatalogueView>,
    sojournal: Option<Vec<SojournalEntry>>,
    fleet: Vec<CraftRow>,
    fleet_pulled_tick: Option<u64>,
    personnel: Vec<PersonnelRow>,
    personnel_pulled_tick: Option<u64>,
}

impl App {
    fn new(host: UiHost) -> App {
        App {
            host,
            screen: Screen::Map,
            warp_days_per_frame: 0,
            running: false,
            map_bodies: Vec::new(),
            map_pulled_tick: None,
            map_px_per_au: 36.0,
            map_pan: Vec2::ZERO,
            selected: None,
            domains: Vec::new(),
            domains_pulled_tick: None,
            sel_domain: None,
            vehicle: None,
            vehicle_pulled_tick: None,
            economy: None,
            economy_pulled_tick: None,
            world: None,
            world_pulled_tick: None,
            transfers: None,
            bases: None,
            sojournal: None,
            fleet: Vec::new(),
            fleet_pulled_tick: None,
            personnel: Vec::new(),
            personnel_pulled_tick: None,
        }
    }

    fn sync_map(&mut self, tick: u64) {
        if self.map_pulled_tick != Some(tick) {
            self.map_bodies = self.host.map_bodies();
            self.map_pulled_tick = Some(tick);
        }
    }

    fn sync_research(&mut self, tick: u64) {
        if self.domains_pulled_tick != Some(tick) {
            self.domains = self.host.research_domains();
            self.domains_pulled_tick = Some(tick);
        }
    }

    fn sync_vehicle(&mut self, tick: u64) {
        if self.vehicle_pulled_tick != Some(tick) {
            self.vehicle = self.host.vehicle_report();
            self.vehicle_pulled_tick = Some(tick);
        }
    }

    fn sync_economy(&mut self, tick: u64) {
        if self.economy_pulled_tick != Some(tick) {
            self.economy = self.host.economy_overview();
            self.economy_pulled_tick = Some(tick);
        }
    }

    fn sync_world(&mut self, tick: u64) {
        if self.world_pulled_tick != Some(tick) {
            self.world = self.host.world_overview();
            self.world_pulled_tick = Some(tick);
        }
    }

    fn sync_fleet(&mut self, tick: u64) {
        if self.fleet_pulled_tick != Some(tick) {
            self.fleet = self.host.fleet();
            self.fleet_pulled_tick = Some(tick);
        }
    }

    fn sync_personnel(&mut self, tick: u64) {
        if self.personnel_pulled_tick != Some(tick) {
            self.personnel = self.host.personnel_summary();
            self.personnel_pulled_tick = Some(tick);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let pending = self.host.pending_interrupts();
        if !pending.is_empty() {
            self.running = false;
        } else if self.running && self.warp_days_per_frame > 0 {
            let _ = self.host.advance(self.warp_days_per_frame * 86_400);
            ctx.request_repaint();
        }

        let status = self.host.status();
        let events = self.host.recent_events(64);
        let shell = vm::shell(&status, &events);
        let elapsed_d = status.tick / 86_400;

        self.top_bar(ctx, &shell, elapsed_d, &pending);
        self.left_nav(ctx);
        self.event_log(ctx, &events);
        self.inspector(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(VOID))
            .show(ctx, |ui| match self.screen {
                Screen::Map => {
                    self.sync_map(status.tick);
                    self.show_map(ui);
                }
                Screen::Research => {
                    self.sync_research(status.tick);
                    self.show_research(ui);
                }
                Screen::Vehicle => {
                    self.sync_vehicle(status.tick);
                    self.show_vehicle(ui);
                }
                Screen::Economy => {
                    self.sync_economy(status.tick);
                    self.show_economy(ui);
                }
                Screen::World => {
                    self.sync_world(status.tick);
                    self.show_world(ui);
                }
                Screen::Trajectory => self.show_trajectory(ui),
                Screen::Operations => {
                    self.sync_fleet(status.tick);
                    self.show_operations(ui);
                }
                Screen::Bases => self.show_bases(ui),
                Screen::Personnel => {
                    self.sync_personnel(status.tick);
                    self.show_personnel(ui);
                }
                Screen::Astro => self.show_astrobiology(ui),
                Screen::Journal => self.show_sojournal(ui),
                Screen::Alerts => self.show_alerts(ui, &events),
            });
    }
}

impl App {
    fn top_bar(
        &mut self,
        ctx: &egui::Context,
        shell: &vm::ShellVM,
        elapsed_d: u64,
        pending: &[(u64, String)],
    ) {
        let frame = egui::Frame::none()
            .fill(Color32::from_rgb(0x0b, 0x0f, 0x16))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .stroke(Stroke::new(1.0, LINE));
        egui::TopBottomPanel::top("top")
            .frame(frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("SOJOURN").strong().size(15.0).color(INK));
                        ui.label(lbl("Mission Console"));
                    });
                    ui.add_space(8.0);
                    led(ui, PLOT);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("ESA").strong().size(12.0).color(INK));
                        ui.label(lbl("Flight Ops // EU"));
                    });
                    ui.add_space(14.0);
                    // Clock.
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&shell.date).monospace().size(15.0).color(INK));
                        ui.label(
                            RichText::new(format!("T+{elapsed_d:>4}d  ·  tick {}", shell.tick))
                                .monospace()
                                .size(9.5)
                                .color(INK_FAINT),
                        );
                    });
                    ui.add_space(12.0);
                    // Run state.
                    let (rl, rc) = if self.running {
                        ("RUNNING", SIGNAL)
                    } else {
                        ("PAUSED", WARN)
                    };
                    led(ui, rc);
                    ui.label(RichText::new(rl).monospace().size(10.0).color(rc));
                    ui.add_space(10.0);
                    // Warp.
                    if ui.add(warp_btn("❚❚", !self.running)).clicked() {
                        self.running = false;
                    }
                    for d in [1u64, 7, 30] {
                        if ui
                            .add(warp_btn(
                                &format!("{d}d"),
                                self.running && self.warp_days_per_frame == d,
                            ))
                            .clicked()
                        {
                            self.warp_days_per_frame = d;
                            self.running = true;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Interrupt / acknowledge.
                        if !pending.is_empty() {
                            let classes: Vec<&str> =
                                pending.iter().map(|(_, c)| c.as_str()).collect();
                            if ui.add(warp_btn("✓ ACK & RESUME", false)).clicked() {
                                let _ = self.host.acknowledge_all();
                            }
                            ui.label(
                                RichText::new(format!("⚠ PAUSED · {}", classes.join(", ")))
                                    .monospace()
                                    .size(10.0)
                                    .color(WARN),
                            );
                        } else {
                            ui.label(
                                RichText::new(format!("◈ {} events", shell.tick.min(99999)))
                                    .monospace()
                                    .size(10.0)
                                    .color(INK_FAINT),
                            );
                        }
                    });
                });
            });
    }

    fn left_nav(&mut self, ctx: &egui::Context) {
        let frame = egui::Frame::none()
            .fill(Color32::from_rgb(0x0a, 0x0e, 0x14))
            .inner_margin(egui::Margin::same(9.0))
            .stroke(Stroke::new(1.0, LINE));
        egui::SidePanel::left("nav")
            .exact_width(196.0)
            .frame(frame)
            .show(ctx, |ui| {
                let mut last_group = "";
                for (code, name, screen, group) in Screen::NAV.iter() {
                    if name.is_empty() {
                        continue; // sentinel
                    }
                    if *group != last_group {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(lbl(group));
                            let r = ui.available_rect_before_wrap();
                            ui.painter().line_segment(
                                [
                                    pos2(r.left() + 4.0, r.center().y),
                                    pos2(r.right(), r.center().y),
                                ],
                                Stroke::new(1.0, LINE),
                            );
                        });
                        last_group = group;
                    }
                    let selected = self.screen == *screen;
                    let txt = RichText::new(format!("{code:<3}  {name}"))
                        .size(12.0)
                        .color(if selected { INK } else { INK_DIM });
                    let resp = ui.add_sized(
                        [ui.available_width(), 24.0],
                        egui::SelectableLabel::new(selected, txt),
                    );
                    if selected {
                        let r = resp.rect;
                        ui.painter().line_segment(
                            [
                                pos2(r.left(), r.top() + 3.0),
                                pos2(r.left(), r.bottom() - 3.0),
                            ],
                            Stroke::new(2.0, SIGNAL),
                        );
                    }
                    if resp.clicked() {
                        self.screen = *screen;
                    }
                }
            });
    }

    fn event_log(&self, ctx: &egui::Context, events: &[sojourn_ui::host::EventView]) {
        let frame = egui::Frame::none()
            .fill(Color32::from_rgb(0x0a, 0x0e, 0x14))
            .stroke(Stroke::new(1.0, LINE));
        egui::TopBottomPanel::bottom("log")
            .exact_height(118.0)
            .frame(frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.label(lbl("Event Log"));
                    ui.label(
                        RichText::new("⏸ = pauses the game (configurable)")
                            .monospace()
                            .size(9.5)
                            .color(INK_FAINT),
                    );
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for e in events.iter().rev() {
                            ui.horizontal(|ui| {
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new(format!("@{:>9}", e.tick))
                                        .monospace()
                                        .size(10.5)
                                        .color(INK_FAINT),
                                );
                                ui.label(
                                    RichText::new(format!("{:<20}", e.class))
                                        .monospace()
                                        .size(9.5)
                                        .color(class_color(&e.class)),
                                );
                                ui.label(RichText::new(&e.class).size(11.5).color(INK_DIM));
                            });
                        }
                    });
            });
    }

    fn inspector(&self, ctx: &egui::Context) {
        let frame = egui::Frame::none()
            .fill(Color32::from_rgb(0x0a, 0x0e, 0x14))
            .inner_margin(egui::Margin::same(12.0))
            .stroke(Stroke::new(1.0, LINE));
        egui::SidePanel::right("insp")
            .exact_width(296.0)
            .frame(frame)
            .show(ctx, |ui| {
                // On the R&D screen, the inspector pins the selected knowledge domain and
                // shows its Understanding-Level breakdown (full traceability, FR-UI-301).
                if self.screen == Screen::Research {
                    self.inspect_domain(ui);
                    return;
                }
                match self
                    .selected
                    .and_then(|id| self.map_bodies.iter().find(|b| b.id == id))
                {
                    Some(b) => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&b.name).strong().size(14.0).color(INK));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new("PINNED")
                                            .monospace()
                                            .size(9.0)
                                            .color(INK_FAINT),
                                    );
                                },
                            );
                        });
                        ui.label(
                            RichText::new("SOLAR-SYSTEM BODY")
                                .monospace()
                                .size(9.5)
                                .color(PLOT),
                        );
                        ui.add_space(8.0);
                        sec(ui, "State");
                        kv(ui, "Catalogue id", &format!("{}", b.id));
                        kv(
                            ui,
                            "Mean radius",
                            &sojourn_ui::units::format(
                                sojourn_ui::units::Quantity::Length,
                                b.radius,
                            ),
                        );
                        kv(
                            ui,
                            "Heliocentric r",
                            &format!("{:.3} AU", b.pos.0.hypot(b.pos.1) / AU_M),
                        );
                        kv(ui, "Frame", "heliocentric · inertial");
                        ui.add_space(10.0);
                        sec(ui, "Context actions");
                        ui.label(
                            RichText::new("Plan transfer · Show in Sojournal")
                                .size(11.0)
                                .color(INK_FAINT),
                        );
                    }
                    None => {
                        ui.label(lbl("Inspector"));
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Click a body on the System Map to pin its inspector.")
                                .size(11.5)
                                .color(INK_FAINT),
                        );
                    }
                }
            });
    }

    /// The System Map — a 2D log-zoom heliocentric console view (R15 read from the
    /// astro surface), with orbit rings, tactical brackets, a reticle on the selection,
    /// a scale bar, and click-to-inspect. The UI computes no positions.
    fn show_map(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        let p = ui.painter_at(rect);
        // Inset backdrop + faint graticule.
        p.rect_filled(rect, 0.0, INSET);
        let grid = Stroke::new(1.0, Color32::from_rgb(0x14, 0x1d, 0x28));
        let step = 48.0;
        let mut x = rect.left();
        while x < rect.right() {
            p.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], grid);
            x += step;
        }
        let mut y = rect.top();
        while y < rect.bottom() {
            p.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], grid);
            y += step;
        }

        if response.dragged() {
            self.map_pan += response.drag_delta();
        }
        // Zoom about the cursor: keep the world point under the pointer fixed.
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0
                && let Some(cursor) = response.hover_pos()
            {
                let pre_c = rect.center() + self.map_pan;
                let pre_s = self.map_px_per_au / AU_M;
                let wx = (cursor.x - pre_c.x) as f64 / pre_s;
                let wy = -((cursor.y - pre_c.y) as f64) / pre_s;
                self.map_px_per_au *= libm::exp((scroll as f64) * 0.0015);
                self.map_px_per_au = self.map_px_per_au.clamp(1.0e-3, 1.0e7);
                let post_s = self.map_px_per_au / AU_M;
                let ncx = cursor.x as f64 - wx * post_s;
                let ncy = cursor.y as f64 + wy * post_s;
                self.map_pan = vec2(
                    (ncx - rect.center().x as f64) as f32,
                    (ncy - rect.center().y as f64) as f32,
                );
            }
        }

        let center = rect.center() + self.map_pan;
        let px_per_m = self.map_px_per_au / AU_M;
        let project = |b: &MapBody| {
            pos2(
                (center.x as f64 + b.pos.0 * px_per_m) as f32,
                (center.y as f64 - b.pos.1 * px_per_m) as f32,
            )
        };

        // Real orbital paths: each body's sampled ellipse, projected as a polyline
        // (stationary — the actual path, not a circle through the dot). Planet orbits
        // read brighter; minor-body orbits are darker grey so they recede.
        let orbit_planet = Stroke::new(1.0, Color32::from_rgb(0x22, 0x30, 0x40));
        let orbit_minor = Stroke::new(1.0, Color32::from_rgb(0x14, 0x1a, 0x22));
        for b in &self.map_bodies {
            if b.orbit.len() < 2 {
                continue;
            }
            let pts: Vec<Pos2> = b
                .orbit
                .iter()
                .map(|&(x, y)| {
                    pos2(
                        (center.x as f64 + x * px_per_m) as f32,
                        (center.y as f64 - y * px_per_m) as f32,
                    )
                })
                .collect();
            if pts.iter().all(|q| !rect.expand(120.0).contains(*q)) {
                continue; // orbit entirely off-screen
            }
            let stroke = if planet_style(&b.name).is_some() {
                orbit_planet
            } else {
                orbit_minor
            };
            p.add(egui::Shape::line(pts, stroke));
        }

        // Bodies (LOD via the tested view-model seam) + labels.
        let inputs: Vec<vm::BodyInput> = self
            .map_bodies
            .iter()
            .map(|b| {
                let s = project(b);
                vm::BodyInput {
                    id: b.id,
                    name: b.name.clone(),
                    pos: (s.x as f64, s.y as f64),
                    apparent_px: (b.radius * px_per_m).max(0.0),
                }
            })
            .collect();
        let labelled = vm::map(&inputs, 3.0);
        for inp in &inputs {
            let pt = pos2(inp.pos.0 as f32, inp.pos.1 as f32);
            if !rect.expand(40.0).contains(pt) {
                continue;
            }
            let sel = self.selected == Some(inp.id);
            let lname = inp.name.to_ascii_lowercase();
            if lname == "sol" {
                // The Sun: amber disk with a corona glow.
                let r = (inp.apparent_px as f32).clamp(6.0, 18.0);
                p.circle_filled(pt, r, if sel { SIGNAL } else { SUN });
                p.circle_stroke(
                    pt,
                    r + 6.0,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xff, 0xd2, 0x7a, 40)),
                );
                p.text(
                    pt + vec2(r + 5.0, -2.0),
                    egui::Align2::LEFT_CENTER,
                    &inp.name,
                    FontId::monospace(10.5),
                    INK_DIM,
                );
            } else if let Some((col, minr, ring)) = planet_style(&inp.name) {
                // A stylised planet/major moon: coloured disk + dark limb (+ Saturn ring).
                let r = (inp.apparent_px as f32).max(minr).min(16.0);
                let c = if sel { SIGNAL } else { col };
                if ring {
                    draw_ellipse(
                        &p,
                        pt,
                        r * 2.1,
                        r * 0.62,
                        Stroke::new(1.0, c.gamma_multiply(0.85)),
                    );
                }
                p.circle_filled(pt, r, c);
                p.circle_stroke(pt, r, Stroke::new(1.0, VOID));
                p.text(
                    pt + vec2(r + 4.0, -2.0),
                    egui::Align2::LEFT_CENTER,
                    &inp.name,
                    FontId::monospace(10.0),
                    if sel { SIGNAL } else { INK_DIM },
                );
            } else {
                // A minor body: a small grey dot.
                let r = (inp.apparent_px as f32).clamp(1.4, 6.0);
                p.circle_filled(
                    pt,
                    r,
                    if sel {
                        SIGNAL
                    } else {
                        Color32::from_rgb(0x70, 0x7c, 0x8c)
                    },
                );
                if inp.apparent_px >= 3.0 || sel {
                    p.text(
                        pt + vec2(r + 4.0, -2.0),
                        egui::Align2::LEFT_CENTER,
                        &inp.name,
                        FontId::monospace(9.5),
                        INK_FAINT,
                    );
                }
            }
        }

        // Reticle on the selection.
        if let Some(id) = self.selected
            && let Some(inp) = inputs.iter().find(|i| i.id == id)
        {
            let pt = pos2(inp.pos.0 as f32, inp.pos.1 as f32);
            let rs = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x3e, 0xe0, 0xa0, 160));
            p.circle_stroke(pt, 15.0, rs);
            for d in [
                vec2(0.0, -22.0),
                vec2(0.0, 22.0),
                vec2(-22.0, 0.0),
                vec2(22.0, 0.0),
            ] {
                p.line_segment([pt + d * 0.7, pt + d], rs);
            }
        }

        // Double-click recentres on the clicked object (or the clicked point);
        // single-click selects the nearest body.
        if response.double_clicked()
            && let Some(click) = response.interact_pointer_pos()
        {
            let mut target = None;
            let mut best = 14.0f32;
            for inp in &inputs {
                let d = pos2(inp.pos.0 as f32, inp.pos.1 as f32).distance(click);
                if d < best {
                    best = d;
                    target = Some(inp.id);
                }
            }
            // world point to centre on: the picked body, else the cursor location.
            let w = target
                .and_then(|id| self.map_bodies.iter().find(|b| b.id == id))
                .map(|b| b.pos)
                .unwrap_or((
                    (click.x - center.x) as f64 / px_per_m,
                    -((click.y - center.y) as f64) / px_per_m,
                ));
            self.map_pan = vec2((-w.0 * px_per_m) as f32, (w.1 * px_per_m) as f32);
            if let Some(id) = target {
                self.selected = Some(id);
            }
        } else if response.clicked()
            && let Some(click) = response.interact_pointer_pos()
        {
            let mut best: Option<(f32, u32)> = None;
            for inp in &inputs {
                let d = pos2(inp.pos.0 as f32, inp.pos.1 as f32).distance(click);
                if d < 8.0 && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                    best = Some((d, inp.id));
                }
            }
            if let Some((_, id)) = best {
                self.selected = Some(id);
            }
        }

        // Tactical brackets + console overlays.
        brackets(&p, rect.shrink(12.0), 26.0, Stroke::new(1.0, LINE_BRIGHT));
        p.text(
            rect.left_top() + vec2(24.0, 22.0),
            egui::Align2::LEFT_TOP,
            "S1 · SYSTEM MAP",
            FontId::monospace(12.0),
            SIGNAL,
        );
        p.text(
            rect.left_top() + vec2(24.0, 40.0),
            egui::Align2::LEFT_TOP,
            format!(
                "HELIOCENTRIC · INERTIAL · {:.2} PX/AU · {} BODIES",
                self.map_px_per_au,
                self.map_bodies.len()
            ),
            FontId::monospace(9.5),
            INK_DIM,
        );
        p.text(
            rect.right_top() + vec2(-24.0, 22.0),
            egui::Align2::RIGHT_TOP,
            "● LIVE FEED",
            FontId::monospace(9.5),
            PLOT,
        );
        p.text(
            rect.left_bottom() + vec2(24.0, -16.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "{} LABELLED · {} MINOR (LOD)",
                labelled.bodies.len(),
                labelled.culled
            ),
            FontId::monospace(9.5),
            INK_FAINT,
        );
        // Scale bar (1 AU).
        let sb = pos2(
            rect.right() - 24.0 - self.map_px_per_au.clamp(20.0, 160.0) as f32,
            rect.bottom() - 18.0,
        );
        let se = pos2(rect.right() - 24.0, rect.bottom() - 18.0);
        p.line_segment([sb, se], Stroke::new(1.0, INK_DIM));
        p.text(
            sb + vec2(-4.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            "0",
            FontId::monospace(9.0),
            INK_FAINT,
        );
        p.text(
            se + vec2(4.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "≈1 AU",
            FontId::monospace(9.0),
            INK_FAINT,
        );
    }

    /// The Research & Development screen (S3, FR-UI-301): the player faction's
    /// domain Understanding Levels as a portfolio of bars, each reading straight from
    /// the FA-05 research surface — private UL fill, the shared world-tide ghost, and
    /// the diminishing-returns knee. Click a row to pin the domain in the inspector.
    fn show_research(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("S3").monospace().size(11.0).color(SIGNAL));
                ui.label(RichText::new("RESEARCH & DEVELOPMENT").strong().size(15.0).color(INK));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("● LIVE").monospace().size(9.5).color(PLOT));
                });
            });
            ui.label(RichText::new("KNOWLEDGE DOMAINS · UNDERSTANDING LEVEL 0–100 · WORLD-TIDE GHOST · DIMINISHING-RETURNS KNEE").monospace().size(9.5).color(INK_DIM));
            ui.add_space(12.0);

            panel_frame().show(ui, |ui| {
                ui.set_min_size(vec2(ui.available_width(), ui.available_height() - 8.0));
                // Equipment header.
                ui.horizontal(|ui| {
                    ui.label(lbl("Domain Understanding"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{} DOMAINS", self.domains.len())).monospace().size(9.0).color(INK_FAINT));
                    });
                });
                ui.separator();
                // Legend.
                ui.horizontal(|ui| {
                    swatch(ui, SIGNAL, "private UL");
                    swatch(ui, PLOT, "world tide");
                    swatch(ui, WARN, "DR knee");
                    swatch(ui, Color32::from_rgb(0x5b, 0x9b, 0xd6), "basic");
                    swatch(ui, Color32::from_rgb(0xe3, 0xc9, 0x8c), "applied");
                });
                ui.add_space(4.0);

                let domains = self.domains.clone();
                let mut clicked: Option<String> = None;
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for d in &domains {
                        let sel = self.sel_domain.as_deref() == Some(d.id.as_str());
                        if ul_row(ui, d, sel).clicked() {
                            clicked = Some(d.id.clone());
                        }
                    }
                });
                if let Some(id) = clicked {
                    self.sel_domain = if self.sel_domain.as_deref() == Some(id.as_str()) { None } else { Some(id) };
                }
            });
        });
    }

    /// The inspector body for a pinned knowledge domain — its UL components, classes
    /// and the effective-UL derivation (`effective = private − tacit penalty`), so the
    /// gated figure is fully traceable to its inputs (FR-UI-301 traceability).
    fn inspect_domain(&self, ui: &mut egui::Ui) {
        let Some(d) = self
            .sel_domain
            .as_deref()
            .and_then(|id| self.domains.iter().find(|d| d.id == id))
        else {
            ui.label(lbl("Inspector"));
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Click a domain on the R&D portfolio to pin its Understanding-Level breakdown.",
                )
                .size(11.5)
                .color(INK_FAINT),
            );
            return;
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(&d.name).strong().size(14.0).color(INK));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(&d.id).monospace().size(10.0).color(INK_FAINT));
            });
        });
        let class = if d.basic_science {
            "BASIC SCIENCE"
        } else {
            "APPLIED SCIENCE"
        };
        ui.label(
            RichText::new(class)
                .monospace()
                .size(9.5)
                .color(if d.basic_science { PLOT } else { WARN }),
        );
        if d.mission_injectable {
            ui.label(
                RichText::new("⊙ MISSION-FED · field work injects UL")
                    .monospace()
                    .size(9.0)
                    .color(SIGNAL),
            );
        }
        ui.add_space(8.0);
        sec(ui, "Understanding level");
        kv(ui, "Private UL", &format!("{:.1} / 100", d.private_ul));
        kv(ui, "World tide", &format!("{:.1} / 100", d.world_ul));
        kv(ui, "Effective UL", &format!("{:.1} / 100", d.effective_ul));
        ui.add_space(10.0);
        sec(ui, "Derivation");
        let tacit = (d.private_ul - d.effective_ul).max(0.0);
        kv(ui, "effective", "= private − tacit");
        kv(ui, "tacit penalty", &format!("{tacit:.1}"));
        ui.label(RichText::new("Tacit penalty rises when a domain has no experienced personnel; 0 here until a roster is assigned.").size(10.0).color(INK_FAINT));
        ui.add_space(10.0);
        sec(ui, "Diminishing returns");
        kv(ui, "DR knee", &format!("UL {:.0}", d.dr_knee));
        ui.label(
            RichText::new("Above the knee, each UL point costs sharply more RP.")
                .size(10.0)
                .color(INK_FAINT),
        );
    }

    /// The Vehicle Designer (S4, FR-VD-201/501/601): a reference design derived by the
    /// vehicle slice — every figure (mass, Δv, power, reliability, cost), a Δv ladder,
    /// the realism red-flags, and the full parts palette with FA-05 maturity. The UI
    /// computes none of it; it reads the slice's derivation and lays it out.
    fn show_vehicle(&mut self, ui: &mut egui::Ui) {
        use sojourn_ui::units::{Quantity, format as fmt};
        let report = self.vehicle.clone();
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("S4").monospace().size(11.0).color(SIGNAL));
                ui.label(RichText::new("VEHICLE DESIGNER").strong().size(15.0).color(INK));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("◈ REFERENCE DESIGN").monospace().size(9.5).color(PLOT));
                });
            });
            let Some(rep) = report else {
                ui.label(RichText::new("VEHICLE SLICE UNAVAILABLE").monospace().size(10.0).color(WARN));
                return;
            };
            ui.label(RichText::new(format!("{} · CLASS {} · DERIVED AT 1 AU · MATURITY FROM R&D (C1 DECOUPLED)", rep.name.to_uppercase(), rep.class.to_uppercase())).monospace().size(9.5).color(INK_DIM));
            ui.add_space(12.0);

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                // --- derived performance: stat tiles ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Derived Performance"));
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        stat_tile(ui, "dry mass", &fmt(Quantity::Mass, rep.dry_mass), PLOT);
                        stat_tile(ui, "wet mass", &fmt(Quantity::Mass, rep.wet_mass), PLOT);
                        stat_tile(ui, "propellant", &fmt(Quantity::Mass, rep.propellant), INK_DIM);
                        stat_tile(ui, "total Δv", &fmt(Quantity::Velocity, rep.total_dv), SIGNAL);
                        stat_tile(ui, "1st-stage thrust", &format!("{:.0} kN", rep.max_thrust / 1.0e3), INK_DIM);
                        let rc = if rep.reliability >= 0.6 { SIGNAL } else { WARN };
                        stat_tile(ui, "reliability", &format!("{:.0}%", rep.reliability * 100.0), rc);
                        stat_tile(ui, "unit cost", &format!("{:.0}", rep.unit_cost), INK_DIM);
                        stat_tile(ui, "build", &format!("{:.0} d", rep.build_days), INK_DIM);
                    });
                });
                ui.add_space(10.0);

                // --- power / thermal balance ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Power & Thermal Balance"));
                    ui.separator();
                    bar_balance(ui, "power", rep.power_gen, rep.power_demand, rep.power_margin);
                    let tcol = if rep.thermal_margin >= 0.0 { SIGNAL } else { Color32::from_rgb(0xff, 0x5e, 0x5e) };
                    kv_row(ui, "thermal margin", &fmt(Quantity::Power, rep.thermal_margin), tcol);
                });
                ui.add_space(10.0);

                // --- Δv ladder ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Δv Ladder"));
                    ui.separator();
                    dv_ladder(ui, 6_000.0, rep.total_dv, &rep.stage_dvs);
                });
                ui.add_space(10.0);

                // --- realism red-flags ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Realism Red-Flags"));
                    ui.separator();
                    if rep.red_flags.is_empty() {
                        ui.label(RichText::new("✓ no red-flags — physically buildable as specified").monospace().size(11.0).color(SIGNAL));
                    } else {
                        for f in &rep.red_flags {
                            let (col, tag) = if f.hard { (Color32::from_rgb(0xff, 0x5e, 0x5e), "HARD") } else { (WARN, "SOFT") };
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("⚑ {tag}")).monospace().size(10.0).color(col));
                                ui.label(RichText::new(&f.constraint).size(11.5).color(INK));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("Δ {:.1}", f.value)).monospace().size(10.5).color(INK_DIM));
                                });
                            });
                        }
                    }
                });
                ui.add_space(10.0);

                // --- component palette ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(lbl("Component Catalogue"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{} PARTS · TRL FROM R&D", rep.components.len())).monospace().size(9.0).color(INK_FAINT));
                        });
                    });
                    ui.separator();
                    ui.label(RichText::new(format!("{:<16}{:<13}{:<26}{:>9}   {}", "PART", "CLASS", "SPEC", "MASS", "TRL")).monospace().size(8.5).color(INK_FAINT));
                    for c in &rep.components {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:<16}", trunc(&c.id, 15))).monospace().size(10.5).color(INK));
                            ui.label(RichText::new(format!("{:<13}", c.class)).monospace().size(10.0).color(INK_DIM));
                            ui.label(RichText::new(format!("{:<26}", trunc(&c.detail, 25))).monospace().size(10.0).color(INK_DIM));
                            ui.label(RichText::new(format!("{:>9}", fmt(Quantity::Mass, c.dry_mass))).monospace().size(10.0).color(INK_DIM));
                            let (tc, tt) = if c.flyable { (SIGNAL, format!("TRL{}", c.trl)) } else if c.trl > 0 { (WARN, format!("TRL{}", c.trl)) } else { (INK_FAINT, "—".to_string()) };
                            ui.label(RichText::new(format!("   {tt}")).monospace().size(10.0).color(tc));
                        });
                    }
                    ui.add_space(4.0);
                    ui.label(RichText::new("Parts read flyable once their FA-05 tech reaches TRL 6 — none yet on a fresh programme.").size(9.5).color(INK_FAINT));
                });
            });
        });
    }

    /// The Economy & Contracts screen (S6, FR-EC-101/301/601/107): the conserved-
    /// currency ledger, the funding profile, the live launch market and the commodity
    /// taxonomy — every figure read from the economy surface. The UI computes nothing.
    fn show_economy(&mut self, ui: &mut egui::Ui) {
        use sojourn_ui::units::{Quantity, format as fmt};
        let view = self.economy.clone();
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("S6").monospace().size(11.0).color(SIGNAL));
                ui.label(RichText::new("ECONOMY & CONTRACTS").strong().size(15.0).color(INK));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("● LIVE").monospace().size(9.5).color(PLOT));
                });
            });
            let Some(view) = view else {
                ui.label(RichText::new("ECONOMY SLICE UNAVAILABLE").monospace().size(10.0).color(WARN));
                return;
            };
            ui.label(RichText::new("CONSERVED-CURRENCY LEDGER · FUNDING · LAUNCH MARKET · COMMODITY TAXONOMY").monospace().size(9.5).color(INK_DIM));
            ui.add_space(12.0);

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                // --- conserved-currency ledger ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(lbl("Conserved Currencies"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("FACTION 0 · ESA").monospace().size(9.0).color(INK_FAINT));
                        });
                    });
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        for b in &view.balances {
                            let val = if b.is_funds { fmt(Quantity::Currency, b.amount) } else { format!("{:.0}", b.amount) };
                            let col = if b.amount > 0.0 { SIGNAL } else { INK_FAINT };
                            stat_tile(ui, &b.name, &val, col);
                        }
                    });
                    ui.label(RichText::new("Every balance conserves — funds appear only via appropriation, sales or financing, never printed.").size(9.5).color(INK_FAINT));
                });
                ui.add_space(10.0);

                // --- funding profile ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Funding Profile"));
                    ui.separator();
                    ui.horizontal(|ui| {
                        led(ui, PLOT);
                        ui.label(RichText::new(&view.funding_kind).monospace().size(12.0).color(INK));
                    });
                    ui.label(RichText::new(&view.funding_headline).monospace().size(11.0).color(INK_DIM));
                });
                ui.add_space(10.0);

                // --- launch market ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Launch Market · $/kg by orbit class"));
                    ui.separator();
                    let cap = view.launch_prices.iter().map(|p| p.price_per_kg).fold(1.0_f64, f64::max);
                    for p in &view.launch_prices {
                        metric_bar(ui, &p.orbit_class, &format!("${:.0}/kg", p.price_per_kg), (p.price_per_kg / cap) as f32, PLOT);
                    }
                    ui.label(RichText::new("Prices fall as world reuse capacity grows — raising ISRU's relative value.").size(9.5).color(INK_FAINT));
                });
                ui.add_space(10.0);

                // --- commodity taxonomy ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(lbl("Commodity Taxonomy"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{} COMMODITIES", view.commodities.len())).monospace().size(9.0).color(INK_FAINT));
                        });
                    });
                    ui.separator();
                    ui.label(RichText::new(format!("{:<22}{:<28}{}", "COMMODITY", "KIND", "UNIT")).monospace().size(8.5).color(INK_FAINT));
                    for c in &view.commodities {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:<22}", trunc(&c.id, 21))).monospace().size(10.5).color(INK));
                            ui.label(RichText::new(format!("{:<28}", trunc(&c.kind, 27))).monospace().size(10.0).color(INK_DIM));
                            ui.label(RichText::new(&c.unit).monospace().size(10.0).color(INK_FAINT));
                        });
                    }
                });
            });
        });
    }

    /// The World & Politics screen (S9, FR-PEA-101/201): the historic-firsts race board
    /// grouped by era, the global science tide, and per-faction political state — read
    /// from the FA-09 polity surface. The UI computes nothing; it lays out the ledger.
    fn show_world(&mut self, ui: &mut egui::Ui) {
        let view = self.world.clone();
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("S9").monospace().size(11.0).color(SIGNAL));
                ui.label(RichText::new("WORLD & POLITICS").strong().size(15.0).color(INK));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("● LIVE").monospace().size(9.5).color(PLOT));
                });
            });
            let Some(view) = view else {
                ui.label(RichText::new("POLITY SLICE UNAVAILABLE").monospace().size(10.0).color(WARN));
                return;
            };
            let claimed = view.milestones.iter().filter(|m| m.claimed_by.is_some()).count();
            ui.label(RichText::new(format!("HISTORIC-FIRSTS RACE BOARD · {}/{} CLAIMED · SCIENCE TIDE · FACTION POLITICS", claimed, view.milestones.len())).monospace().size(9.5).color(INK_DIM));
            ui.add_space(12.0);

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                // --- science tide ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Global Science Tide"));
                    ui.separator();
                    let frac = (view.science_tide / 100.0).clamp(0.0, 1.0) as f32;
                    metric_bar(ui, "tide", &format!("{:.1}", view.science_tide), frac, PLOT);
                    ui.label(RichText::new("The shared world understanding the whole field rises on — discoveries lift it for everyone.").size(9.5).color(INK_FAINT));
                });
                ui.add_space(10.0);

                // --- faction politics ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Faction Politics"));
                    ui.separator();
                    if view.factions.is_empty() {
                        ui.label(RichText::new("⟳ AWAITING WORLD INITIALISATION").monospace().size(10.5).color(WARN));
                        ui.label(RichText::new("Mood, prestige and appropriation/approval modifiers appear once the world is seeded with factions.").size(9.5).color(INK_FAINT));
                    } else {
                        for f in &view.factions {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("FACTION {}", f.id)).monospace().size(11.0).color(INK));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("appr ×{:.2} · latency {:.0}d", f.appropriation_factor, f.approval_latency_days)).monospace().size(9.5).color(INK_DIM));
                                });
                            });
                            let mc = if f.mood >= 0.0 { SIGNAL } else { Color32::from_rgb(0xff, 0x5e, 0x5e) };
                            kv_row(ui, "mood", &format!("{:+.2}", f.mood), mc);
                            kv_row(ui, "prestige", &format!("{:.0}", f.prestige), INK_DIM);
                        }
                    }
                });
                ui.add_space(10.0);

                // --- historic-firsts race board ---
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(lbl("Historic Firsts · Race Board"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{} FIRSTS", view.milestones.len())).monospace().size(9.0).color(INK_FAINT));
                        });
                    });
                    ui.separator();
                    let mut era = "";
                    for m in &view.milestones {
                        if m.era != era {
                            era = &m.era;
                            ui.add_space(3.0);
                            ui.label(RichText::new(format!("▸ {} ERA", era.to_uppercase())).monospace().size(9.5).color(era_color(era)));
                        }
                        milestone_row(ui, m);
                    }
                });
            });
        });
    }

    /// The Trajectory planner (S2, FR-UI-201): heliocentric Hohmann Δv from Terra to
    /// each Sun-orbiting body, plotted as a Δv ladder. The figures are FA-02 two-body
    /// physics read from the host; the porkchop refines them per launch window.
    fn show_trajectory(&mut self, ui: &mut egui::Ui) {
        if self.transfers.is_none() {
            self.transfers = Some(self.host.transfer_options());
        }
        let transfers = self.transfers.clone().unwrap_or_default();
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            screen_head(ui, "S2", "TRAJECTORY PLANNER", "● LIVE", PLOT, "HELIOCENTRIC HOHMANN TRANSFERS FROM TERRA · TWO-BODY Δv + TIME-OF-FLIGHT FROM THE FA-02 CATALOGUE");
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Δv Ladder · Terra → body"));
                    ui.separator();
                    if transfers.is_empty() {
                        awaiting(ui, "NO TRANSFER TARGETS", "the catalogue yields no Sun-orbiting destinations to plan to.");
                    } else {
                        let cap = transfers.iter().map(|t| t.dv_mps).fold(1.0_f64, f64::max);
                        for t in &transfers {
                            metric_bar(ui, &t.to, &format!("{:.0} m/s · {:.0} d · {:.2} AU", t.dv_mps, t.tof_days, t.r_to_au), (t.dv_mps / cap) as f32, PLOT);
                        }
                        ui.add_space(4.0);
                        ui.label(RichText::new("Heliocentric two-burn Hohmann (circular-coplanar approximation); the porkchop planner refines per launch window.").size(9.5).color(INK_FAINT));
                    }
                });
            });
        });
    }

    /// The Operations / Fleet screen (S5, FR-UI-501): every propagated craft with its
    /// dominant body and flight status, read from the FA-02 store.
    fn show_operations(&mut self, ui: &mut egui::Ui) {
        let fleet = self.fleet.clone();
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            screen_head(ui, "S5", "OPERATIONS / FLEET", "● LIVE", PLOT, &format!("{} ASSETS · DOMINANT BODY · FLIGHT STATUS", fleet.len()));
            panel_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(lbl("Fleet"));
                ui.separator();
                if fleet.is_empty() {
                    awaiting(ui, "NO ASSETS IN FLIGHT", "craft appear here once you launch or spawn vehicles; status, dominant body and ops load track live.");
                } else {
                    ui.label(RichText::new(format!("{:<6}{:<22}{:<16}{}", "ID", "NAME", "DOMINANT", "STATUS")).monospace().size(8.5).color(INK_FAINT));
                    for c in &fleet {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:<6}", c.id)).monospace().size(10.5).color(INK_DIM));
                            ui.label(RichText::new(format!("{:<22}", trunc(&c.name, 21))).monospace().size(10.5).color(INK));
                            ui.label(RichText::new(format!("{:<16}", trunc(&c.dominant, 15))).monospace().size(10.0).color(INK_DIM));
                            ui.label(RichText::new(&c.status).monospace().size(10.0).color(SIGNAL));
                        });
                    }
                }
            });
        });
    }

    /// The Bases & Construction screen (S7, FR-BC-101): the buildable module catalogue
    /// and base-class templates (live from the first frame) + founded bases.
    fn show_bases(&mut self, ui: &mut egui::Ui) {
        use sojourn_ui::units::{Quantity, format as fmt};
        if self.bases.is_none() {
            self.bases = Some(self.host.base_catalogue());
        }
        let view = self.bases.clone();
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            screen_head(ui, "S7", "BASES & CONSTRUCTION", "● LIVE", PLOT, "MODULE CATALOGUE · CLASS TEMPLATES · FOUNDED BASES");
            let Some(view) = view else {
                ui.label(RichText::new("BASE SLICE UNAVAILABLE").monospace().size(10.0).color(WARN));
                return;
            };
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Founded Bases"));
                    ui.separator();
                    if view.founded == 0 {
                        awaiting(ui, "NO BASES FOUNDED", "found a base on a surveyed site to begin construction; power, closure and shielding then derive here.");
                    } else {
                        ui.label(RichText::new(format!("{} bases", view.founded)).monospace().size(11.0).color(INK));
                    }
                });
                ui.add_space(10.0);
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Base-Class Templates"));
                    ui.separator();
                    for c in &view.classes {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:<18}", c.id)).monospace().size(10.5).color(INK));
                            ui.label(RichText::new(if c.orbital { "orbital" } else { "surface" }).monospace().size(10.0).color(PLOT));
                            ui.label(RichText::new(if c.crewed { "crewed" } else { "uncrewed" }).monospace().size(10.0).color(if c.crewed { SIGNAL } else { INK_FAINT }));
                        });
                    }
                });
                ui.add_space(10.0);
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(lbl("Module Catalogue"));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{} MODULES", view.modules.len())).monospace().size(9.0).color(INK_FAINT));
                        });
                    });
                    ui.separator();
                    ui.label(RichText::new(format!("{:<18}{:<14}{:<8}{:>9}{:>10}", "MODULE", "CLASS", "TECH", "MASS", "POWER")).monospace().size(8.5).color(INK_FAINT));
                    for m in &view.modules {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:<18}", trunc(&m.id, 17))).monospace().size(10.5).color(INK));
                            ui.label(RichText::new(format!("{:<14}", m.class)).monospace().size(10.0).color(INK_DIM));
                            ui.label(RichText::new(format!("{:<8}", m.tech)).monospace().size(10.0).color(INK_FAINT));
                            ui.label(RichText::new(format!("{:>9}", fmt(Quantity::Mass, m.dry_mass))).monospace().size(10.0).color(INK_DIM));
                            ui.label(RichText::new(format!("{:>10}", fmt(Quantity::Power, m.power_demand))).monospace().size(10.0).color(INK_DIM));
                        });
                    }
                });
            });
        });
    }

    /// The Personnel screen (S8, FR-UI-801): head-count by role from the FA-05 roster.
    fn show_personnel(&mut self, ui: &mut egui::Ui) {
        let rows = self.personnel.clone();
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            screen_head(ui, "S8", "PERSONNEL", "● LIVE", PLOT, "ROSTER HEAD-COUNT BY ROLE · ASTRONAUT READINESS");
            panel_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(lbl("Roster by Role"));
                ui.separator();
                let total: u64 = rows.iter().filter(|r| r.role != "AstronautReady").map(|r| r.count).sum();
                if total == 0 {
                    awaiting(ui, "NO PERSONNEL HIRED", "hire researchers, engineers and astronauts; tacit knowledge, fatigue and astronaut readiness then track here.");
                }
                ui.horizontal_wrapped(|ui| {
                    for r in &rows {
                        let col = if r.count > 0 { SIGNAL } else { INK_FAINT };
                        stat_tile(ui, &r.role, &format!("{}", r.count), col);
                    }
                });
            });
        });
    }

    /// The Science Returns & Astrobiology screen (S10, FR-UI-1001): the **staged**
    /// evidence model — never a single "life found" moment — and the candidates under
    /// investigation. The honesty guard: the UI mirrors only published consensus, never
    /// ground truth, and no candidate exists until the world is initialised.
    fn show_astrobiology(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            screen_head(ui, "S10", "SCIENCE RETURNS & ASTROBIOLOGY", "◈ TRUTH-GUARDED", Color32::from_rgb(0xb9, 0x8b, 0xff), "STAGED EVIDENCE · ABIOTIC COMPETITORS · PRESTIGE-WEIGHTED CONSENSUS · SAMPLE-RETURN GATE");
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Staged Evidence Model"));
                    ui.separator();
                    let stages = [
                        "1 · Habitability established",
                        "2 · Prebiotic chemistry detected",
                        "3 · Candidate biosignature",
                        "4 · Corroborated, abiotic competitors weighed",
                        "5 · Conclusive — consensus ≥0.9 (or ≤0.1), sample-return gated",
                    ];
                    for (i, s) in stages.iter().enumerate() {
                        ui.horizontal(|ui| {
                            led(ui, INK_FAINT);
                            ui.label(RichText::new(*s).monospace().size(11.0).color(if i == 4 { Color32::from_rgb(0xb9, 0x8b, 0xff) } else { INK_DIM }));
                        });
                    }
                    ui.add_space(4.0);
                    ui.label(RichText::new("Evidence only ever accrues in stages and only published consensus is shown — there is no single 'life found' flag, and a positive is never certain over an abiotic explanation.").size(9.5).color(INK_FAINT));
                });
                ui.add_space(10.0);
                panel_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(lbl("Candidates Under Investigation"));
                    ui.separator();
                    awaiting(ui, "NO CANDIDATES YET", "candidate worlds and their evidence ladders populate once the world is initialised and prospecting/sampling missions return data.");
                });
            });
        });
    }

    /// The Sojournal encyclopedia (S11, FR-UI-1101): a source-cited browser assembled
    /// from the provenance every datum carries — the concrete proof that every figure
    /// in the game traces to a real, cited source.
    fn show_sojournal(&mut self, ui: &mut egui::Ui) {
        if self.sojournal.is_none() {
            self.sojournal = Some(self.host.sojournal_entries());
        }
        let entries = self.sojournal.clone().unwrap_or_default();
        egui::Frame::none()
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                screen_head(
                    ui,
                    "S11",
                    "SOJOURNAL",
                    "◈ SOURCED",
                    SIGNAL,
                    &format!(
                        "{} ENTRIES · EVERY FIGURE TRACES TO A CITED SOURCE",
                        entries.len()
                    ),
                );
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        panel_frame().show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let mut cat = "";
                            for e in &entries {
                                if e.category != cat {
                                    cat = &e.category;
                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new(format!("▸ {}", cat.to_uppercase()))
                                            .monospace()
                                            .size(9.5)
                                            .color(PLOT),
                                    );
                                    ui.separator();
                                }
                                ui.label(RichText::new(&e.title).monospace().size(11.0).color(INK));
                                ui.label(
                                    RichText::new(trunc(&e.source, 110))
                                        .size(9.5)
                                        .color(INK_FAINT),
                                );
                                ui.add_space(3.0);
                            }
                        });
                    });
            });
    }

    /// The Alerts / Event Log screen (S12, FR-UI-1201): the full event feed, newest
    /// first, class-coloured — the same journalled stream the bottom dock previews.
    fn show_alerts(&mut self, ui: &mut egui::Ui, events: &[sojourn_ui::host::EventView]) {
        egui::Frame::none().inner_margin(egui::Margin::same(16.0)).show(ui, |ui| {
            screen_head(ui, "S12", "ALERTS / EVENT LOG", "● LIVE", PLOT, &format!("{} EVENTS · CLASS-CODED · ⏸ CLASSES PAUSE THE GAME (CONFIGURABLE)", events.len()));
            panel_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(lbl("Event Feed · newest first"));
                ui.separator();
                if events.is_empty() {
                    awaiting(ui, "NO EVENTS YET", "the journalled event stream fills as the simulation advances; classes can be set to pause the game.");
                } else {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        for e in events.iter().rev() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("@{:>10}", e.tick)).monospace().size(10.0).color(INK_FAINT));
                                ui.label(RichText::new(format!("{:<22}", trunc(&e.class, 21))).monospace().size(10.5).color(class_color(&e.class)));
                                ui.label(RichText::new(format!("#{}", e.id)).monospace().size(9.5).color(INK_FAINT));
                            });
                        }
                    });
                }
            });
        });
    }
}

// ---- small console widgets --------------------------------------------------

/// A standard screen header: S-code + title + a right-aligned status tag + subtitle.
fn screen_head(ui: &mut egui::Ui, code: &str, title: &str, tag: &str, tag_col: Color32, sub: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(code).monospace().size(11.0).color(SIGNAL));
        ui.label(RichText::new(title).strong().size(15.0).color(INK));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(tag).monospace().size(9.5).color(tag_col));
        });
    });
    ui.label(RichText::new(sub).monospace().size(9.5).color(INK_DIM));
    ui.add_space(12.0);
}

/// An "awaiting state" note: an amber marker + a faint explanation of what activates it.
fn awaiting(ui: &mut egui::Ui, msg: &str, detail: &str) {
    ui.label(
        RichText::new(format!("⟳ {msg}"))
            .monospace()
            .size(10.5)
            .color(WARN),
    );
    ui.label(RichText::new(detail).size(9.5).color(INK_FAINT));
}

fn led(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(9.0, 9.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.5, color);
}

/// A small colour swatch + caption (for the R&D legend).
fn swatch(ui: &mut egui::Ui, color: Color32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
    ui.label(RichText::new(label).monospace().size(9.0).color(INK_FAINT));
    ui.add_space(6.0);
}

/// A bordered stat tile: a faint caption above a big mono value, with a left accent
/// (for the Vehicle Designer's derived-performance readout).
fn stat_tile(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(132.0, 46.0), egui::Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 0.0, INSET);
    p.rect_stroke(rect, 0.0, Stroke::new(1.0, LINE));
    p.rect_filled(
        Rect::from_min_size(rect.left_top(), vec2(2.5, rect.height())),
        0.0,
        accent,
    );
    p.text(
        rect.left_top() + vec2(9.0, 7.0),
        egui::Align2::LEFT_TOP,
        label.to_uppercase(),
        FontId::monospace(8.5),
        INK_FAINT,
    );
    p.text(
        rect.left_bottom() + vec2(9.0, -8.0),
        egui::Align2::LEFT_BOTTOM,
        value,
        FontId::monospace(14.0),
        INK,
    );
}

/// A key/value row with a coloured value (balances).
fn kv_row(ui: &mut egui::Ui, k: &str, val: &str, col: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(k).monospace().size(11.0).color(INK_FAINT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(val).monospace().size(12.0).color(col));
        });
    });
}

/// A labelled bar filled to `frac` ∈ [0,1] with a right-aligned value caption.
fn metric_bar(ui: &mut egui::Ui, label: &str, value_label: &str, frac: f32, col: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), egui::Sense::hover());
    let p = ui.painter_at(rect);
    p.text(
        pos2(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        label.to_uppercase(),
        FontId::monospace(9.0),
        INK_FAINT,
    );
    let bar = Rect::from_min_max(
        pos2(rect.left() + 96.0, rect.center().y - 5.0),
        pos2(rect.right() - 100.0, rect.center().y + 5.0),
    );
    p.rect_filled(bar, 0.0, INSET);
    p.rect_stroke(bar, 0.0, Stroke::new(1.0, LINE));
    let w = (bar.width() * frac.clamp(0.0, 1.0)).max(0.0);
    p.rect_filled(
        Rect::from_min_size(bar.left_top(), vec2(w, bar.height())),
        0.0,
        col,
    );
    p.text(
        pos2(rect.right(), rect.center().y),
        egui::Align2::RIGHT_CENTER,
        value_label,
        FontId::monospace(10.0),
        INK_DIM,
    );
}

/// A generation-vs-demand balance bar: the demand fill, the generation marker (blue
/// tick), green when the margin is positive, red when demand exceeds generation.
fn bar_balance(ui: &mut egui::Ui, label: &str, generation: f64, demand: f64, margin: f64) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), egui::Sense::hover());
    let p = ui.painter_at(rect);
    p.text(
        pos2(rect.left(), rect.center().y),
        egui::Align2::LEFT_CENTER,
        label.to_uppercase(),
        FontId::monospace(9.0),
        INK_FAINT,
    );
    let bar = Rect::from_min_max(
        pos2(rect.left() + 96.0, rect.center().y - 6.0),
        pos2(rect.right() - 110.0, rect.center().y + 6.0),
    );
    p.rect_filled(bar, 0.0, INSET);
    p.rect_stroke(bar, 0.0, Stroke::new(1.0, LINE));
    let cap = generation.max(demand).max(1.0);
    let ok = margin >= 0.0;
    let dw = (bar.width() as f64 * (demand / cap)) as f32;
    p.rect_filled(
        Rect::from_min_size(bar.left_top(), vec2(dw.max(0.0), bar.height())),
        0.0,
        if ok {
            SIGNAL
        } else {
            Color32::from_rgb(0xff, 0x5e, 0x5e)
        },
    );
    let gx = bar.left() + (bar.width() as f64 * (generation / cap)) as f32;
    p.line_segment(
        [pos2(gx, bar.top() - 2.0), pos2(gx, bar.bottom() + 2.0)],
        Stroke::new(1.5, PLOT),
    );
    let txt = format!("{:.1}/{:.1} kW", demand / 1.0e3, generation / 1.0e3);
    p.text(
        pos2(rect.right(), rect.center().y),
        egui::Align2::RIGHT_CENTER,
        txt,
        FontId::monospace(10.0),
        if ok {
            INK_DIM
        } else {
            Color32::from_rgb(0xff, 0x5e, 0x5e)
        },
    );
}

/// The Δv ladder (FR-VD bespoke widget): required-vs-delivered bars on a shared scale
/// plus the per-stage breakdown and a verdict.
fn dv_ladder(ui: &mut egui::Ui, required: f64, delivered: f64, stages: &[f64]) {
    let cap = required.max(delivered).max(1.0);
    metric_bar(
        ui,
        "required",
        &format!("{required:.0} m/s"),
        (required / cap) as f32,
        WARN,
    );
    let dcol = if delivered >= required { SIGNAL } else { WARN };
    metric_bar(
        ui,
        "delivered",
        &format!("{delivered:.0} m/s"),
        (delivered / cap) as f32,
        dcol,
    );
    if stages.len() > 1 {
        let segs: Vec<String> = stages
            .iter()
            .enumerate()
            .map(|(i, dv)| format!("S{i}:{dv:.0}"))
            .collect();
        ui.label(
            RichText::new(format!("stages  {}", segs.join("  ·  ")))
                .monospace()
                .size(9.5)
                .color(INK_FAINT),
        );
    }
    let (vc, vt) = if delivered >= required {
        (SIGNAL, "✓ meets mission Δv requirement")
    } else {
        (
            WARN,
            "below mission Δv requirement — restage or add propellant",
        )
    };
    ui.label(RichText::new(vt).monospace().size(10.0).color(vc));
}

/// Era accent colour for the milestone race board.
fn era_color(era: &str) -> Color32 {
    match era {
        "foothold" => PLOT,
        "cislunar" => SIGNAL,
        "frontier" => WARN,
        "endgame" => Color32::from_rgb(0xb9, 0x8b, 0xff),
        _ => INK_DIM,
    }
}

/// One milestone row: a weight badge, the description, and the world-first claim state.
fn milestone_row(ui: &mut egui::Ui, m: &MilestoneRow) {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), 22.0), egui::Sense::hover());
    let p = ui.painter_at(rect);
    if resp.hovered() {
        p.rect_filled(rect, 0.0, PANEL2);
    }
    let cy = rect.center().y;
    let claimed = m.claimed_by.is_some();
    // Weight badge (significance / prestige on claim).
    let badge = Rect::from_min_size(pos2(rect.left() + 2.0, cy - 8.0), vec2(46.0, 16.0));
    p.rect_filled(badge, 0.0, INSET);
    p.rect_stroke(badge, 0.0, Stroke::new(1.0, LINE));
    p.text(
        badge.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:.0}", m.weight),
        FontId::monospace(9.5),
        era_color(&m.era),
    );
    // Description.
    p.text(
        pos2(rect.left() + 56.0, cy),
        egui::Align2::LEFT_CENTER,
        trunc(&m.description, 66),
        FontId::monospace(10.5),
        if claimed { INK_FAINT } else { INK },
    );
    // Claim state.
    let (cc, ct) = match m.claimed_by {
        Some(f) => (SIGNAL, format!("✓ F{f}")),
        None => (INK_FAINT, "UNCLAIMED".to_string()),
    };
    p.text(
        pos2(rect.right() - 4.0, cy),
        egui::Align2::RIGHT_CENTER,
        ct,
        FontId::monospace(9.5),
        cc,
    );
}

/// Truncate a string to `n` characters with an ellipsis (table cells).
fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let t: String = s.chars().take(n.saturating_sub(1)).collect();
        format!("{t}…")
    }
}

/// One Understanding-Level row: a left accent (basic/applied), the domain id + name,
/// and a 0–100 bar painted with the private-UL fill, the world-tide ghost marker and
/// the diminishing-returns knee. The figures are read from the research surface; this
/// only draws them. Returns the row response for click-to-pin.
fn ul_row(ui: &mut egui::Ui, d: &DomainView, selected: bool) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), 30.0), egui::Sense::click());
    let p = ui.painter_at(rect);
    if selected {
        p.rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(0x3e, 0xe0, 0xa0, 22),
        );
    } else if resp.hovered() {
        p.rect_filled(rect, 0.0, PANEL2);
    }
    // Left accent: basic = blue, applied = amber.
    let accent = if d.basic_science {
        Color32::from_rgb(0x5b, 0x9b, 0xd6)
    } else {
        Color32::from_rgb(0xe3, 0xc9, 0x8c)
    };
    p.rect_filled(
        Rect::from_min_size(rect.left_top(), vec2(2.5, rect.height())),
        0.0,
        accent,
    );

    let cy = rect.center().y;
    // Id + name (left column).
    p.text(
        pos2(rect.left() + 10.0, cy),
        egui::Align2::LEFT_CENTER,
        &d.id,
        FontId::monospace(10.5),
        if selected { SIGNAL } else { INK_DIM },
    );
    p.text(
        pos2(rect.left() + 44.0, cy),
        egui::Align2::LEFT_CENTER,
        &d.name,
        FontId::monospace(11.0),
        INK,
    );
    if d.mission_injectable {
        p.text(
            pos2(rect.left() + 168.0, cy),
            egui::Align2::LEFT_CENTER,
            "⊙",
            FontId::monospace(10.0),
            SIGNAL,
        );
    }

    // Bar track.
    let bar = Rect::from_min_max(
        pos2(rect.left() + 188.0, cy - 5.0),
        pos2(rect.right() - 52.0, cy + 5.0),
    );
    if bar.width() > 12.0 {
        p.rect_filled(bar, 0.0, INSET);
        p.rect_stroke(bar, 0.0, Stroke::new(1.0, LINE));
        let frac = |v: f64| (v.clamp(0.0, 100.0) / 100.0) as f32;
        // World-tide ghost: faint fill up to the tide.
        let tide_w = bar.width() * frac(d.world_ul);
        if tide_w > 0.5 {
            p.rect_filled(
                Rect::from_min_size(bar.left_top(), vec2(tide_w, bar.height())),
                0.0,
                Color32::from_rgba_unmultiplied(0x5a, 0xa8, 0xff, 60),
            );
        }
        // Private-UL fill (the phosphor signal).
        let ul_w = bar.width() * frac(d.private_ul);
        if ul_w > 0.5 {
            p.rect_filled(
                Rect::from_min_size(bar.left_top(), vec2(ul_w, bar.height())),
                0.0,
                SIGNAL,
            );
        }
        // DR-knee marker.
        let kx = bar.left() + bar.width() * frac(d.dr_knee);
        p.line_segment(
            [pos2(kx, bar.top() - 2.0), pos2(kx, bar.bottom() + 2.0)],
            Stroke::new(1.0, WARN),
        );
    }
    // Effective-UL value (right column).
    p.text(
        pos2(rect.right() - 6.0, cy),
        egui::Align2::RIGHT_CENTER,
        format!("{:.0}", d.effective_ul),
        FontId::monospace(11.0),
        if d.effective_ul > 0.0 { INK } else { INK_FAINT },
    );
    resp
}

/// A tiny stylised look for the recognisable planets / major moons: `(colour, min
/// draw radius px, has a ring)`. Minor bodies (asteroids, tiny moons) return `None`
/// → a small grey dot. Names match the FA-03 catalogue ids (classical/Latin).
fn planet_style(name: &str) -> Option<(Color32, f32, bool)> {
    let c = Color32::from_rgb;
    Some(match name.to_ascii_lowercase().as_str() {
        "mercury" => (c(0x9a, 0x8d, 0x7a), 3.0, false),
        "venus" => (c(0xe6, 0xcd, 0x8a), 3.6, false),
        "terra" | "earth" => (c(0x5b, 0x9b, 0xd6), 3.6, false),
        "ares" | "mars" => (c(0xc8, 0x69, 0x4a), 3.4, false),
        "jupiter" => (c(0xcd, 0xa9, 0x82), 5.6, false),
        "saturn" => (c(0xe3, 0xc9, 0x8c), 4.8, true),
        "uranus" => (c(0x9f, 0xd6, 0xe0), 4.2, false),
        "neptune" => (c(0x4a, 0x6c, 0xd6), 4.2, false),
        "pluto" => (c(0xc8, 0xb8, 0xa0), 2.6, false),
        "luna" | "moon" => (c(0xb8, 0xbc, 0xc4), 2.6, false),
        "io" => (c(0xe8, 0xd8, 0x78), 2.2, false),
        "europa" => (c(0xd8, 0xe0, 0xe8), 2.2, false),
        "ganymede" => (c(0xa6, 0x98, 0x84), 2.5, false),
        "callisto" => (c(0x6a, 0x6a, 0x72), 2.5, false),
        "titan" => (c(0xd8, 0xa8, 0x78), 2.7, false),
        "enceladus" => (c(0xe8, 0xf0, 0xf8), 2.0, false),
        "phobos" | "deimos" => (c(0x7a, 0x70, 0x66), 1.8, false),
        _ => return None,
    })
}

/// Draw a thin ellipse outline (for Saturn's ring) as a closed polyline.
fn draw_ellipse(p: &egui::Painter, center: egui::Pos2, rx: f32, ry: f32, stroke: Stroke) {
    const N: usize = 28;
    let pts: Vec<egui::Pos2> = (0..=N)
        .map(|k| {
            let a = k as f64 / N as f64 * std::f64::consts::TAU;
            pos2(
                center.x + rx * libm::cos(a) as f32,
                center.y + ry * libm::sin(a) as f32,
            )
        })
        .collect();
    p.add(egui::Shape::line(pts, stroke));
}

fn warp_btn(text: &str, on: bool) -> egui::Button<'static> {
    let rt = RichText::new(text.to_string())
        .monospace()
        .size(11.0)
        .color(if on { VOID } else { INK_DIM });
    let fill = if on { SIGNAL } else { INSET };
    egui::Button::new(rt)
        .fill(fill)
        .stroke(Stroke::new(1.0, LINE))
        .rounding(egui::Rounding::ZERO)
}

fn class_color(class: &str) -> Color32 {
    if class.contains("astrobiology") || class.contains("discovery") {
        Color32::from_rgb(0xb9, 0x8b, 0xff)
    } else if class.contains("milestone") || class.contains("grand-goal") {
        Color32::from_rgb(0xec, 0xe2, 0xc2)
    } else if class.contains("contamination") || class.contains("loss") || class.contains("failure")
    {
        Color32::from_rgb(0xff, 0x5e, 0x5e)
    } else if class.contains("funding") || class.contains("milestone-claimed") {
        SIGNAL
    } else {
        PLOT
    }
}

fn sec(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.label(lbl(title));
    ui.separator();
}

fn kv(ui: &mut egui::Ui, k: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(k).monospace().size(11.0).color(INK_FAINT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(val).monospace().size(12.0).color(INK));
        });
    });
}

fn main() -> eframe::Result<()> {
    let host = match UiHost::new_game(&repo_root(), 7) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("failed to start the simulation core: {e}");
            std::process::exit(1);
        }
    };
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1380.0, 860.0])
            .with_min_inner_size([1100.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sojourn — Mission Console",
        opts,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(App::new(host)))
        }),
    )
}
