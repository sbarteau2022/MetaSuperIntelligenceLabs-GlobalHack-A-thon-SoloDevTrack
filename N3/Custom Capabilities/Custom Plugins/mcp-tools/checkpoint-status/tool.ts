class CheckpointStatus {
  action: string;
  status: string;
  turn_count: number;
  total_0cells: number;
  total_1cell_arcs: number;
  pinned_objective: string;
  ledger_entries: number;
  constructor(action: string, status: string, turn_count: number, total_0cells: number, total_1cell_arcs: number, pinned_objective: string, ledger_entries: number) {
    this.action = action;
    this.status = status;
    this.turn_count = turn_count;
    this.total_0cells = total_0cells;
    this.total_1cell_arcs = total_1cell_arcs;
    this.pinned_objective = pinned_objective;
    this.ledger_entries = ledger_entries;
  }
}

const DEFAULT_TOTAL_0CELLS: number = 222528;
const DEFAULT_TOTAL_1CELL_ARCS: number = 308625;

function checkpoint_path(): string {
  return "/tmp/atoma_brain_latest_checkpoint.json";
}

function status_value(): string {
  return "ok";
}

function milestone_prefix(): string {
  return "milestone: ";
}

function ledger_field(): string {
  return "folded_decisions_and_tools";
}

function is_valid_action(raw: string): boolean {
  if (raw === "status") {
    return true;
  }
  if (raw === "save_note") {
    return true;
  }
  return false;
}

function is_save_note_request(raw: string): boolean {
  if (raw === "save_note") {
    return true;
  }
  return false;
}

function should_append_note(raw: string, note: string): boolean {
  if (raw === "save_note") {
    if (note === "") {
      return false;
    }
    return true;
  }
  return false;
}

function default_turn_count(is_set: boolean, raw: number): number {
  if (is_set) {
    return raw;
  }
  return 0;
}

function default_total_0cells(is_set: boolean, raw: number): number {
  if (is_set) {
    return raw;
  }
  return DEFAULT_TOTAL_0CELLS;
}

function default_total_1cell_arcs(is_set: boolean, raw: number): number {
  if (is_set) {
    return raw;
  }
  return DEFAULT_TOTAL_1CELL_ARCS;
}

function default_pinned_objective(is_set: boolean, raw: string): string {
  if (is_set) {
    return raw;
  }
  return "";
}

function next_ledger_entries(current: number, appended: boolean): number {
  if (appended) {
    return current + 1;
  }
  return current;
}

function make_status(action: string, status: string, turn_count: number, total_0cells: number, total_1cell_arcs: number, pinned_objective: string, ledger_entries: number): CheckpointStatus {
  return new CheckpointStatus(action, status, turn_count, total_0cells, total_1cell_arcs, pinned_objective, ledger_entries);
}
