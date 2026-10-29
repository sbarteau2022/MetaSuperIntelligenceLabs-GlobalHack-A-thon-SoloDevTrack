use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TacticalLevel {
    T0March,
    T1Scout,
    T2Maneuver,
    T3Campaign,
    T4GrandStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Status,
    Receipts,
    SetLevel,
    Run,
    Judge,
    Recon,
    Measure,
    Inventory,
    Plan,
    Branch,
    Fallback,
    Merge,
    Temple,
    Gates,
    Budget,
    Escalate,
    Preregister,
    Hypotheses,
    Probe,
    Recalculate,
    ThreatModel,
    Decoy,
    Mirror,
}

impl Command {
    pub fn slash(&self) -> &'static str {
        match self {
            Command::Status => "/status",
            Command::Receipts => "/receipts",
            Command::SetLevel => "/level",
            Command::Run => "/run",
            Command::Judge => "/judge",
            Command::Recon => "/recon",
            Command::Measure => "/measure",
            Command::Inventory => "/inventory",
            Command::Plan => "/plan",
            Command::Branch => "/branch",
            Command::Fallback => "/fallback",
            Command::Merge => "/merge",
            Command::Temple => "/temple",
            Command::Gates => "/gates",
            Command::Budget => "/budget",
            Command::Escalate => "/escalate",
            Command::Preregister => "/preregister",
            Command::Hypotheses => "/hypotheses",
            Command::Probe => "/probe",
            Command::Recalculate => "/recalculate",
            Command::ThreatModel => "/threatmodel",
            Command::Decoy => "/decoy",
            Command::Mirror => "/mirror",
        }
    }
    pub fn level(&self) -> TacticalLevel {
        match self {
            Command::Status
            | Command::Receipts
            | Command::SetLevel
            | Command::Run
            | Command::Judge => TacticalLevel::T0March,
            Command::Recon | Command::Measure | Command::Inventory => TacticalLevel::T1Scout,
            Command::Plan | Command::Branch | Command::Fallback | Command::Merge => {
                TacticalLevel::T2Maneuver
            }
            Command::Temple
            | Command::Gates
            | Command::Budget
            | Command::Escalate
            | Command::Preregister => TacticalLevel::T3Campaign,
            Command::Hypotheses
            | Command::Probe
            | Command::Recalculate
            | Command::ThreatModel
            | Command::Decoy
            | Command::Mirror => TacticalLevel::T4GrandStrategy,
        }
    }
}

impl FromStr for TacticalLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "T0" => Ok(TacticalLevel::T0March),
            "T1" => Ok(TacticalLevel::T1Scout),
            "T2" => Ok(TacticalLevel::T2Maneuver),
            "T3" => Ok(TacticalLevel::T3Campaign),
            "T4" => Ok(TacticalLevel::T4GrandStrategy),
            _ => Err(()),
        }
    }
}

pub fn allowed_commands(level: TacticalLevel) -> Vec<Command> {
    use Command::*;
    let all = [
        Status,
        Receipts,
        SetLevel,
        Run,
        Judge,
        Recon,
        Measure,
        Inventory,
        Plan,
        Branch,
        Fallback,
        Merge,
        Temple,
        Gates,
        Budget,
        Escalate,
        Preregister,
        Hypotheses,
        Probe,
        Recalculate,
        ThreatModel,
        Decoy,
        Mirror,
    ];
    all.into_iter().filter(|c| c.level() <= level).collect()
}

pub fn authorize(level: TacticalLevel, cmd: Command) -> Result<(), String> {
    if cmd.level() <= level {
        Ok(())
    } else {
        Err(format!("{} not allowed at {:?}", cmd.slash(), level))
    }
}

/// Levels never drop mid-problem (monotone).
pub fn raise_level(cur: TacticalLevel, req: TacticalLevel) -> TacticalLevel {
    if req > cur {
        req
    } else {
        cur
    }
}
