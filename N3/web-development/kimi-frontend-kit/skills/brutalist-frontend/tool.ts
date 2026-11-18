class BrutalistConfig {
  substrate: string;
  use_border_radius: boolean;
  use_gradient: boolean;
  text_color: string;
  constructor(substrate: string, use_border_radius: boolean, use_gradient: boolean, text_color: string) {
    this.substrate = substrate;
    this.use_border_radius = use_border_radius;
    this.use_gradient = use_gradient;
    this.text_color = text_color;
  }
}
function substrate_is_valid(substrate: string): boolean {
  if (substrate === "swiss-print") {
    return true;
  }
  if (substrate === "tactical-terminal") {
    return true;
  }
  return false;
}
function corners_allowed(use_border_radius: boolean): boolean {
  if (use_border_radius) {
    return false;
  }
  return true;
}
function gradient_allowed(use_gradient: boolean): boolean {
  if (use_gradient) {
    return false;
  }
  return true;
}
function terminal_green_allowed(target: string): boolean {
  if (target === "single-indicator") {
    return true;
  }
  return false;
}
function brutalist_gate(cfg: BrutalistConfig): boolean {
  if (substrate_is_valid(cfg.substrate)) {
    if (corners_allowed(cfg.use_border_radius)) {
      if (gradient_allowed(cfg.use_gradient)) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_brutalist(substrate: string, use_border_radius: boolean, use_gradient: boolean, text_color: string): BrutalistConfig {
  return new BrutalistConfig(substrate, use_border_radius, use_gradient, text_color);
}
