class FrontierStack {
  react19: boolean;
  tailwind_v4: boolean;
  tsgo: boolean;
  motion_src: string;
  constructor(react19: boolean, tailwind_v4: boolean, tsgo: boolean, motion_src: string) {
    this.react19 = react19;
    this.tailwind_v4 = tailwind_v4;
    this.tsgo = tsgo;
    this.motion_src = motion_src;
  }
}
function motion_src_valid(motion_src: string): boolean {
  if (motion_src === "motion-react") {
    return true;
  }
  return false;
}
function lenis_valid(surface: string): boolean {
  if (surface === "marketing") {
    return true;
  }
  return false;
}
function frontier_gate(cfg: FrontierStack): boolean {
  if (cfg.react19) {
    if (cfg.tailwind_v4) {
      if (cfg.tsgo) {
        if (motion_src_valid(cfg.motion_src)) {
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
function make_frontier(react19: boolean, tailwind_v4: boolean, tsgo: boolean, motion_src: string): FrontierStack {
  return new FrontierStack(react19, tailwind_v4, tsgo, motion_src);
}
