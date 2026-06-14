//! Screen view-model tests over **stub snapshots** — no renderer (FR-UI-1506).
//! Covers US1 (shell/map), US4 (planner), US7 (operations/virtualisation), US8
//! (economy ledger), US9 (bases), US5 (R&D) and US12 (the astrobiology honesty guard).

use sojourn_ui::host::{EventView, Status};
use sojourn_ui::viewmodel as vm;
use sojourn_ui::widgets_data::{
    BaseSchematic, DvLadder, EvidenceMeter, PorkchopField, ResourceLedger, TrlLadder,
    UnderstandingBars,
};

#[test]
fn shell_formats_the_date_and_ticker() {
    let s = Status {
        tick: 86_400,
        date: (2031, 7, 20),
        total_events: 3,
        pending_interrupts: 1,
    };
    let ev = vec![
        EventView {
            id: 1,
            tick: 10,
            class: "launch-failure".into(),
        },
        EventView {
            id: 2,
            tick: 20,
            class: "milestone-claimed".into(),
        },
    ];
    let sv = vm::shell(&s, &ev);
    assert_eq!(sv.date, "2031-07-20");
    assert_eq!(sv.pending_interrupts, 1);
    assert_eq!(sv.ticker.first().unwrap(), "milestone-claimed"); // newest first
}

#[test]
fn map_culls_sub_pixel_bodies_for_level_of_detail() {
    let bodies = vec![
        vm::BodyInput {
            id: 4,
            name: "Mars".into(),
            pos: (2.0e11, 0.0),
            apparent_px: 8.0,
        },
        vm::BodyInput {
            id: 999,
            name: "tiny asteroid".into(),
            pos: (3.0e11, 0.0),
            apparent_px: 0.2,
        },
    ];
    let m = vm::map(&bodies, 1.0);
    assert_eq!(m.bodies.len(), 1, "the sub-pixel body is culled");
    assert_eq!(m.culled, 1);
    assert_eq!(m.bodies[0].name, "Mars");
}

#[test]
fn operations_table_virtualises_to_the_viewport() {
    let rows: Vec<vm::CraftRow> = (0..5000)
        .map(|i| vm::CraftRow {
            id: i,
            name: format!("craft-{i:05}"),
            location: "LEO".into(),
            fuel: 0.5,
            health: 1.0,
            task: "idle".into(),
        })
        .collect();
    let o = vm::operations(&rows, 100, 30);
    assert_eq!(o.total, 5000, "total reflects all rows");
    assert_eq!(
        o.visible.len(),
        30,
        "only the viewport rows are materialised"
    );
    assert_eq!(
        o.visible[0].name, "craft-00100",
        "sorted + offset to the viewport"
    );
}

#[test]
fn planner_checks_required_against_available_dv() {
    let porkchop = PorkchopField {
        departures: vec![0.0, 1.0],
        arrivals: vec![200.0, 210.0],
        dv: vec![3800.0, 3700.0, 3900.0, 4000.0],
        optimum: Some((1, 0)),
    };
    let feasible = vm::planner(
        porkchop.clone(),
        DvLadder {
            segments: vec![("TMI".into(), 3600.0)],
            available: 4000.0,
        },
    );
    assert!(feasible.feasible);
    assert_eq!(feasible.required, "3.6 km/s");
    let infeasible = vm::planner(
        porkchop,
        DvLadder {
            segments: vec![("TMI".into(), 4500.0)],
            available: 4000.0,
        },
    );
    assert!(!infeasible.feasible, "required Δv > available ⇒ infeasible");
}

#[test]
fn economy_groups_inventory_by_location() {
    let ledger = ResourceLedger {
        rows: vec![
            ("Shackleton".into(), "water".into(), 5000.0),
            ("Shackleton".into(), "LOX".into(), 3000.0),
            ("LEO depot".into(), "LH2".into(), 1000.0),
        ],
    };
    let e = vm::economy(ledger);
    assert_eq!(e.subtotals.len(), 2, "two Δv-addressed locations");
    let (loc, mass) = e.subtotals.iter().find(|(l, _)| l == "Shackleton").unwrap();
    assert_eq!(loc, "Shackleton");
    assert_eq!(mass, "8 t"); // 8000 kg
}

#[test]
fn bases_gauges_flag_below_threshold() {
    let sch = BaseSchematic {
        modules: vec!["hab".into(), "fission".into()],
        gauges: vec![
            ("power margin".into(), 1.2, 1.0),
            ("sustainability".into(), 0.6, 0.8),
        ],
    };
    let b = vm::bases(sch);
    assert!(
        b.gauges
            .iter()
            .find(|(n, _, _)| n == "power margin")
            .unwrap()
            .2,
        "power ok"
    );
    assert!(
        !b.gauges
            .iter()
            .find(|(n, _, _)| n == "sustainability")
            .unwrap()
            .2,
        "sustainability below threshold"
    );
}

#[test]
fn research_surfaces_dead_end_warnings() {
    let bars = UnderstandingBars {
        domains: vec![("Propulsion".into(), 0.6, 0.7, 0.3)],
    };
    let r = vm::research(
        bars,
        vec![
            TrlLadder {
                current: 5,
                test_progress: 0.4,
                risk: 0.3,
                dead_end: false,
            },
            TrlLadder {
                current: 3,
                test_progress: 0.1,
                risk: 0.9,
                dead_end: true,
            },
        ],
    );
    assert!(r.has_dead_end, "a dead-end program raises the warning");
}

#[test]
fn astrobiology_meter_is_honest() {
    // An open candidate: high consensus but NOT conclusive ⇒ never reported positive.
    let open = vm::astrobiology(&vm::CandidateInput {
        name: "Enceladus".into(),
        consensus: 0.85,
        posteriors: vec![0.9, 0.4],
        conclusive: None,
        disagreement: true,
    });
    assert_eq!(
        open.meter.conclusive, None,
        "no conclusion the core has not set"
    );
    assert!(open.meter.disagreement, "public disagreement is shown");

    // The core marked it conclusive-positive ⇒ the UI reflects exactly that.
    let resolved = vm::astrobiology(&vm::CandidateInput {
        name: "Enceladus".into(),
        consensus: 0.95,
        posteriors: vec![0.96, 0.95],
        conclusive: Some(true),
        disagreement: false,
    });
    assert_eq!(resolved.meter.conclusive, Some(true));
    // (Structural honesty: there is NO ground-truth field on CandidateInput — the UI
    //  cannot access or fabricate one; EvidenceMeter only ever mirrors `conclusive`.)
    let _ = EvidenceMeter {
        candidate: "x".into(),
        consensus: 0.0,
        posteriors: vec![],
        conclusive: None,
        disagreement: false,
    };
}
