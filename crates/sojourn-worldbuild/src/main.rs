//! Offline data-build tool (FR-WORLD-106). Reads local source snapshots under
//! `<sources>/` (developer-fetched SBDB/MPC JSON + transcribed fact-sheet tables,
//! in human-friendly natural units) and emits the committed, schema-valid astro
//! catalogue RON under `<out>/` with per-entry provenance and a snapshot date.
//! **Network-free**: fetching the raw snapshots is a separate documented step
//! (`data/world/sources/README.md`); this tool only transforms local inputs.
//!
//! Each `*.json` source file declares an `out` basename and a list of bodies in
//! natural units (AU, degrees); the tool epoch-normalises, converts to SI/radians
//! (`argp = ϖ − Ω`, `M = L − ϖ` when the long-form is given) and writes
//! `<out>/<out>.ron` parseable by `sojourn_astro::Catalog`.
//!
//! Usage: `sojourn-worldbuild <sources-dir> <out-dir>`

use serde::{Deserialize, Serialize};
use std::path::Path;

const AU_M: f64 = 1.495_978_707e11;
const DEG: f64 = core::f64::consts::PI / 180.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} <sources-dir> <out-dir>", args[0]);
        std::process::exit(2);
    }
    match build(&args[1], &args[2]) {
        Ok(report) => println!("{report}"),
        Err(e) => {
            eprintln!("worldbuild: {e}");
            std::process::exit(1);
        }
    }
}

// ---- source schema (natural units) ----------------------------------------

#[derive(Debug, Deserialize)]
struct SourceFile {
    out: String,
    #[serde(default)]
    epoch_ns: i64,
    provenance: String,
    snapshot_date: String,
    bodies: Vec<SrcBody>,
}

#[derive(Debug, Deserialize)]
struct SrcBody {
    id: u32,
    name_id: String,
    mu: f64,
    radius: f64,
    source: String,
    #[serde(default)]
    gravitating: bool,
    #[serde(default)]
    divertible: bool,
    j2: Option<f64>,
    rotation_period_s: Option<f64>,
    atmosphere: Option<Atmo>,
    parent: Option<u32>,
    a_au: Option<f64>,
    a_m: Option<f64>,
    e: Option<f64>,
    i_deg: Option<f64>,
    node_deg: Option<f64>,
    argp_deg: Option<f64>,
    ma_deg: Option<f64>,
    peri_long_deg: Option<f64>,
    mean_long_deg: Option<f64>,
}

// ---- output schema (mirrors sojourn_astro::BodyDef exactly) ----------------

#[derive(Debug, Serialize)]
struct Atmo {
    interface_alt_m: f64,
    drag_alt_m: f64,
    rho0: f64,
    scale_height_m: f64,
    source: String,
}
// Atmo is deserialized from source and serialized to output unchanged.
impl<'de> Deserialize<'de> for Atmo {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct A {
            interface_alt_m: f64,
            drag_alt_m: f64,
            rho0: f64,
            scale_height_m: f64,
            source: String,
        }
        let a = A::deserialize(d)?;
        Ok(Atmo {
            interface_alt_m: a.interface_alt_m,
            drag_alt_m: a.drag_alt_m,
            rho0: a.rho0,
            scale_height_m: a.scale_height_m,
            source: a.source,
        })
    }
}

#[derive(Debug, Serialize)]
struct OutElements {
    sma: f64,
    ecc: f64,
    inc: f64,
    raan: f64,
    argp: f64,
    mean_anomaly_at_epoch: f64,
    epoch_ns: i64,
}

#[derive(Debug, Serialize)]
struct OutBody {
    id: u32,
    name_id: String,
    mu: f64,
    radius: f64,
    j2: Option<f64>,
    rotation_period_s: Option<f64>,
    atmosphere: Option<Atmo>,
    gravitating: bool,
    divertible: bool,
    parent: Option<u32>,
    elements: Option<OutElements>,
    source: String,
}

#[derive(Debug, Serialize)]
struct OutFile {
    bodies: Vec<OutBody>,
}

fn build(sources_dir: &str, out_dir: &str) -> Result<String, String> {
    let sdir = Path::new(sources_dir);
    let odir = Path::new(out_dir);
    std::fs::create_dir_all(odir).map_err(|e| format!("creating {}: {e}", odir.display()))?;

    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(sdir)
        .map_err(|e| format!("reading {}: {e}", sdir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("no .json snapshots in {}", sdir.display()));
    }

    let mut total = 0usize;
    let mut written = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("reading {}: {e}", f.display()))?;
        let src: SourceFile = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", f.display()))?;
        let mut bodies = Vec::new();
        for b in src.bodies {
            if b.source.trim().is_empty() {
                return Err(format!("{}: body {} has empty source", f.display(), b.id));
            }
            let elements = match b.parent {
                None => None,
                Some(_) => Some(convert_elements(&b, src.epoch_ns, f)?),
            };
            let source = format!("{} [snapshot {}: {}]", b.source, src.snapshot_date, src.provenance);
            bodies.push(OutBody {
                id: b.id,
                name_id: b.name_id,
                mu: b.mu,
                radius: b.radius,
                j2: b.j2,
                rotation_period_s: b.rotation_period_s,
                atmosphere: b.atmosphere,
                gravitating: b.gravitating,
                divertible: b.divertible,
                parent: b.parent,
                elements,
                source,
            });
        }
        total += bodies.len();
        let cfg = ron::ser::PrettyConfig::new().struct_names(false);
        let body_text = ron::ser::to_string_pretty(&OutFile { bodies }, cfg)
            .map_err(|e| format!("serialising {}: {e}", src.out))?;
        let header = format!(
            "// GENERATED by sojourn-worldbuild from {} (snapshot {}). Do not edit by hand.\n// {}\n",
            f.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
            src.snapshot_date,
            src.provenance
        );
        let out_path = odir.join(format!("{}.ron", src.out));
        std::fs::write(&out_path, format!("{header}{body_text}\n"))
            .map_err(|e| format!("writing {}: {e}", out_path.display()))?;
        written.push(format!("{} ({} bodies)", out_path.display(), total));
    }
    Ok(format!(
        "worldbuild: {} bodies across {} files →\n  {}",
        total,
        files.len(),
        written.join("\n  ")
    ))
}

fn convert_elements(b: &SrcBody, epoch_ns: i64, f: &Path) -> Result<OutElements, String> {
    let need = |o: Option<f64>, name: &str| {
        o.ok_or_else(|| format!("{}: body {} missing orbital field '{name}'", f.display(), b.id))
    };
    let sma = match (b.a_m, b.a_au) {
        (Some(m), _) => m,
        (None, Some(au)) => au * AU_M,
        (None, None) => return Err(format!("{}: body {} missing a_m/a_au", f.display(), b.id)),
    };
    let ecc = need(b.e, "e")?;
    let inc = need(b.i_deg, "i_deg")? * DEG;
    let raan = need(b.node_deg, "node_deg")? * DEG;
    // Long-form (planet tables) gives ϖ (peri longitude) and L (mean longitude);
    // direct form (SBDB) gives ω and M.
    let (argp, ma) = match (b.peri_long_deg, b.mean_long_deg) {
        (Some(peri_long), Some(mean_long)) => {
            let node = need(b.node_deg, "node_deg")?;
            (
                (peri_long - node).rem_euclid(360.0) * DEG,
                (mean_long - peri_long).rem_euclid(360.0) * DEG,
            )
        }
        _ => (
            need(b.argp_deg, "argp_deg")?.rem_euclid(360.0) * DEG,
            need(b.ma_deg, "ma_deg")?.rem_euclid(360.0) * DEG,
        ),
    };
    Ok(OutElements {
        sma,
        ecc,
        inc,
        raan,
        argp,
        mean_anomaly_at_epoch: ma,
        epoch_ns,
    })
}
