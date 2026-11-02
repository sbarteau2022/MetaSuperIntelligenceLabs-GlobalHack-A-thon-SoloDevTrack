class DreamRequest {
  charts: number;
  bridges: number;
  base: string;
  path: string;
  constructor(charts: number, bridges: number, base: string, path: string) {
    this.charts = charts;
    this.bridges = bridges;
    this.base = base;
    this.path = path;
  }
}

function default_charts(is_set: boolean, raw: number): number {
  if (is_set) {
    return raw;
  }
  return 16;
}

function default_bridges(is_set: boolean, raw: number): number {
  if (is_set) {
    return raw;
  }
  return 4;
}

function default_base(is_set: boolean, raw: string): string {
  if (is_set) {
    return raw;
  }
  return "http://127.0.0.1:8088";
}

function dream_path(): string {
  return "/v1/mind/dream";
}

function make_request(charts: number, bridges: number, base: string, path: string): DreamRequest {
  return new DreamRequest(charts, bridges, base, path);
}
