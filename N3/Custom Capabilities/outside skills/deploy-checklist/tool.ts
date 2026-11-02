class DeployPlan {
  suite_green: boolean;
  rollback_ready: boolean;
  secrets_ok: boolean;
  watch_signals: boolean;
  constructor(suite_green: boolean, rollback_ready: boolean, secrets_ok: boolean, watch_signals: boolean) {
    this.suite_green = suite_green;
    this.rollback_ready = rollback_ready;
    this.secrets_ok = secrets_ok;
    this.watch_signals = watch_signals;
  }
}
function deploy_gate(cfg: DeployPlan): boolean {
  if (cfg.suite_green) {
    if (cfg.rollback_ready) {
      if (cfg.secrets_ok) {
        if (cfg.watch_signals) {
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
function make_deploy(suite_green: boolean, rollback_ready: boolean, secrets_ok: boolean, watch_signals: boolean): DeployPlan {
  return new DeployPlan(suite_green, rollback_ready, secrets_ok, watch_signals);
}
