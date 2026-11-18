class BrainAction {
  action: string;
  domain: string;
  note_present: boolean;
  constructor(action: string, domain: string, note_present: boolean) {
    this.action = action;
    this.domain = domain;
    this.note_present = note_present;
  }
}
function brain_action_valid(action: string): boolean {
  if (action === "status") {
    return true;
  }
  if (action === "save_note") {
    return true;
  }
  return false;
}
function brain_domain_valid(domain: string): boolean {
  if (domain === "coding_intensive") {
    return true;
  }
  if (domain === "security_cyber") {
    return true;
  }
  if (domain === "physics_math") {
    return true;
  }
  if (domain === "legal_statutory") {
    return true;
  }
  if (domain === "medical_clinical") {
    return true;
  }
  return false;
}
function brain_gate(cfg: BrainAction): boolean {
  if (brain_action_valid(cfg.action)) {
    if (brain_domain_valid(cfg.domain)) {
      return true;
    }
    return false;
  }
  return false;
}
function save_note_ok(action: string, note_present: boolean): boolean {
  if (action === "save_note") {
    if (note_present) {
      return true;
    }
    return false;
  }
  return true;
}
function make_brain(action: string, domain: string, note_present: boolean): BrainAction {
  return new BrainAction(action, domain, note_present);
}
