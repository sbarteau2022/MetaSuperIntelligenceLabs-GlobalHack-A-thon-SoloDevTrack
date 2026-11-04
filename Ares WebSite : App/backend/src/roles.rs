//! Roles L0 + L1..L12 and epistemic marks (backend-native equivalents of
//! `atlas13::RoleId` / `atlas13::EpistemicMark`).
//!
//! Lane names use RG-AH vocabulary (mission, packet, lane, judge, receipt,
//! atom, arc, basin, cache, tap, gate). No vendor, host-app, file-path, or
//! IDE names in role names.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EpistemicMark {
    Verified = 0,
    Reported = 1,
    Inferred = 2,
    Theory = 3,
}

impl EpistemicMark {
    pub const fn as_tag(self) -> &'static str {
        match self {
            Self::Verified => "[verified]",
            Self::Reported => "[reported]",
            Self::Inferred => "[inferred]",
            Self::Theory => "[theory]",
        }
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "[verified]" => Some(Self::Verified),
            "[reported]" => Some(Self::Reported),
            "[inferred]" => Some(Self::Inferred),
            "[theory]" => Some(Self::Theory),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoleId {
    Coordinator = 0,
    MissionIntake = 1,
    PacketFormer = 2,
    EvidenceTap = 3,
    LaneBuilder = 4,
    LaneVerifier = 5,
    Judge = 6,
    ReceiptWriter = 7,
    AtomKeeper = 8,
    BasinCurator = 9,
    CacheWarden = 10,
    ArcPlanner = 11,
    GateKeeper = 12,
}

impl RoleId {
    pub const ALL_13: [Self; 13] = [
        Self::Coordinator,
        Self::MissionIntake,
        Self::PacketFormer,
        Self::EvidenceTap,
        Self::LaneBuilder,
        Self::LaneVerifier,
        Self::Judge,
        Self::ReceiptWriter,
        Self::AtomKeeper,
        Self::BasinCurator,
        Self::CacheWarden,
        Self::ArcPlanner,
        Self::GateKeeper,
    ];

    pub const CARVED_12: [Self; 12] = [
        Self::MissionIntake,
        Self::PacketFormer,
        Self::EvidenceTap,
        Self::LaneBuilder,
        Self::LaneVerifier,
        Self::Judge,
        Self::ReceiptWriter,
        Self::AtomKeeper,
        Self::BasinCurator,
        Self::CacheWarden,
        Self::ArcPlanner,
        Self::GateKeeper,
    ];

    pub const fn level(self) -> u8 {
        self as u8
    }

    pub const fn carved_index(self) -> Option<usize> {
        let lvl = self as usize;
        if lvl == 0 {
            None
        } else {
            Some(lvl - 1)
        }
    }

    pub const fn from_level(level: u8) -> Option<Self> {
        match level {
            0 => Some(Self::Coordinator),
            1 => Some(Self::MissionIntake),
            2 => Some(Self::PacketFormer),
            3 => Some(Self::EvidenceTap),
            4 => Some(Self::LaneBuilder),
            5 => Some(Self::LaneVerifier),
            6 => Some(Self::Judge),
            7 => Some(Self::ReceiptWriter),
            8 => Some(Self::AtomKeeper),
            9 => Some(Self::BasinCurator),
            10 => Some(Self::CacheWarden),
            11 => Some(Self::ArcPlanner),
            12 => Some(Self::GateKeeper),
            _ => None,
        }
    }

    pub const fn id_slug(self) -> &'static str {
        match self {
            Self::Coordinator => "coordinator",
            Self::MissionIntake => "mission_intake",
            Self::PacketFormer => "packet_former",
            Self::EvidenceTap => "evidence_tap",
            Self::LaneBuilder => "lane_builder",
            Self::LaneVerifier => "lane_verifier",
            Self::Judge => "judge",
            Self::ReceiptWriter => "receipt_writer",
            Self::AtomKeeper => "atom_keeper",
            Self::BasinCurator => "basin_curator",
            Self::CacheWarden => "cache_warden",
            Self::ArcPlanner => "arc_planner",
            Self::GateKeeper => "gate_keeper",
        }
    }

    pub const fn short_code(self) -> &'static str {
        match self {
            Self::Coordinator => "L0",
            Self::MissionIntake => "L1",
            Self::PacketFormer => "L2",
            Self::EvidenceTap => "L3",
            Self::LaneBuilder => "L4",
            Self::LaneVerifier => "L5",
            Self::Judge => "L6",
            Self::ReceiptWriter => "L7",
            Self::AtomKeeper => "L8",
            Self::BasinCurator => "L9",
            Self::CacheWarden => "L10",
            Self::ArcPlanner => "L11",
            Self::GateKeeper => "L12",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Coordinator => "Coordinator",
            Self::MissionIntake => "Mission Intake",
            Self::PacketFormer => "Packet Former",
            Self::EvidenceTap => "Evidence Tap",
            Self::LaneBuilder => "Lane Builder",
            Self::LaneVerifier => "Lane Verifier",
            Self::Judge => "Judge",
            Self::ReceiptWriter => "Receipt Writer",
            Self::AtomKeeper => "Atom Keeper",
            Self::BasinCurator => "Basin Curator",
            Self::CacheWarden => "Cache Warden",
            Self::ArcPlanner => "Arc Planner",
            Self::GateKeeper => "Gate Keeper",
        }
    }
}

/// The backend's own neighbour relation over the 12 carved lanes.
///
/// Backend-native replacement for the source's icosahedral geometry: lanes
/// are neighbours iff they are adjacent stages of the typed workflow
/// (§6: mission → packet → evidence → run → verdict → receipt → atom →
/// capability), plus the coordinator-adjacent governance lanes. This is a
/// workflow adjacency, stated as data — not a geometric claim, and it is
/// not presented as one. Antipodal pairs (maximally distant stages) are
/// never neighbours.
pub fn lane_neighbours(a: RoleId, b: RoleId) -> bool {
    if a == RoleId::Coordinator || b == RoleId::Coordinator || a == b {
        return false;
    }
    // Stage order in the typed workflow.
    fn stage(r: RoleId) -> Option<i32> {
        match r {
            RoleId::MissionIntake => Some(0),
            RoleId::PacketFormer => Some(1),
            RoleId::EvidenceTap => Some(2),
            RoleId::LaneBuilder => Some(3),
            RoleId::LaneVerifier => Some(4),
            RoleId::Judge => Some(5),
            RoleId::ReceiptWriter => Some(6),
            RoleId::AtomKeeper => Some(7),
            RoleId::BasinCurator => Some(8),
            RoleId::CacheWarden => Some(9),
            RoleId::ArcPlanner => Some(10),
            RoleId::GateKeeper => Some(11),
            RoleId::Coordinator => None,
        }
    }
    match (stage(a), stage(b)) {
        (Some(x), Some(y)) => (x - y).abs() == 1,
        _ => false,
    }
}
