class HighEndConfig {
  font: string;
  motion_ease: string;
  anim_prop: string;
  blur_target: string;
  constructor(font: string, motion_ease: string, anim_prop: string, blur_target: string) {
    this.font = font;
    this.motion_ease = motion_ease;
    this.anim_prop = anim_prop;
    this.blur_target = blur_target;
  }
}
function font_allowed(font: string): boolean {
  if (font === "inter") {
    return false;
  }
  if (font === "roboto") {
    return false;
  }
  if (font === "arial") {
    return false;
  }
  if (font === "helvetica") {
    return false;
  }
  return true;
}
function ease_allowed(motion_ease: string): boolean {
  if (motion_ease === "linear") {
    return false;
  }
  if (motion_ease === "ease-in-out") {
    return false;
  }
  return true;
}
function prop_allowed(anim_prop: string): boolean {
  if (anim_prop === "transform") {
    return true;
  }
  if (anim_prop === "opacity") {
    return true;
  }
  return false;
}
function blur_allowed(blur_target: string): boolean {
  if (blur_target === "fixed-only") {
    return true;
  }
  return false;
}
function highend_gate(cfg: HighEndConfig): boolean {
  if (font_allowed(cfg.font)) {
    if (ease_allowed(cfg.motion_ease)) {
      if (prop_allowed(cfg.anim_prop)) {
        if (blur_allowed(cfg.blur_target)) {
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
function make_highend(font: string, motion_ease: string, anim_prop: string, blur_target: string): HighEndConfig {
  return new HighEndConfig(font, motion_ease, anim_prop, blur_target);
}
