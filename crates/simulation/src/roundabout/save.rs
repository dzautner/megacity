//! Save/load serialization for `RoundaboutRegistry`.

use crate::grid::RoadType;
use crate::Saveable;

use super::{CirculationDirection, Roundabout, RoundaboutRegistry, RoundaboutTrafficRule};

/// Encode a `RoadType` as a single byte.
pub(crate) fn road_type_to_u8(rt: RoadType) -> u8 {
    match rt {
        RoadType::Local => 0,
        RoadType::Avenue => 1,
        RoadType::Boulevard => 2,
        RoadType::Highway => 3,
        RoadType::OneWay => 4,
        RoadType::Path => 5,
    }
}

/// Decode a `RoadType` from a single byte.
pub(crate) fn road_type_from_u8(b: u8) -> RoadType {
    match b {
        0 => RoadType::Local,
        1 => RoadType::Avenue,
        2 => RoadType::Boulevard,
        3 => RoadType::Highway,
        4 => RoadType::OneWay,
        5 => RoadType::Path,
        _ => RoadType::Local,
    }
}

impl Saveable for RoundaboutRegistry {
    const SAVE_KEY: &'static str = "roundabout_registry";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.roundabouts.is_empty() {
            return None;
        }

        let mut buf = Vec::new();

        // Number of roundabouts (4 bytes)
        let count = self.roundabouts.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());

        for rb in &self.roundabouts {
            // center_x, center_y, radius (4 bytes each = 12 bytes)
            buf.extend_from_slice(&(rb.center_x as u32).to_le_bytes());
            buf.extend_from_slice(&(rb.center_y as u32).to_le_bytes());
            buf.extend_from_slice(&(rb.radius as u32).to_le_bytes());

            // road_type (1 byte)
            buf.push(road_type_to_u8(rb.road_type));

            // direction (1 byte: 0 = Clockwise, 1 = Counterclockwise)
            buf.push(match rb.direction {
                CirculationDirection::Clockwise => 0,
                CirculationDirection::Counterclockwise => 1,
            });

            // traffic_rule (1 byte: 0 = YieldOnEntry, 1 = PriorityOnRoundabout)
            buf.push(match rb.traffic_rule {
                RoundaboutTrafficRule::YieldOnEntry => 0,
                RoundaboutTrafficRule::PriorityOnRoundabout => 1,
            });

            // ring_cells count + data
            let rc_count = rb.ring_cells.len() as u32;
            buf.extend_from_slice(&rc_count.to_le_bytes());
            for &(x, y) in &rb.ring_cells {
                buf.extend_from_slice(&(x as u16).to_le_bytes());
                buf.extend_from_slice(&(y as u16).to_le_bytes());
            }

            // segment_ids count + data
            let seg_count = rb.segment_ids.len() as u32;
            buf.extend_from_slice(&seg_count.to_le_bytes());
            for &sid in &rb.segment_ids {
                buf.extend_from_slice(&sid.to_le_bytes());
            }

            // approach_connections count + data
            let ac_count = rb.approach_connections.len() as u32;
            buf.extend_from_slice(&ac_count.to_le_bytes());
            for &(x, y) in &rb.approach_connections {
                buf.extend_from_slice(&(x as u16).to_le_bytes());
                buf.extend_from_slice(&(y as u16).to_le_bytes());
            }
        }

        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let mut pos = 0;

        let read_u32 = |bytes: &[u8], pos: &mut usize| -> u32 {
            if *pos + 4 > bytes.len() {
                return 0;
            }
            let val = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap_or([0; 4]));
            *pos += 4;
            val
        };

        let read_u16 = |bytes: &[u8], pos: &mut usize| -> u16 {
            if *pos + 2 > bytes.len() {
                return 0;
            }
            let val = u16::from_le_bytes(bytes[*pos..*pos + 2].try_into().unwrap_or([0; 2]));
            *pos += 2;
            val
        };

        let read_u8 = |bytes: &[u8], pos: &mut usize| -> u8 {
            if *pos >= bytes.len() {
                return 0;
            }
            let val = bytes[*pos];
            *pos += 1;
            val
        };

        let count = read_u32(bytes, &mut pos) as usize;
        let mut roundabouts = Vec::with_capacity(count);

        for _ in 0..count {
            let center_x = read_u32(bytes, &mut pos) as usize;
            let center_y = read_u32(bytes, &mut pos) as usize;
            let radius = read_u32(bytes, &mut pos) as usize;
            let road_type = road_type_from_u8(read_u8(bytes, &mut pos));
            let direction = match read_u8(bytes, &mut pos) {
                0 => CirculationDirection::Clockwise,
                _ => CirculationDirection::Counterclockwise,
            };
            let traffic_rule = match read_u8(bytes, &mut pos) {
                0 => RoundaboutTrafficRule::YieldOnEntry,
                _ => RoundaboutTrafficRule::PriorityOnRoundabout,
            };

            let rc_count = read_u32(bytes, &mut pos) as usize;
            let mut ring_cells = Vec::with_capacity(rc_count);
            for _ in 0..rc_count {
                let x = read_u16(bytes, &mut pos) as usize;
                let y = read_u16(bytes, &mut pos) as usize;
                ring_cells.push((x, y));
            }

            let seg_count = read_u32(bytes, &mut pos) as usize;
            let mut segment_ids = Vec::with_capacity(seg_count);
            for _ in 0..seg_count {
                segment_ids.push(read_u32(bytes, &mut pos));
            }

            let ac_count = read_u32(bytes, &mut pos) as usize;
            let mut approach_connections = Vec::with_capacity(ac_count);
            for _ in 0..ac_count {
                let x = read_u16(bytes, &mut pos) as usize;
                let y = read_u16(bytes, &mut pos) as usize;
                approach_connections.push((x, y));
            }

            roundabouts.push(Roundabout {
                center_x,
                center_y,
                radius,
                road_type,
                direction,
                traffic_rule,
                ring_cells,
                segment_ids,
                approach_connections,
            });
        }

        Self {
            roundabouts,
            stats: Vec::new(), // stats are transient, not saved
        }
    }
}
