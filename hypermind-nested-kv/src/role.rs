//! Map-owned role gate.
//!
//! Formerly `atomadb::Role`. The map's access control is not the RAG's.
//! Same variants, same parse strings, same read/deposit semantics, so
//! existing HYPERMIND_ROLE values keep working.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Role {
    Ingest,
    AgentRt,
    PanelAdmin,
}

impl Role {
    pub fn parse(s: &str) -> Result<Role, String> {
        match s {
            "ingest" => Ok(Role::Ingest),
            "agent_rt" => Ok(Role::AgentRt),
            "panel_admin" => Ok(Role::PanelAdmin),
            _ => Err(format!("unknown role: {s}")),
        }
    }
    pub fn can_read_mind(&self) -> bool {
        matches!(self, Role::AgentRt | Role::PanelAdmin)
    }
    pub fn can_deposit_mind(&self) -> bool {
        matches!(self, Role::AgentRt | Role::PanelAdmin)
    }
    pub fn can_mutate_mind(&self) -> bool {
        matches!(self, Role::PanelAdmin)
    }
}
