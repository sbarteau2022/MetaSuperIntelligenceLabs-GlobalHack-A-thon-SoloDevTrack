class TsConventions {
  no_enum: boolean;
  no_any: boolean;
  brand_ok: boolean;
  import_style: string;
  constructor(no_enum: boolean, no_any: boolean, brand_ok: boolean, import_style: string) {
    this.no_enum = no_enum;
    this.no_any = no_any;
    this.brand_ok = brand_ok;
    this.import_style = import_style;
  }
}
function import_valid(import_style: string): boolean {
  if (import_style === "type-only") {
    return true;
  }
  if (import_style === "satisfies") {
    return true;
  }
  return false;
}
function ts_gate(cfg: TsConventions): boolean {
  if (cfg.no_enum) {
    if (cfg.no_any) {
      if (cfg.brand_ok) {
        if (import_valid(cfg.import_style)) {
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
function make_ts(no_enum: boolean, no_any: boolean, brand_ok: boolean, import_style: string): TsConventions {
  return new TsConventions(no_enum, no_any, brand_ok, import_style);
}
