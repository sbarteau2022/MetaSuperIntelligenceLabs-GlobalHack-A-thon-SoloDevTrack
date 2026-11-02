class ReactConventions {
  server_by_default: boolean;
  boundary: string;
  state_level: string;
  async_model: string;
  constructor(server_by_default: boolean, boundary: string, state_level: string, async_model: string) {
    this.server_by_default = server_by_default;
    this.boundary = boundary;
    this.state_level = state_level;
    this.async_model = async_model;
  }
}
function boundary_valid(boundary: string): boolean {
  if (boundary === "deep-leaf-client") {
    return true;
  }
  return false;
}
function state_valid(state_level: string): boolean {
  if (state_level === "local") {
    return true;
  }
  if (state_level === "lifted-shared") {
    return true;
  }
  if (state_level === "zustand-global") {
    return true;
  }
  if (state_level === "tanstack-server") {
    return true;
  }
  return false;
}
function async_valid(async_model: string): boolean {
  if (async_model === "suspense-action") {
    return true;
  }
  return false;
}
function react_gate(cfg: ReactConventions): boolean {
  if (cfg.server_by_default) {
    if (boundary_valid(cfg.boundary)) {
      if (state_valid(cfg.state_level)) {
        if (async_valid(cfg.async_model)) {
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
function make_react(server_by_default: boolean, boundary: string, state_level: string, async_model: string): ReactConventions {
  return new ReactConventions(server_by_default, boundary, state_level, async_model);
}
