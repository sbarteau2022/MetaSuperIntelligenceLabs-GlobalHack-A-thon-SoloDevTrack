class IncidentReport {
  impact_set: boolean;
  bleeding_stopped: boolean;
  recovery_measured: boolean;
  actions_owned: boolean;
  constructor(impact_set: boolean, bleeding_stopped: boolean, recovery_measured: boolean, actions_owned: boolean) {
    this.impact_set = impact_set;
    this.bleeding_stopped = bleeding_stopped;
    this.recovery_measured = recovery_measured;
    this.actions_owned = actions_owned;
  }
}
function incident_gate(cfg: IncidentReport): boolean {
  if (cfg.impact_set) {
    if (cfg.bleeding_stopped) {
      if (cfg.recovery_measured) {
        if (cfg.actions_owned) {
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
function make_incident(impact_set: boolean, bleeding_stopped: boolean, recovery_measured: boolean, actions_owned: boolean): IncidentReport {
  return new IncidentReport(impact_set, bleeding_stopped, recovery_measured, actions_owned);
}
