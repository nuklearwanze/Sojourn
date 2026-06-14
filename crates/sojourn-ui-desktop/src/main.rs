//! Sojourn desktop renderer (FA-10) — the thin egui/eframe shell over the
//! `sojourn-ui` view-model. It owns no game logic and no authoritative state: it hosts
//! the core via `UiHost`, pulls the view-model (events + throttled snapshots) and
//! paints it; every action routes back as a journalled command.

use eframe::egui;
use sojourn_ui::host::UiHost;
use sojourn_ui::viewmodel as vm;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // The binary runs from the repo root (cargo run -p sojourn-ui-desktop).
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Which screen is active (left-nav state — UI-ephemeral, never in the save).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Map,
    Operations,
    Astrobiology,
    Alerts,
}

struct App {
    host: UiHost,
    screen: Screen,
    warp_days_per_frame: u64,
    running: bool,
}

impl App {
    fn new(host: UiHost) -> App {
        App {
            host,
            screen: Screen::Map,
            warp_days_per_frame: 0,
            running: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Time-warp: advance the core off the renderer's frame (never blocks it).
        if self.running && self.warp_days_per_frame > 0 {
            let _ = self.host.advance(self.warp_days_per_frame * 86_400);
            ctx.request_repaint();
        }

        let status = self.host.status();
        let events = self.host.recent_events(32);
        let shell = vm::shell(&status, &events);

        // Top bar: clock + time-warp.
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("📅 {}", shell.date));
                ui.separator();
                if ui
                    .button(if self.running { "⏸ pause" } else { "▶ run" })
                    .clicked()
                {
                    self.running = !self.running;
                }
                ui.label("warp:");
                for d in [0u64, 1, 7, 30] {
                    if ui
                        .selectable_label(self.warp_days_per_frame == d, format!("{d}d"))
                        .clicked()
                    {
                        self.warp_days_per_frame = d;
                    }
                }
                ui.separator();
                if shell.pending_interrupts > 0 {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("⚠ {} interrupt(s)", shell.pending_interrupts),
                    );
                }
            });
        });

        // Left nav.
        egui::SidePanel::left("nav").show(ctx, |ui| {
            ui.heading("Sojourn");
            for (label, s) in [
                ("System Map", Screen::Map),
                ("Operations", Screen::Operations),
                ("Astrobiology", Screen::Astrobiology),
                ("Alerts", Screen::Alerts),
            ] {
                ui.selectable_value(&mut self.screen, s, label);
            }
        });

        // Bottom event ticker.
        egui::TopBottomPanel::bottom("ticker").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for c in &shell.ticker {
                    ui.label(format!("• {c}"));
                }
            });
        });

        // Central work area: the active screen.
        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Map => {
                ui.heading("System Map");
                ui.label(format!(
                    "tick {} — bodies render from the astro/world surfaces (LOD-culled).",
                    status.tick
                ));
                ui.label("Zoom/pan + layer toggles paint the MapVM (custom painter).");
            }
            Screen::Operations => {
                ui.heading("Operations / Fleet");
                ui.label("Virtualised fleet table (egui_extras TableBuilder over OperationsVM).");
            }
            Screen::Astrobiology => {
                ui.heading("Science Returns & Astrobiology");
                ui.label(
                    "Evidence meter — probabilistic, per candidate; never a binary 'life found'.",
                );
            }
            Screen::Alerts => {
                ui.heading("Alerts / Event Log");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for e in &events {
                        ui.label(format!("@{:>10}  {}", e.tick, e.class));
                    }
                });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let host = match UiHost::new_game(&repo_root(), 7) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("failed to start the simulation core: {e}");
            std::process::exit(1);
        }
    };
    let opts = eframe::NativeOptions::default();
    eframe::run_native(
        "Sojourn",
        opts,
        Box::new(|_cc| Ok(Box::new(App::new(host)))),
    )
}
