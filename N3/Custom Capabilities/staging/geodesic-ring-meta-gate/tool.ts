class GeodesicGate {
  q_fixed: boolean;
  bounded_cap: boolean;
  tests_pass: boolean;
  constructor(q_fixed: boolean, bounded_cap: boolean, tests_pass: boolean) {
    this.q_fixed = q_fixed;
    this.bounded_cap = bounded_cap;
    this.tests_pass = tests_pass;
  }
}
function geodesic_gate(cfg: GeodesicGate): boolean {
  if (cfg.q_fixed) {
    if (cfg.bounded_cap) {
      if (cfg.tests_pass) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function opcode_valid(opcode: number): boolean {
  if (opcode >= 202563) {
    if (opcode <= 202565) {
      return true;
    }
    return false;
  }
  return false;
}
function make_geodesic(q_fixed: boolean, bounded_cap: boolean, tests_pass: boolean): GeodesicGate {
  return new GeodesicGate(q_fixed, bounded_cap, tests_pass);
}
