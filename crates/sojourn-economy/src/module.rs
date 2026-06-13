//! The `SimModule` implementation: the economy slice, the command surface via
//! `ModulePayload`, the daily resource-flow step + monthly market sub-tick (R7),
//! seeded streams, events, and serde with econ-data pinning. Depends only on
//! `sojourn-core`; cross-slice physics arrives inside command payloads as composed
//! values (R1/R2).

use crate::commodity::{Commodities, StrategicDefs};
use crate::contract::{Contract, ContractState, Deliverable, License, Partnership};
use crate::cost::CostParams;
use crate::facility::{Facility, FacilityDefs};
use crate::funding::{FundingDefs, FundingProfile, FundingState};
use crate::ids::*;
use crate::inputs::{EdgePrice, GradeBelief};
use crate::isru::{self, IsruDefs, IsruPlant};
use crate::ledger::{Cause, Currency, Ledger, Leg, StockKey, Transaction};
use crate::market::{LaunchMarketDefs, LaunchMarketState, MarketParams};
use crate::network::{EdgeKind, NetworkDefs};
use crate::project::{Project, ProjectState};
use crate::shipment::{Depot, Shipment, ShipmentState, Vehicle};
use serde::{Deserialize, Serialize};
use sojourn_core::{
    Command, CommandOutcome, CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;
use std::path::Path;

const TICKS_PER_DAY: u64 = 86_400;
const MARKET_TICK_DAYS: u64 = 30;

/// A journalled economy command (routed via `Command::ModulePayload`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EconomyCommand {
    /// Set an opening currency balance (scenario/setup).
    SetBalance {
        /// Faction.
        faction: u32,
        /// Currency.
        currency: Currency,
        /// Value.
        amount: f64,
    },
    /// Set an opening stock at a location (scenario/setup).
    SetStock {
        /// Commodity id.
        commodity: String,
        /// Location id.
        location: String,
        /// Amount.
        amount: f64,
    },
    /// Apply an arbitrary conserving transaction (credit/debit legs).
    Transact {
        /// Faction the transaction is attributed to.
        faction: u32,
        /// The legs.
        legs: Vec<Leg>,
        /// Cause.
        cause: Cause,
    },
    /// Dispatch a shipment over an edge (price supplied as a composed value).
    DispatchShipment {
        /// Faction.
        faction: u32,
        /// Edge id.
        edge: String,
        /// Cargo (commodity id, amount).
        cargo: Vec<(String, f64)>,
        /// The assigned vehicle.
        vehicle: Vehicle,
        /// The composed edge price (from the FA-02 planner).
        edge_price: EdgePrice,
    },
    /// Initialise a faction's funding state from its data profile.
    RegisterFunding {
        /// Faction.
        faction: u32,
        /// Tick the first fiscal period ends (agency) — ignored for private.
        period_end_tick: u64,
    },
    /// Apply an agency appropriation (opaque political modifier).
    ApplyAppropriation {
        /// Faction.
        faction: u32,
        /// Baseline amount.
        amount: f64,
        /// Directed/earmarked amount.
        directed: f64,
        /// Political modifier (±fraction).
        political_modifier: f64,
    },
    /// Inject private financing.
    InjectFinancing {
        /// Faction.
        faction: u32,
        /// Amount (Funds).
        amount: f64,
    },
    /// Buy launch capacity on the market (Funds → mass-to-orbit).
    BuyLaunch {
        /// Faction.
        faction: u32,
        /// Orbit class.
        orbit_class: String,
        /// Mass (kg).
        mass_kg: f64,
    },
    /// Set the world launch-capacity index (test/world-event hook).
    SetMarketCapacity {
        /// New index.
        index: f64,
    },
    /// Commission a facility (capex; sizes capacity/ops pool).
    CommissionFacility {
        /// Faction.
        faction: u32,
        /// Template id.
        template: String,
    },
    /// Upgrade a facility one level.
    UpgradeFacility {
        /// Faction.
        faction: u32,
        /// Facility id.
        facility: u32,
    },
    /// Set the active-craft load on a faction's ops pool (test/world hook).
    SetOpsLoad {
        /// Faction.
        faction: u32,
        /// Active-craft demand on the pool.
        used: f64,
    },
    /// Site an ISRU plant at a node.
    SiteIsruPlant {
        /// Faction.
        faction: u32,
        /// Node id.
        node: String,
        /// Process id.
        process: String,
    },
    /// Operate an ISRU plant for a number of days (ramps scale; credits output).
    OperateIsru {
        /// Faction.
        faction: u32,
        /// Plant id.
        plant: u32,
        /// Days to operate.
        days: u64,
        /// Believed grade at the node (composed from FA-03).
        grade: GradeBelief,
        /// Available/demanded power factor ∈ [0,1].
        power_factor: f64,
    },
    /// Post a service-contract RFP.
    PostRfp {
        /// Issuer.
        issuer: u32,
        /// Destination node.
        node: String,
        /// Commodity to deliver.
        commodity: String,
        /// Amount.
        amount: f64,
        /// Reward (Funds).
        reward_funds: f64,
        /// Heritage on success.
        heritage: f64,
        /// Penalty on failure.
        penalty: f64,
    },
    /// Bid on a contract.
    BidContract {
        /// Bidder.
        faction: u32,
        /// Contract id.
        contract: u32,
    },
    /// Award a contract to a winner.
    AwardContract {
        /// Contract id.
        contract: u32,
        /// Winner.
        winner: u32,
    },
    /// Report fulfilment/failure of a contract.
    ReportContract {
        /// Contract id.
        contract: u32,
        /// Success?
        success: bool,
    },
    /// Form a partnership.
    FormPartnership {
        /// Faction A.
        a: u32,
        /// Faction B.
        b: u32,
        /// Initial trust ∈ [0,1].
        trust: f64,
    },
    /// Renege on a partnership (trust decays).
    RenegePartnership {
        /// Faction A.
        a: u32,
        /// Faction B.
        b: u32,
    },
    /// Open a delivery-driven project (the Slice 7 seam).
    OpenProject {
        /// Faction.
        faction: u32,
        /// Target node.
        node: String,
        /// Required (commodity, amount).
        required: Vec<(String, f64)>,
        /// Required crew-time (hours).
        crew_time_req: f64,
    },
    /// Deliver a commodity to a project.
    DeliverToProject {
        /// Project id.
        project: u32,
        /// Commodity.
        commodity: String,
        /// Amount.
        amount: f64,
    },
}

/// Encode an [`EconomyCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn economy_payload(cmd: &EconomyCommand) -> Command {
    let kind = match cmd {
        EconomyCommand::SetBalance { .. } => "set-balance",
        EconomyCommand::SetStock { .. } => "set-stock",
        EconomyCommand::Transact { .. } => "transact",
        EconomyCommand::DispatchShipment { .. } => "dispatch",
        EconomyCommand::RegisterFunding { .. } => "register-funding",
        EconomyCommand::ApplyAppropriation { .. } => "appropriation",
        EconomyCommand::InjectFinancing { .. } => "financing",
        EconomyCommand::BuyLaunch { .. } => "buy-launch",
        EconomyCommand::SetMarketCapacity { .. } => "set-market-capacity",
        EconomyCommand::CommissionFacility { .. } => "commission-facility",
        EconomyCommand::UpgradeFacility { .. } => "upgrade-facility",
        EconomyCommand::SetOpsLoad { .. } => "set-ops-load",
        EconomyCommand::SiteIsruPlant { .. } => "site-isru",
        EconomyCommand::OperateIsru { .. } => "operate-isru",
        EconomyCommand::PostRfp { .. } => "post-rfp",
        EconomyCommand::BidContract { .. } => "bid",
        EconomyCommand::AwardContract { .. } => "award",
        EconomyCommand::ReportContract { .. } => "report-contract",
        EconomyCommand::FormPartnership { .. } => "form-partnership",
        EconomyCommand::RenegePartnership { .. } => "renege",
        EconomyCommand::OpenProject { .. } => "open-project",
        EconomyCommand::DeliverToProject { .. } => "deliver-project",
    };
    Command::ModulePayload {
        module: "economy".into(),
        kind: kind.into(),
        payload: postcard::to_allocvec(cmd).expect("economy command encodes"),
    }
}

/// The economy slice — all owned state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomySlice {
    /// The ledger (currencies + location-addressed stocks + journal).
    pub ledger: Ledger,
    /// Per-faction funding state.
    pub funding: BTreeMap<FactionId, FundingState>,
    /// Active + terminal shipments.
    pub shipments: BTreeMap<ShipmentId, Shipment>,
    /// Depots by node.
    pub depots: BTreeMap<String, Depot>,
    /// ISRU plants.
    pub plants: BTreeMap<PlantId, IsruPlant>,
    /// Launch-market state.
    pub market: LaunchMarketState,
    /// Contracts.
    pub contracts: BTreeMap<ContractId, Contract>,
    /// Partnerships keyed by ordered pair.
    pub partnerships: BTreeMap<(FactionId, FactionId), Partnership>,
    /// Licences.
    pub licenses: Vec<License>,
    /// Facilities.
    pub facilities: BTreeMap<FacilityId, Facility>,
    /// Per-faction ops pool.
    pub ops: BTreeMap<FactionId, crate::ops::OpsPool>,
    /// Projects (the Slice 7 seam).
    pub projects: BTreeMap<ProjectId, Project>,
    /// Next-id counters.
    pub next_shipment: u32,
    /// Next plant id.
    pub next_plant: u32,
    /// Next contract id.
    pub next_contract: u32,
    /// Next facility id.
    pub next_facility: u32,
    /// Next project id.
    pub next_project: u32,
    /// Last stepped tick.
    pub last_tick: u64,
    /// Econ-data content hash (pin).
    pub data_hash: [u8; 32],
}

/// The economy module (immutable loaded data).
pub struct EconomyModule {
    /// Commodity taxonomy.
    pub commodities: Commodities,
    /// Strategic-material supplies.
    pub strategic: StrategicDefs,
    /// Transport graph.
    pub network: NetworkDefs,
    /// Funding profiles.
    pub funding_defs: FundingDefs,
    /// ISRU processes.
    pub isru: IsruDefs,
    /// Cost-uncertainty params.
    pub cost: CostParams,
    /// Launch-market params.
    pub launch_market: LaunchMarketDefs,
    /// Market/contract params + niche markets.
    pub markets: MarketParams,
    /// Facility templates.
    pub facility_defs: FacilityDefs,
    /// Content hash over all econ data.
    pub content_hash: [u8; 32],
}

fn normalize(t: &str) -> String {
    t.replace("\r\n", "\n")
}

impl EconomyModule {
    /// Load and validate all econ data from a directory (`data/econ`).
    pub fn load(dir: &Path) -> Result<EconomyModule, String> {
        let read =
            |n: &str| std::fs::read_to_string(dir.join(n)).map_err(|e| format!("reading {n}: {e}"));
        let commodities_txt = read("commodities.ron")?;
        let strategic_txt = read("strategic.ron")?;
        let network_txt = read("network.ron")?;
        let funding_txt = read("funding.ron")?;
        let isru_txt = read("isru.ron")?;
        let cost_txt = read("cost.ron")?;
        let launch_txt = read("launch_market.ron")?;
        let markets_txt = read("markets.ron")?;
        let facilities_txt = read("facilities.ron")?;

        let commodities = Commodities::from_source(&commodities_txt)?;
        let strategic = StrategicDefs::from_source(&strategic_txt)?;
        let network = NetworkDefs::from_source(&network_txt)?;
        let funding_defs = FundingDefs::from_source(&funding_txt)?;
        let isru = IsruDefs::from_source(&isru_txt)?;
        let cost: CostParams = ron::from_str(&cost_txt).map_err(|e| format!("cost.ron: {e}"))?;
        if cost.source.trim().is_empty() {
            return Err("cost.ron: empty source".into());
        }
        let launch_market = LaunchMarketDefs::from_source(&launch_txt)?;
        let markets = MarketParams::from_source(&markets_txt)?;
        let facility_defs = FacilityDefs::from_source(&facilities_txt)?;

        // Cross-file: strategic refs from commodities must resolve.
        for r in commodities.strategic_refs() {
            if !strategic.by_id.contains_key(r) {
                return Err(format!("commodity strategic ref '{r}' has no supply"));
            }
        }

        let canonical = [
            &commodities_txt,
            &strategic_txt,
            &network_txt,
            &funding_txt,
            &isru_txt,
            &cost_txt,
            &launch_txt,
            &markets_txt,
            &facilities_txt,
        ]
        .iter()
        .map(|t| normalize(t))
        .collect::<Vec<_>>()
        .join("\0");
        let content_hash = *blake3::hash(canonical.as_bytes()).as_bytes();

        Ok(EconomyModule {
            commodities,
            strategic,
            network,
            funding_defs,
            isru,
            cost,
            launch_market,
            markets,
            facility_defs,
            content_hash,
        })
    }

    fn slice_mut<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut EconomySlice {
        slice
            .as_any_mut()
            .downcast_mut::<EconomySlice>()
            .expect("economy slice type")
    }
}

/// A uniform `f64 ∈ [0,1)` from a kernel rng.
fn uniform(rng: &mut dyn rand_core::RngCore) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

impl SimModule for EconomyModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "economy".into(),
            owned_slice: "economy/slice".into(),
            publishes: vec!["economy/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "budget-cycle".into(),
                "contract-awarded".into(),
                "bankruptcy".into(),
                "agency-gutted".into(),
                "shipment-arrived".into(),
                "isru-online".into(),
                "market-shock".into(),
                "ops-oversubscribed".into(),
            ],
            subscribes: vec![],
            phase: 1,
            streams: vec![
                "economy/cost-overrun".into(),
                "economy/isru-yield".into(),
                "economy/market".into(),
                "economy/contracts".into(),
                "economy/ops-anomaly".into(),
            ],
            cadence_ticks: TICKS_PER_DAY,
            escalations: vec![],
        }
    }

    fn init(&self, _ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        Box::new(EconomySlice {
            ledger: Ledger::default(),
            funding: BTreeMap::new(),
            shipments: BTreeMap::new(),
            depots: BTreeMap::new(),
            plants: BTreeMap::new(),
            market: LaunchMarketState {
                world_capacity_index: self.launch_market.reference_capacity,
            },
            contracts: BTreeMap::new(),
            partnerships: BTreeMap::new(),
            licenses: Vec::new(),
            facilities: BTreeMap::new(),
            ops: BTreeMap::new(),
            projects: BTreeMap::new(),
            next_shipment: 0,
            next_plant: 0,
            next_contract: 0,
            next_facility: 0,
            next_project: 0,
            last_tick: 0,
            data_hash: self.content_hash,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        let now = ctx.tick().0;
        let s = self.slice_mut(slice);
        if now <= s.last_tick {
            return;
        }
        s.last_tick = now;
        let mut events: Vec<(String, BTreeMap<String, ViewValue>)> = Vec::new();

        // Advance shipments: open windows, deliver arrivals.
        let ids: Vec<ShipmentId> = s.shipments.keys().copied().collect();
        for id in ids {
            let (state, arrive, to_node, cargo) = {
                let sh = &s.shipments[&id];
                (
                    sh.state,
                    sh.arrive_tick,
                    sh.to_node.clone(),
                    sh.cargo.clone(),
                )
            };
            match state {
                ShipmentState::WaitingWindow => {
                    if now >= s.shipments[&id].depart_tick {
                        s.shipments.get_mut(&id).unwrap().state = ShipmentState::InTransit;
                    }
                }
                ShipmentState::InTransit if now >= arrive => {
                    for (cid, amt) in &cargo {
                        let key = StockKey {
                            commodity: cid.clone(),
                            location: LocationId(to_node.clone()),
                        };
                        *s.ledger.stocks.entry(key).or_insert(0.0) += amt;
                    }
                    s.shipments.get_mut(&id).unwrap().state = ShipmentState::Delivered;
                    events.push((
                        "shipment-arrived".into(),
                        BTreeMap::from([("shipment".to_string(), ViewValue::U64(u64::from(id.0)))]),
                    ));
                }
                _ => {}
            }
        }

        // Monthly market sub-tick: price fluctuation + private burn + fiscal periods.
        if now.is_multiple_of(MARKET_TICK_DAYS * TICKS_PER_DAY) {
            let wiggle = {
                let rng = ctx.rng("economy/market");
                (uniform(rng) * 2.0 - 1.0) * self.launch_market.fluctuation
            };
            let before = s.market.world_capacity_index;
            s.market.world_capacity_index = (before * (1.0 + wiggle)).max(1e-6);
            if wiggle.abs() >= self.markets.shock_threshold {
                events.push((
                    "market-shock".into(),
                    BTreeMap::from([("delta_bp".to_string(), ViewValue::F64(wiggle * 10_000.0))]),
                ));
            }
            // Private burn over the period.
            let factions: Vec<FactionId> = s.funding.keys().copied().collect();
            for f in factions {
                if let Some(FundingState::Private {
                    burn_rate,
                    bankrupt,
                }) = s.funding.get(&f).cloned()
                    && !bankrupt
                {
                    let burn = burn_rate * MARKET_TICK_DAYS as f64;
                    let cash = s.ledger.balance(f, Currency::Funds);
                    if cash < burn {
                        // Floor at zero, flag bankruptcy.
                        if cash > 0.0 {
                            let _ = s.ledger.apply(Transaction {
                                tick: now,
                                faction: f,
                                legs: vec![Leg::Currency(f, Currency::Funds, -cash)],
                                cause: Cause::Consume,
                            });
                        }
                        s.funding.insert(
                            f,
                            FundingState::Private {
                                burn_rate,
                                bankrupt: true,
                            },
                        );
                        events.push((
                            "bankruptcy".into(),
                            BTreeMap::from([(
                                "faction".to_string(),
                                ViewValue::U64(u64::from(f.0)),
                            )]),
                        ));
                    } else {
                        let _ = s.ledger.apply(Transaction {
                            tick: now,
                            faction: f,
                            legs: vec![Leg::Currency(f, Currency::Funds, -burn)],
                            cause: Cause::Consume,
                        });
                    }
                }
            }
        }

        for (class, payload) in events {
            ctx.emit(&class, payload);
        }
    }

    fn on_command(
        &self,
        slice: &mut dyn StateSlice,
        kind: &str,
        payload: &[u8],
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let cmd: EconomyCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed economy payload (kind '{kind}'): {e}"
                ));
            }
        };
        let now = ctx.tick().0;
        self.apply_command(slice, cmd, now, ctx)
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<EconomySlice>()
            .expect("economy slice type");
        vec![ViewSnapshot {
            id: "economy/status".into(),
            fields: BTreeMap::from([
                (
                    "shipments".to_string(),
                    ViewValue::U64(s.shipments.len() as u64),
                ),
                ("plants".to_string(), ViewValue::U64(s.plants.len() as u64)),
                (
                    "contracts".to_string(),
                    ViewValue::U64(s.contracts.len() as u64),
                ),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<EconomySlice>()
            .expect("economy slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: EconomySlice =
            postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
                reason: e.to_string(),
            })?;
        if s.data_hash != self.content_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.data_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's economy data differs from the loaded data/econ content; \
                       install the matching version"
                    .into(),
            });
        }
        Ok(Box::new(s))
    }
}

impl EconomyModule {
    #[allow(clippy::too_many_lines)]
    fn apply_command(
        &self,
        slice: &mut dyn StateSlice,
        cmd: EconomyCommand,
        now: u64,
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let s = self.slice_mut(slice);
        match cmd {
            EconomyCommand::SetBalance {
                faction,
                currency,
                amount,
            } => {
                s.ledger.set_balance(FactionId(faction), currency, amount);
                CommandOutcome::Applied
            }
            EconomyCommand::SetStock {
                commodity,
                location,
                amount,
            } => {
                s.ledger
                    .set_stock(StockKey::new(&commodity, &location), amount);
                CommandOutcome::Applied
            }
            EconomyCommand::Transact {
                faction,
                legs,
                cause,
            } => {
                match s.ledger.apply(Transaction {
                    tick: now,
                    faction: FactionId(faction),
                    legs,
                    cause,
                }) {
                    Ok(()) => CommandOutcome::Applied,
                    Err(e) => CommandOutcome::Rejected(e.to_string()),
                }
            }
            EconomyCommand::DispatchShipment {
                faction,
                edge,
                cargo,
                vehicle,
                edge_price,
            } => self.dispatch(s, faction, &edge, cargo, vehicle, edge_price, now),
            EconomyCommand::RegisterFunding {
                faction,
                period_end_tick,
            } => {
                let f = FactionId(faction);
                let Some(profile) = self.funding_defs.by_faction.get(&f) else {
                    return CommandOutcome::Rejected(format!("no funding profile for {faction}"));
                };
                let state = match profile {
                    FundingProfile::Agency { .. } => FundingState::Agency {
                        period_end_tick,
                        gutted: false,
                    },
                    FundingProfile::Private {
                        starting_cash,
                        daily_burn,
                        ..
                    } => {
                        s.ledger.set_balance(f, Currency::Funds, *starting_cash);
                        FundingState::Private {
                            burn_rate: *daily_burn,
                            bankrupt: false,
                        }
                    }
                };
                s.funding.insert(f, state);
                CommandOutcome::Applied
            }
            EconomyCommand::ApplyAppropriation {
                faction,
                amount,
                directed,
                political_modifier,
            } => {
                let f = FactionId(faction);
                let total = (amount * (1.0 + political_modifier)).max(0.0) + directed.max(0.0);
                let _ = s.ledger.apply(Transaction {
                    tick: now,
                    faction: f,
                    legs: vec![Leg::Currency(f, Currency::Funds, total)],
                    cause: Cause::Appropriation,
                });
                // Gutting check against the caretaker floor.
                if let Some(FundingProfile::Agency {
                    caretaker_floor, ..
                }) = self.funding_defs.by_faction.get(&f)
                {
                    let gutted = total < *caretaker_floor;
                    if let Some(FundingState::Agency {
                        period_end_tick, ..
                    }) = s.funding.get(&f)
                    {
                        let pe = *period_end_tick;
                        s.funding.insert(
                            f,
                            FundingState::Agency {
                                period_end_tick: pe,
                                gutted,
                            },
                        );
                    }
                    if gutted {
                        ctx.emit(
                            "agency-gutted",
                            BTreeMap::from([(
                                "faction".to_string(),
                                ViewValue::U64(u64::from(faction)),
                            )]),
                        );
                    }
                }
                ctx.emit(
                    "budget-cycle",
                    BTreeMap::from([("faction".to_string(), ViewValue::U64(u64::from(faction)))]),
                );
                CommandOutcome::Applied
            }
            EconomyCommand::InjectFinancing { faction, amount } => {
                let f = FactionId(faction);
                let _ = s.ledger.apply(Transaction {
                    tick: now,
                    faction: f,
                    legs: vec![Leg::Currency(f, Currency::Funds, amount.max(0.0))],
                    cause: Cause::Financing,
                });
                CommandOutcome::Applied
            }
            EconomyCommand::BuyLaunch {
                faction,
                orbit_class,
                mass_kg,
            } => {
                let f = FactionId(faction);
                let oc = OrbitClass(orbit_class);
                let Some(price) = self
                    .launch_market
                    .price_at(&oc, s.market.world_capacity_index)
                else {
                    return CommandOutcome::Rejected(format!("unknown orbit class '{}'", oc.0));
                };
                let cost = price * mass_kg.max(0.0);
                match s.ledger.apply(Transaction {
                    tick: now,
                    faction: f,
                    legs: vec![
                        Leg::Currency(f, Currency::Funds, -cost),
                        Leg::Currency(f, Currency::MassToOrbit, mass_kg.max(0.0)),
                    ],
                    cause: Cause::Trade,
                }) {
                    Ok(()) => CommandOutcome::Applied,
                    Err(e) => CommandOutcome::Rejected(e.to_string()),
                }
            }
            EconomyCommand::SetMarketCapacity { index } => {
                s.market.world_capacity_index = index.max(1e-6);
                CommandOutcome::Applied
            }
            EconomyCommand::CommissionFacility { faction, template } => {
                let f = FactionId(faction);
                let Some(def) = self.facility_defs.get(&template) else {
                    return CommandOutcome::Rejected(format!("unknown facility '{template}'"));
                };
                if let Err(e) = s.ledger.apply(Transaction {
                    tick: now,
                    faction: f,
                    legs: vec![Leg::Currency(f, Currency::Funds, -def.capex)],
                    cause: Cause::FacilityOpex,
                }) {
                    return CommandOutcome::Rejected(e.to_string());
                }
                let id = FacilityId(s.next_facility);
                s.next_facility += 1;
                s.facilities.insert(
                    id,
                    Facility {
                        id,
                        faction: f,
                        template: template.clone(),
                        kind: def.kind,
                        level: 1,
                        capacity_per_level: def.capacity,
                    },
                );
                self.resize_ops(s, f);
                CommandOutcome::Applied
            }
            EconomyCommand::UpgradeFacility { faction, facility } => {
                let f = FactionId(faction);
                let Some(fac) = s.facilities.get_mut(&FacilityId(facility)) else {
                    return CommandOutcome::Rejected(format!("unknown facility {facility}"));
                };
                if fac.faction != f {
                    return CommandOutcome::Rejected("facility belongs to another faction".into());
                }
                fac.level = fac.level.saturating_add(1);
                self.resize_ops(s, f);
                CommandOutcome::Applied
            }
            EconomyCommand::SetOpsLoad { faction, used } => {
                let f = FactionId(faction);
                let pool = s.ops.entry(f).or_default();
                pool.used = used.max(0.0);
                if pool.oversubscribed() {
                    let mult = {
                        let rng = ctx.rng("economy/ops-anomaly");
                        1.0 + uniform(rng) * pool.overage_fraction()
                    };
                    ctx.emit(
                        "ops-oversubscribed",
                        BTreeMap::from([
                            ("faction".to_string(), ViewValue::U64(u64::from(faction))),
                            ("anomaly_mult".to_string(), ViewValue::F64(mult)),
                        ]),
                    );
                }
                CommandOutcome::Applied
            }
            EconomyCommand::SiteIsruPlant {
                faction,
                node,
                process,
            } => {
                if self.isru.get(&process).is_none() {
                    return CommandOutcome::Rejected(format!("unknown isru process '{process}'"));
                }
                let id = PlantId(s.next_plant);
                s.next_plant += 1;
                s.plants.insert(
                    id,
                    IsruPlant {
                        id,
                        faction: FactionId(faction),
                        node,
                        process,
                        scale_level: 0,
                        days_at_level: 0,
                        online_tick: 0,
                        cumulative_output: 0.0,
                    },
                );
                CommandOutcome::Applied
            }
            EconomyCommand::OperateIsru {
                faction: _,
                plant,
                days,
                grade,
                power_factor,
            } => self.operate_isru(s, plant, days, &grade, power_factor, now, ctx),
            EconomyCommand::PostRfp {
                issuer,
                node,
                commodity,
                amount,
                reward_funds,
                heritage,
                penalty,
            } => {
                let id = ContractId(s.next_contract);
                s.next_contract += 1;
                s.contracts.insert(
                    id,
                    Contract {
                        id,
                        issuer: FactionId(issuer),
                        deliverable: Deliverable {
                            node,
                            commodity,
                            amount,
                        },
                        reward_funds,
                        heritage,
                        penalty,
                        state: ContractState::Posted,
                    },
                );
                CommandOutcome::Applied
            }
            EconomyCommand::BidContract { faction, contract } => {
                let Some(c) = s.contracts.get_mut(&ContractId(contract)) else {
                    return CommandOutcome::Rejected(format!("unknown contract {contract}"));
                };
                c.bid(FactionId(faction));
                CommandOutcome::Applied
            }
            EconomyCommand::AwardContract { contract, winner } => {
                let Some(c) = s.contracts.get_mut(&ContractId(contract)) else {
                    return CommandOutcome::Rejected(format!("unknown contract {contract}"));
                };
                if c.award(FactionId(winner)) {
                    ctx.emit(
                        "contract-awarded",
                        BTreeMap::from([
                            ("contract".to_string(), ViewValue::U64(u64::from(contract))),
                            ("winner".to_string(), ViewValue::U64(u64::from(winner))),
                        ]),
                    );
                    CommandOutcome::Applied
                } else {
                    CommandOutcome::Rejected("contract not awardable in its state".into())
                }
            }
            EconomyCommand::ReportContract { contract, success } => {
                let Some(c) = s.contracts.get_mut(&ContractId(contract)) else {
                    return CommandOutcome::Rejected(format!("unknown contract {contract}"));
                };
                let issuer = c.issuer;
                let Some((reward, penalty)) = c.report(success) else {
                    return CommandOutcome::Rejected(
                        "contract not in an awardable/working state".into(),
                    );
                };
                let winner = match &c.state {
                    ContractState::Fulfilled(w) | ContractState::Failed(w) => *w,
                    _ => issuer,
                };
                if success {
                    let _ = s.ledger.apply(Transaction {
                        tick: now,
                        faction: winner,
                        legs: vec![Leg::Currency(winner, Currency::Funds, reward)],
                        cause: Cause::ContractReward,
                    });
                } else {
                    let cash = s.ledger.balance(winner, Currency::Funds);
                    let pen = penalty.min(cash.max(0.0));
                    let _ = s.ledger.apply(Transaction {
                        tick: now,
                        faction: winner,
                        legs: vec![Leg::Currency(winner, Currency::Funds, -pen)],
                        cause: Cause::Penalty,
                    });
                }
                CommandOutcome::Applied
            }
            EconomyCommand::FormPartnership { a, b, trust } => {
                let pair = Partnership::order(FactionId(a), FactionId(b));
                s.partnerships.insert(
                    pair,
                    Partnership {
                        pair,
                        trust: trust.clamp(0.0, 1.0),
                    },
                );
                CommandOutcome::Applied
            }
            EconomyCommand::RenegePartnership { a, b } => {
                let pair = Partnership::order(FactionId(a), FactionId(b));
                let Some(p) = s.partnerships.get_mut(&pair) else {
                    return CommandOutcome::Rejected("no such partnership".into());
                };
                p.trust = (p.trust - self.markets.trust_decay_on_renege).max(0.0);
                CommandOutcome::Applied
            }
            EconomyCommand::OpenProject {
                faction,
                node,
                required,
                crew_time_req,
            } => {
                let id = ProjectId(s.next_project);
                s.next_project += 1;
                let required = required
                    .into_iter()
                    .map(|(c, a)| (CommodityId(c), a))
                    .collect();
                s.projects.insert(
                    id,
                    Project {
                        id,
                        faction: FactionId(faction),
                        target_node: node,
                        required,
                        crew_time_req,
                        delivered: BTreeMap::new(),
                        crew_delivered: 0.0,
                        state: ProjectState::Open,
                    },
                );
                CommandOutcome::Applied
            }
            EconomyCommand::DeliverToProject {
                project,
                commodity,
                amount,
            } => {
                let Some(p) = s.projects.get_mut(&ProjectId(project)) else {
                    return CommandOutcome::Rejected(format!("unknown project {project}"));
                };
                p.deliver(&CommodityId(commodity), amount);
                CommandOutcome::Applied
            }
        }
    }

    fn resize_ops(&self, s: &mut EconomySlice, f: FactionId) {
        let capacity: f64 = s
            .facilities
            .values()
            .filter(|fac| fac.faction == f && fac.kind.is_ground_segment())
            .map(crate::facility::Facility::capacity)
            .sum();
        let pool = s.ops.entry(f).or_default();
        pool.capacity = capacity;
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &self,
        s: &mut EconomySlice,
        faction: u32,
        edge: &str,
        cargo: Vec<(String, f64)>,
        vehicle: Vehicle,
        edge_price: EdgePrice,
        now: u64,
    ) -> CommandOutcome {
        let f = FactionId(faction);
        let Some(edge_def) = self.network.edge(edge).cloned() else {
            return CommandOutcome::Rejected(format!("unknown edge '{edge}'"));
        };
        let total_cargo: f64 = cargo.iter().map(|(_, a)| a).sum();
        if total_cargo > vehicle.payload_kg + 1e-9 {
            return CommandOutcome::Rejected(format!(
                "cargo {total_cargo} kg exceeds payload {} kg",
                vehicle.payload_kg
            ));
        }
        // Debit the appropriate budget (R5).
        let legs = match &edge_def.kind {
            EdgeKind::Launch { orbit_class } => {
                let _ = orbit_class;
                vec![Leg::Currency(f, Currency::MassToOrbit, -total_cargo)]
            }
            EdgeKind::InSpace | EdgeKind::Surface => {
                if edge_price.dv_mps > vehicle.dv_capacity_mps + 1e-9 {
                    return CommandOutcome::Rejected(format!(
                        "edge needs {} m/s Δv, vehicle has {}",
                        edge_price.dv_mps, vehicle.dv_capacity_mps
                    ));
                }
                vec![Leg::Currency(f, Currency::DeltaV, -edge_price.dv_mps)]
            }
        };
        if let Err(e) = s.ledger.apply(Transaction {
            tick: now,
            faction: f,
            legs,
            cause: match edge_def.kind {
                EdgeKind::Launch { .. } => Cause::Launch,
                _ => Cause::Transfer,
            },
        }) {
            return CommandOutcome::Rejected(e.to_string());
        }
        // Remove cargo from the origin (conserved) — origin stock if present.
        let from_node = edge_def.from.clone();
        for (cid, amt) in &cargo {
            let key = StockKey {
                commodity: CommodityId(cid.clone()),
                location: LocationId(from_node.clone()),
            };
            let have = s.ledger.stock(&key);
            if have + 1e-9 >= *amt {
                *s.ledger.stocks.entry(key).or_insert(0.0) -= amt;
            }
        }
        let depart = if edge_price.window_open {
            now
        } else {
            edge_price.next_window_tick.max(now)
        };
        let arrive = depart + edge_price.tof_s.max(0.0) as u64;
        let id = ShipmentId(s.next_shipment);
        s.next_shipment += 1;
        s.shipments.insert(
            id,
            Shipment {
                id,
                faction: f,
                edge: edge.to_string(),
                to_node: edge_def.to.clone(),
                cargo: cargo
                    .into_iter()
                    .map(|(c, a)| (CommodityId(c), a))
                    .collect(),
                vehicle,
                depart_tick: depart,
                arrive_tick: arrive,
                state: if edge_price.window_open {
                    ShipmentState::InTransit
                } else {
                    ShipmentState::WaitingWindow
                },
                note: String::new(),
            },
        );
        CommandOutcome::Applied
    }

    #[allow(clippy::too_many_arguments)]
    fn operate_isru(
        &self,
        s: &mut EconomySlice,
        plant: u32,
        days: u64,
        grade: &GradeBelief,
        power_factor: f64,
        now: u64,
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let Some(p) = s.plants.get(&PlantId(plant)).cloned() else {
            return CommandOutcome::Rejected(format!("unknown plant {plant}"));
        };
        let Some(process) = self.isru.get(&p.process).cloned() else {
            return CommandOutcome::Rejected("unknown isru process".into());
        };
        let gf = {
            let rng = ctx.rng("economy/isru-yield");
            isru::grade_factor(grade, uniform(rng))
        };
        let mut output = 0.0;
        let mut scale = p.scale_level;
        let mut days_at = p.days_at_level;
        let mut online_tick = p.online_tick;
        for _ in 0..days {
            output += isru::daily_yield(&process, scale, gf, power_factor);
            days_at += 1;
            if days_at >= process.ramp_days_per_level && scale + 1 < process.scaleup_curve.len() {
                scale += 1;
                days_at = 0;
                if scale + 1 == process.scaleup_curve.len() && online_tick == 0 {
                    online_tick = now;
                    ctx.emit(
                        "isru-online",
                        BTreeMap::from([("plant".to_string(), ViewValue::U64(u64::from(plant)))]),
                    );
                }
            }
        }
        // Credit the local stock (conserved production leg).
        let key = StockKey {
            commodity: process.output.clone(),
            location: LocationId(p.node.clone()),
        };
        *s.ledger.stocks.entry(key).or_insert(0.0) += output;
        let plant_mut = s.plants.get_mut(&PlantId(plant)).unwrap();
        plant_mut.scale_level = scale;
        plant_mut.days_at_level = days_at;
        plant_mut.online_tick = online_tick;
        plant_mut.cumulative_output += output;
        CommandOutcome::Applied
    }
}
