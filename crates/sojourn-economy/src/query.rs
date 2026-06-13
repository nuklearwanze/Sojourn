//! The read-only economy-query surface (R17, `contracts/economy-queries.md`). Pure
//! functions over an `EconomySnapshot` composing the economy slice with the
//! composed-value inputs (R2). No mutation, no hidden truth; money figures trace
//! to mass × Δv (FR-EC-805).

use crate::contract::{Contract, Partnership};
use crate::cost::{self, CostEstimate, CostParams};
use crate::funding::FundingState;
use crate::ids::{ContractId, FactionId, OrbitClass, PlantId};
use crate::inputs::{EconomyInputs, TechMaturity, VehicleCost};
use crate::isru::{self, BreakEven, IsruDefs, IsruPlant};
use crate::ledger::{Currency, StockKey};
use crate::market::{LaunchMarketDefs, MarketParams};
use crate::module::{EconomyModule, EconomySlice};
use crate::network::NetworkDefs;
use crate::ops::OpsPool;
use crate::project::Project;
use crate::trace::TraceTree;
use sojourn_core::{CoreError, SimCore};
use std::collections::BTreeMap;

/// A truth-free, query-time snapshot of the economy.
pub struct EconomySnapshot {
    /// Tick.
    pub tick: u64,
    slice: EconomySlice,
    inputs: EconomyInputs,
    launch_market: LaunchMarketDefs,
    markets: MarketParams,
    cost_params: CostParams,
    isru: IsruDefs,
    network: NetworkDefs,
}

/// The cost of moving cargo over an edge (composed from the planner price).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteCost {
    /// Δv to traverse (m/s).
    pub dv_mps: f64,
    /// Time-of-flight (s).
    pub tof_s: f64,
    /// Mass-to-orbit consumed (kg), for launch edges.
    pub mass_to_orbit_kg: f64,
    /// Whether a window is currently open.
    pub window_open: bool,
}

impl EconomySnapshot {
    /// Extract from a running kernel, composing the host-supplied inputs.
    pub fn from_core(
        core: &SimCore,
        module: &EconomyModule,
        inputs: EconomyInputs,
    ) -> Result<EconomySnapshot, CoreError> {
        let tick = core.status().tick;
        core.with_slice("economy", |slice| {
            let s = slice
                .as_any()
                .downcast_ref::<EconomySlice>()
                .expect("economy slice type");
            EconomySnapshot {
                tick,
                slice: s.clone(),
                inputs: inputs.clone(),
                launch_market: module.launch_market.clone(),
                markets: module.markets.clone(),
                cost_params: module.cost.clone(),
                isru: module.isru.clone(),
                network: module.network.clone(),
            }
        })
    }

    /// Build directly from a module + slice + inputs (tests).
    pub fn from_parts(
        module: &EconomyModule,
        slice: EconomySlice,
        inputs: EconomyInputs,
    ) -> EconomySnapshot {
        EconomySnapshot {
            tick: slice.last_tick,
            slice,
            inputs,
            launch_market: module.launch_market.clone(),
            markets: module.markets.clone(),
            cost_params: module.cost.clone(),
            isru: module.isru.clone(),
            network: module.network.clone(),
        }
    }

    // ---- ledger ----

    /// A faction's six currency balances.
    pub fn balances(&self, f: FactionId) -> BTreeMap<Currency, f64> {
        [
            Currency::Funds,
            Currency::DeltaV,
            Currency::MassToOrbit,
            Currency::CrewTime,
            Currency::OpsCapacity,
            Currency::PoliticalCapital,
        ]
        .into_iter()
        .map(|c| (c, self.slice.ledger.balance(f, c)))
        .collect()
    }

    /// The stock of a commodity at a location.
    pub fn stock(&self, commodity: &str, location: &str) -> f64 {
        self.slice.ledger.stock(&StockKey::new(commodity, location))
    }

    /// All non-zero stocks.
    pub fn inventory(&self) -> Vec<(StockKey, f64)> {
        self.slice.ledger.inventory()
    }

    /// Transactions since a tick.
    pub fn journal_since(&self, since_tick: u64) -> usize {
        self.slice.ledger.journal_since(since_tick).len()
    }

    // ---- logistics ----

    /// The route cost for an edge, composing the supplied planner edge price. The
    /// edge's `(from, to)` node locations key the `EdgePrice` input (R2).
    pub fn route_cost(&self, edge_id: &str, cargo_kg: f64) -> Option<RouteCost> {
        let edge = self.network.edge(edge_id)?;
        let from_loc = self.network.node(&edge.from)?.location.clone();
        let to_loc = self.network.node(&edge.to)?.location.clone();
        let price = self.inputs.edge_price(&from_loc, &to_loc)?;
        let mass_to_orbit_kg = match &edge.kind {
            crate::network::EdgeKind::Launch { .. } => cargo_kg,
            _ => 0.0,
        };
        Some(RouteCost {
            dv_mps: price.dv_mps,
            tof_s: price.tof_s,
            mass_to_orbit_kg,
            window_open: price.window_open,
        })
    }

    /// A shipment by id (state inspection).
    pub fn shipment(&self, id: u32) -> Option<&crate::shipment::Shipment> {
        self.slice.shipments.get(&crate::ids::ShipmentId(id))
    }

    // ---- funding ----

    /// A faction's funding state.
    pub fn funding_state(&self, f: FactionId) -> Option<&FundingState> {
        self.slice.funding.get(&f)
    }

    // ---- cost ----

    /// P50/P80 cost estimate for a design at a cumulative production count.
    pub fn cost_estimate(
        &self,
        design: u32,
        maturity: &TechMaturity,
        cumulative_units: u32,
    ) -> Option<CostEstimate> {
        let basis = self.inputs.vehicle_cost(design)?;
        Some(cost::estimate(
            basis,
            maturity,
            cumulative_units,
            &self.cost_params,
        ))
    }

    /// The traceability tree for a cost estimate (money → mass/maturity basis).
    pub fn trace_cost(
        &self,
        design: u32,
        maturity: &TechMaturity,
        cumulative_units: u32,
    ) -> Option<TraceTree> {
        let basis = self.inputs.vehicle_cost(design)?;
        Some(cost::trace_estimate(
            basis,
            maturity,
            cumulative_units,
            &self.cost_params,
        ))
    }

    /// Direct access to a vehicle cost basis (test helper).
    pub fn vehicle_cost(&self, design: u32) -> Option<&VehicleCost> {
        self.inputs.vehicle_cost(design)
    }

    // ---- ISRU ----

    /// An ISRU plant by id.
    pub fn plant(&self, id: u32) -> Option<&IsruPlant> {
        self.slice.plants.get(&PlantId(id))
    }

    /// Break-even for a plant over an operating horizon, composing market prices.
    pub fn isru_break_even(
        &self,
        plant: u32,
        yield_per_day: f64,
        price_per_kg_saved: f64,
        delivery_price_per_kg: f64,
        days: f64,
    ) -> Option<BreakEven> {
        let p = self.slice.plants.get(&PlantId(plant))?;
        let process = self.isru.get(&p.process)?;
        Some(isru::break_even(
            process,
            yield_per_day,
            price_per_kg_saved,
            delivery_price_per_kg,
            days,
        ))
    }

    // ---- markets & contracts ----

    /// The launch $/kg for an orbit class at the current world-capacity index.
    pub fn market_price(&self, orbit_class: &str) -> Option<f64> {
        self.launch_market.price_at(
            &OrbitClass(orbit_class.to_string()),
            self.slice.market.world_capacity_index,
        )
    }

    /// A niche-market price ceiling.
    pub fn niche_price(&self, id: &str) -> Option<f64> {
        self.markets.niche(id).map(|n| n.price_ceiling)
    }

    /// A contract by id.
    pub fn contract(&self, id: u32) -> Option<&Contract> {
        self.slice.contracts.get(&ContractId(id))
    }

    /// A partnership by faction pair.
    pub fn partnership(&self, a: FactionId, b: FactionId) -> Option<&Partnership> {
        self.slice.partnerships.get(&Partnership::order(a, b))
    }

    // ---- facilities & ops ----

    /// Total capacity of a faction's facilities of a kind.
    pub fn facility_capacity(&self, f: FactionId, kind: crate::facility::FacilityKind) -> f64 {
        self.slice
            .facilities
            .values()
            .filter(|fac| fac.faction == f && fac.kind == kind)
            .map(crate::facility::Facility::capacity)
            .sum()
    }

    /// A faction's ops/comms pool utilisation.
    pub fn ops_utilisation(&self, f: FactionId) -> OpsPool {
        self.slice.ops.get(&f).copied().unwrap_or_default()
    }

    // ---- projects ----

    /// A project by id.
    pub fn project(&self, id: u32) -> Option<&Project> {
        self.slice.projects.get(&crate::ids::ProjectId(id))
    }
}
