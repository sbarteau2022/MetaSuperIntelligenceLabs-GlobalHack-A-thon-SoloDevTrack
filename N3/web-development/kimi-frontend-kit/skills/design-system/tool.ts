class TokenPaths {
  has_config: boolean;
  out_dir_set: boolean;
  fix_only: boolean;
  constructor(has_config: boolean, out_dir_set: boolean, fix_only: boolean) {
    this.has_config = has_config;
    this.out_dir_set = out_dir_set;
    this.fix_only = fix_only;
  }
}
function token_ext_valid(ext: string): boolean {
  if (ext === "css") {
    return true;
  }
  if (ext === "json") {
    return true;
  }
  return false;
}
function hardcoded_hit(value: string): boolean {
  if (value === "hardcoded") {
    return true;
  }
  return false;
}
function search_domain_valid(domain: string): boolean {
  if (domain === "strategy") {
    return true;
  }
  if (domain === "layout") {
    return true;
  }
  if (domain === "copy") {
    return true;
  }
  if (domain === "chart") {
    return true;
  }
  return false;
}
function design_system_gate(cfg: TokenPaths, ext: string): boolean {
  if (cfg.has_config) {
    if (token_ext_valid(ext)) {
      if (cfg.out_dir_set) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_token_paths(has_config: boolean, out_dir_set: boolean, fix_only: boolean): TokenPaths {
  return new TokenPaths(has_config, out_dir_set, fix_only);
}
