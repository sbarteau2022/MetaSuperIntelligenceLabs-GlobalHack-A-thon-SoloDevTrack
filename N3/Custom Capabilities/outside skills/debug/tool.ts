class DebugCase {
  reproduced: boolean;
  failing_test: boolean;
  cause_fixed: boolean;
  suite_green: boolean;
  constructor(reproduced: boolean, failing_test: boolean, cause_fixed: boolean, suite_green: boolean) {
    this.reproduced = reproduced;
    this.failing_test = failing_test;
    this.cause_fixed = cause_fixed;
    this.suite_green = suite_green;
  }
}
function debug_gate(cfg: DebugCase): boolean {
  if (cfg.reproduced) {
    if (cfg.failing_test) {
      if (cfg.cause_fixed) {
        if (cfg.suite_green) {
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
function deploy_ready(suite_green: boolean, rollback_ready: boolean, secrets_ok: boolean): boolean {
  if (suite_green) {
    if (rollback_ready) {
      if (secrets_ok) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_debug(reproduced: boolean, failing_test: boolean, cause_fixed: boolean, suite_green: boolean): DebugCase {
  return new DebugCase(reproduced, failing_test, cause_fixed, suite_green);
}
