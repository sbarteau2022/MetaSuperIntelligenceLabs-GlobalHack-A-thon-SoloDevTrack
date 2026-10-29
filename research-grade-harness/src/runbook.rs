use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PacketStatus {
    Pending,
    Ready,
    Running,
    Passed,
    Failed,
    Skipped,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args_json: String,
    pub declared: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeRef {
    pub id: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_rounds: u32,
    pub material_change_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub id: String,
    pub task: String,
    pub depends_on: Vec<String>,
    pub declared_context_tokens: usize,
    pub tool_calls: Vec<ToolCall>,
    pub judges: Vec<JudgeRef>,
    pub branch: Option<String>,
    pub retry_policy: RetryPolicy,
    pub status: PacketStatus,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub objective: String,
    pub packets: Vec<Packet>,
}

impl Plan {
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for p in &self.packets {
            if !ids.insert(p.id.clone()) {
                return Err(format!("duplicate packet id: {}", p.id));
            }
            if p.judges.is_empty() {
                return Err(format!("packet {} has no judges (MadMind hard gate)", p.id));
            }
        }
        let idset: HashSet<String> = self.packets.iter().map(|p| p.id.clone()).collect();
        for p in &self.packets {
            for d in &p.depends_on {
                if !idset.contains(d) {
                    return Err(format!("packet {} depends on unknown {}", p.id, d));
                }
            }
        }
        // adjacency
        let mut adj: std::collections::HashMap<String, Vec<String>> = self
            .packets
            .iter()
            .map(|p| (p.id.clone(), vec![]))
            .collect();
        let mut indeg: std::collections::HashMap<String, usize> = self
            .packets
            .iter()
            .map(|p| (p.id.clone(), p.depends_on.len()))
            .collect();
        for p in &self.packets {
            for d in &p.depends_on {
                adj.get_mut(d).unwrap().push(p.id.clone());
            }
        }
        let mut q: VecDeque<String> = indeg
            .iter()
            .filter(|(_, &v)| v == 0)
            .map(|(k, _)| k.clone())
            .collect();
        let mut seen = 0usize;
        while let Some(n) = q.pop_front() {
            seen += 1;
            for m in adj.get(&n).unwrap() {
                let d = indeg.get_mut(m).unwrap();
                *d = d.saturating_sub(1);
                if *d == 0 {
                    q.push_back(m.clone());
                }
            }
        }
        if seen != self.packets.len() {
            return Err("plan cycle detected".into());
        }
        Ok(())
    }

    pub fn ready_packets(&self) -> Vec<String> {
        self.packets
            .iter()
            .filter(|p| p.status == PacketStatus::Pending)
            .filter(|p| {
                p.depends_on.iter().all(|d| {
                    self.packet(d)
                        .map(|x| x.status == PacketStatus::Passed)
                        .unwrap_or(false)
                })
            })
            .map(|p| p.id.clone())
            .collect()
    }

    pub fn packet(&self, id: &str) -> Option<&Packet> {
        self.packets.iter().find(|p| p.id == id)
    }

    pub fn packet_mut(&mut self, id: &str) -> Option<&mut Packet> {
        self.packets.iter_mut().find(|p| p.id == id)
    }
}
