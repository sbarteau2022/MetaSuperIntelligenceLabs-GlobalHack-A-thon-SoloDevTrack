class CarveResult {
  domain: string = "";
  basin0: string = "";
  basin1: string = "";
  basin2: string = "";
  basin3: string = "";
  basin_count: number = 0;
  mass_pct: number = 0;
  pruned_pct: number = 0;
  holonomy_rad: number = 0;
  endpoint: string = "";
  constructor() {
    this.domain = "";
    this.basin0 = "";
    this.basin1 = "";
    this.basin2 = "";
    this.basin3 = "";
    this.basin_count = 0;
    this.mass_pct = 0;
    this.pruned_pct = 0;
    this.holonomy_rad = 0;
    this.endpoint = "";
  }
}
function carve_specialist(domain: string): CarveResult {
  let r: CarveResult = new CarveResult();
  if (domain === "security_cyber") {
    r.domain = "security_cyber";
    r.basin0 = "L7";
    r.basin1 = "L3";
    r.basin2 = "L12";
    r.basin3 = "";
    r.basin_count = 3;
    r.mass_pct = 51.3;
    r.pruned_pct = 75.4;
    r.holonomy_rad = 0.3244;
    r.endpoint = "http://127.0.0.1:8088/api/carve?domain=security_cyber";
    return r;
  }
  if (domain === "physics_math") {
    r.domain = "physics_math";
    r.basin0 = "L6";
    r.basin1 = "L11";
    r.basin2 = "L7";
    r.basin3 = "";
    r.basin_count = 3;
    r.mass_pct = 67.1;
    r.pruned_pct = 74.1;
    r.holonomy_rad = 0.3529;
    r.endpoint = "http://127.0.0.1:8088/api/carve?domain=physics_math";
    return r;
  }
  if (domain === "legal_statutory") {
    r.domain = "legal_statutory";
    r.basin0 = "L4";
    r.basin1 = "L10";
    r.basin2 = "L7";
    r.basin3 = "";
    r.basin_count = 3;
    r.mass_pct = 60.7;
    r.pruned_pct = 73.2;
    r.holonomy_rad = 0.5049;
    r.endpoint = "http://127.0.0.1:8088/api/carve?domain=legal_statutory";
    return r;
  }
  if (domain === "medical_clinical") {
    r.domain = "medical_clinical";
    r.basin0 = "L9";
    r.basin1 = "L5";
    r.basin2 = "L3";
    r.basin3 = "";
    r.basin_count = 3;
    r.mass_pct = 54.7;
    r.pruned_pct = 75.7;
    r.holonomy_rad = 0.3529;
    r.endpoint = "http://127.0.0.1:8088/api/carve?domain=medical_clinical";
    return r;
  }
  r.domain = "coding_intensive";
  r.basin0 = "L4";
  r.basin1 = "L7";
  r.basin2 = "L2";
  r.basin3 = "L6";
  r.basin_count = 4;
  r.mass_pct = 85.9;
  r.pruned_pct = 64.3;
  r.holonomy_rad = 0.6774;
  r.endpoint = "http://127.0.0.1:8088/api/carve?domain=coding_intensive";
  return r;
}
