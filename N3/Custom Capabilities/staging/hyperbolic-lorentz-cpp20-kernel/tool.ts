class LorentzCheck {
  constexpr_ok: boolean;
  syntax_ok: boolean;
  constructor(constexpr_ok: boolean, syntax_ok: boolean) {
    this.constexpr_ok = constexpr_ok;
    this.syntax_ok = syntax_ok;
  }
}
function lorentz_gate(cfg: LorentzCheck): boolean {
  if (cfg.constexpr_ok) {
    if (cfg.syntax_ok) {
      return true;
    }
    return false;
  }
  return false;
}
function make_lorentz(constexpr_ok: boolean, syntax_ok: boolean): LorentzCheck {
  return new LorentzCheck(constexpr_ok, syntax_ok);
}
