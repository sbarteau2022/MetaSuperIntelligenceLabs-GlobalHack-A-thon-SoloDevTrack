class DesignVerdict {
  grounded: boolean;
  default_used: boolean;
  tone_ok: boolean;
  accessible: boolean;
  constructor(grounded: boolean, default_used: boolean, tone_ok: boolean, accessible: boolean) {
    this.grounded = grounded;
    this.default_used = default_used;
    this.tone_ok = tone_ok;
    this.accessible = accessible;
  }
}
function brief_wins(brief_pins: boolean, default_used: boolean): boolean {
  if (brief_pins) {
    return true;
  }
  if (default_used) {
    return false;
  }
  return true;
}
function design_gate(cfg: DesignVerdict): boolean {
  if (cfg.grounded) {
    if (brief_wins(true, cfg.default_used)) {
      if (cfg.tone_ok) {
        if (cfg.accessible) {
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
function make_verdict(grounded: boolean, default_used: boolean, tone_ok: boolean, accessible: boolean): DesignVerdict {
  return new DesignVerdict(grounded, default_used, tone_ok, accessible);
}
