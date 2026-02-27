//! Formatting helpers for [`WorldSnapshot`] data — produces human/LLM-readable
//! text summaries of city spatial state.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::world_snapshot::{
    BuildingEntry, RoadCellEntry, ServiceEntry, UtilityEntry, WaterRegion, ZoneRegion,
};

/// Format building entries as a human-readable table.
pub fn format_buildings(entries: &[BuildingEntry]) -> String {
    if entries.is_empty() {
        return "No buildings.".to_string();
    }

    let mut out = String::new();
    writeln!(
        out,
        "Buildings ({} total):",
        entries.len()
    )
    .ok();
    writeln!(
        out,
        "  {:>8} {:>8}  {:>16}  {:>5}  {:>8}  {:>9}",
        "X", "Y", "Zone", "Level", "Capacity", "Occupancy"
    )
    .ok();
    for b in entries {
        writeln!(
            out,
            "  {:>8} {:>8}  {:>16?}  {:>5}  {:>8}  {:>9}",
            b.pos.0, b.pos.1, b.zone_type, b.level, b.capacity, b.occupancy,
        )
        .ok();
    }
    out
}

/// Format service entries as a human-readable list.
pub fn format_services(entries: &[ServiceEntry]) -> String {
    if entries.is_empty() {
        return "No services.".to_string();
    }

    let mut out = String::new();
    writeln!(out, "Services ({} total):", entries.len()).ok();
    for s in entries {
        writeln!(
            out,
            "  ({:>3},{:>3})  {:?}  radius={:.0}",
            s.pos.0, s.pos.1, s.service_type, s.radius,
        )
        .ok();
    }
    out
}

/// Format utility entries as a human-readable list.
pub fn format_utilities(entries: &[UtilityEntry]) -> String {
    if entries.is_empty() {
        return "No utilities.".to_string();
    }

    let mut out = String::new();
    writeln!(out, "Utilities ({} total):", entries.len()).ok();
    for u in entries {
        writeln!(
            out,
            "  ({:>3},{:>3})  {:?}  range={}",
            u.pos.0, u.pos.1, u.utility_type, u.range,
        )
        .ok();
    }
    out
}

/// Format road cells as a summary count by road type.
pub fn format_roads_summary(entries: &[RoadCellEntry]) -> String {
    if entries.is_empty() {
        return "No roads.".to_string();
    }

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for r in entries {
        *counts.entry(format!("{:?}", r.road_type)).or_default() += 1;
    }

    let mut out = String::new();
    writeln!(
        out,
        "Roads ({} total cells):",
        entries.len()
    )
    .ok();
    for (road_type, count) in &counts {
        writeln!(out, "  {road_type}: {count} cells").ok();
    }
    out
}

/// Format zone regions as a summary table.
pub fn format_zones_summary(regions: &[ZoneRegion]) -> String {
    if regions.is_empty() {
        return "No zoned areas.".to_string();
    }

    let mut out = String::new();
    writeln!(
        out,
        "Zone Regions ({} total):",
        regions.len()
    )
    .ok();
    for z in regions {
        let w = z.max.0 - z.min.0 + 1;
        let h = z.max.1 - z.min.1 + 1;
        writeln!(
            out,
            "  ({},{})-({},{})  {:?}  {}x{} ({} cells, {} buildings)",
            z.min.0,
            z.min.1,
            z.max.0,
            z.max.1,
            z.zone_type,
            w,
            h,
            w * h,
            z.building_count,
        )
        .ok();
    }
    out
}

/// Format water regions as a summary.
pub fn format_terrain(water: &[WaterRegion]) -> String {
    if water.is_empty() {
        return "No water bodies.".to_string();
    }

    let mut out = String::new();
    writeln!(
        out,
        "Water Regions ({} total):",
        water.len()
    )
    .ok();
    for w_region in water {
        let w = w_region.max.0 - w_region.min.0 + 1;
        let h = w_region.max.1 - w_region.min.1 + 1;
        writeln!(
            out,
            "  ({},{})-({},{})  {}x{} ({} cells)",
            w_region.min.0,
            w_region.min.1,
            w_region.max.0,
            w_region.max.1,
            w,
            h,
            w * h,
        )
        .ok();
    }
    out
}
