class ProMaxQuery {
  mode: string;
  domain: string;
  stack: string;
  max_results: number;
  constructor(mode: string, domain: string, stack: string, max_results: number) {
    this.mode = mode;
    this.domain = domain;
    this.stack = stack;
    this.max_results = max_results;
  }
}
function promax_domain_valid(domain: string): boolean {
  if (domain === "style") {
    return true;
  }
  if (domain === "color") {
    return true;
  }
  if (domain === "chart") {
    return true;
  }
  if (domain === "landing") {
    return true;
  }
  if (domain === "product") {
    return true;
  }
  if (domain === "ux") {
    return true;
  }
  if (domain === "typography") {
    return true;
  }
  if (domain === "stack") {
    return true;
  }
  return false;
}
function promax_mode_valid(mode: string): boolean {
  if (mode === "search") {
    return true;
  }
  if (mode === "design-system") {
    return true;
  }
  return false;
}
function promax_count_valid(max_results: number): boolean {
  if (max_results >= 1) {
    if (max_results <= 20) {
      return true;
    }
    return false;
  }
  return false;
}
function dial_valid(value: number): boolean {
  if (value >= 1) {
    if (value <= 10) {
      return true;
    }
    return false;
  }
  return false;
}
function promax_gate(cfg: ProMaxQuery): boolean {
  if (promax_mode_valid(cfg.mode)) {
    if (promax_domain_valid(cfg.domain)) {
      if (promax_count_valid(cfg.max_results)) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_promax(mode: string, domain: string, stack: string, max_results: number): ProMaxQuery {
  return new ProMaxQuery(mode, domain, stack, max_results);
}
