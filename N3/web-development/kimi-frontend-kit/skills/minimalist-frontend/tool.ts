class MinimalistConfig {
  shadow: string;
  bg_style: string;
  text_color: string;
  radius: number;
  constructor(shadow: string, bg_style: string, text_color: string, radius: number) {
    this.shadow = shadow;
    this.bg_style = bg_style;
    this.text_color = text_color;
    this.radius = radius;
  }
}
function shadow_allowed(shadow: string): boolean {
  if (shadow === "none") {
    return true;
  }
  if (shadow === "diffuse-005") {
    return true;
  }
  return false;
}
function bg_allowed(bg_style: string): boolean {
  if (bg_style === "flat") {
    return true;
  }
  if (bg_style === "subtle-texture") {
    return true;
  }
  if (bg_style === "imagery") {
    return true;
  }
  return false;
}
function text_allowed(text_color: string): boolean {
  if (text_color === "absolute-black") {
    return false;
  }
  return true;
}
function radius_allowed(radius: number): boolean {
  if (radius <= 12) {
    return true;
  }
  return false;
}
function minimalist_gate(cfg: MinimalistConfig): boolean {
  if (shadow_allowed(cfg.shadow)) {
    if (bg_allowed(cfg.bg_style)) {
      if (text_allowed(cfg.text_color)) {
        if (radius_allowed(cfg.radius)) {
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
function make_minimalist(shadow: string, bg_style: string, text_color: string, radius: number): MinimalistConfig {
  return new MinimalistConfig(shadow, bg_style, text_color, radius);
}
