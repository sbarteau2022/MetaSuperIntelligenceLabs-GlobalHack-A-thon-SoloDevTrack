class R3fFrame {
  set_state_in_frame: boolean;
  animated_prop: string;
  node_count: number;
  node_mode: string;
  constructor(set_state_in_frame: boolean, animated_prop: string, node_count: number, node_mode: string) {
    this.set_state_in_frame = set_state_in_frame;
    this.animated_prop = animated_prop;
    this.node_count = node_count;
    this.node_mode = node_mode;
  }
}
function frame_state_valid(set_state_in_frame: boolean): boolean {
  if (set_state_in_frame) {
    return false;
  }
  return true;
}
function gpu_prop_valid(animated_prop: string): boolean {
  if (animated_prop === "transform") {
    return true;
  }
  if (animated_prop === "opacity") {
    return true;
  }
  return false;
}
function graph_mode_valid(node_count: number, node_mode: string): boolean {
  if (node_count > 200) {
    if (node_mode === "canvas-webgl") {
      return true;
    }
    return false;
  }
  return true;
}
function r3f_gate(cfg: R3fFrame): boolean {
  if (frame_state_valid(cfg.set_state_in_frame)) {
    if (gpu_prop_valid(cfg.animated_prop)) {
      if (graph_mode_valid(cfg.node_count, cfg.node_mode)) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_r3f(set_state_in_frame: boolean, animated_prop: string, node_count: number, node_mode: string): R3fFrame {
  return new R3fFrame(set_state_in_frame, animated_prop, node_count, node_mode);
}
