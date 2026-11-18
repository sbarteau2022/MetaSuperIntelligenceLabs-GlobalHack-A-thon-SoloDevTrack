class PluginBuild {
  spec_done: boolean;
  examples_count: number;
  sandbox_ok: boolean;
  approved: boolean;
  kind: string;
  constructor(spec_done: boolean, examples_count: number, sandbox_ok: boolean, approved: boolean, kind: string) {
    this.spec_done = spec_done;
    this.examples_count = examples_count;
    this.sandbox_ok = sandbox_ok;
    this.approved = approved;
    this.kind = kind;
  }
}
function kind_valid(kind: string): boolean {
  if (kind === "skill") {
    return true;
  }
  if (kind === "plugin") {
    return true;
  }
  if (kind === "connector") {
    return true;
  }
  return false;
}
function examples_valid(examples_count: number): boolean {
  if (examples_count >= 2) {
    return true;
  }
  return false;
}
function plugin_gate(cfg: PluginBuild): boolean {
  if (cfg.spec_done) {
    if (examples_valid(cfg.examples_count)) {
      if (cfg.sandbox_ok) {
        if (cfg.approved) {
          if (kind_valid(cfg.kind)) {
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
function make_plugin(spec_done: boolean, examples_count: number, sandbox_ok: boolean, approved: boolean, kind: string): PluginBuild {
  return new PluginBuild(spec_done, examples_count, sandbox_ok, approved, kind);
}
