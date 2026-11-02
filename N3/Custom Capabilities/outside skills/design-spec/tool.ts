class DesignSpec {
  purpose_set: boolean;
  non_goals_set: boolean;
  rules_numbered: boolean;
  edge_table_done: boolean;
  examples_count: number;
  constructor(purpose_set: boolean, non_goals_set: boolean, rules_numbered: boolean, edge_table_done: boolean, examples_count: number) {
    this.purpose_set = purpose_set;
    this.non_goals_set = non_goals_set;
    this.rules_numbered = rules_numbered;
    this.edge_table_done = edge_table_done;
    this.examples_count = examples_count;
  }
}
function examples_enough(examples_count: number): boolean {
  if (examples_count >= 3) {
    return true;
  }
  return false;
}
function spec_gate(cfg: DesignSpec): boolean {
  if (cfg.purpose_set) {
    if (cfg.non_goals_set) {
      if (cfg.rules_numbered) {
        if (cfg.edge_table_done) {
          if (examples_enough(cfg.examples_count)) {
            return true;
          }
          return false;
        }
        return false;
      }
      return false;
    }
    return false;
  }
  return false;
}
function claim_sealed(sealed: boolean): boolean {
  if (sealed) {
    return true;
  }
  return false;
}
function make_spec(purpose_set: boolean, non_goals_set: boolean, rules_numbered: boolean, edge_table_done: boolean, examples_count: number): DesignSpec {
  return new DesignSpec(purpose_set, non_goals_set, rules_numbered, edge_table_done, examples_count);
}
