class ShadcnRequest {
  component: string;
  overwrite: boolean;
  config_present: boolean;
  constructor(component: string, overwrite: boolean, config_present: boolean) {
    this.component = component;
    this.overwrite = overwrite;
    this.config_present = config_present;
  }
}
function component_name_valid(component: string): boolean {
  if (component === "") {
    return false;
  }
  return true;
}
function tailwind_ext_valid(ext: string): boolean {
  if (ext === "ts") {
    return true;
  }
  if (ext === "js") {
    return true;
  }
  if (ext === "cjs") {
    return true;
  }
  if (ext === "mjs") {
    return true;
  }
  return false;
}
function framework_valid(framework: string): boolean {
  if (framework === "react") {
    return true;
  }
  if (framework === "vue") {
    return true;
  }
  if (framework === "svelte") {
    return true;
  }
  if (framework === "nextjs") {
    return true;
  }
  return false;
}
function shadcn_gate(cfg: ShadcnRequest): boolean {
  if (component_name_valid(cfg.component)) {
    if (cfg.config_present) {
      return true;
    }
    return false;
  }
  return false;
}
function make_shadcn(component: string, overwrite: boolean, config_present: boolean): ShadcnRequest {
  return new ShadcnRequest(component, overwrite, config_present);
}
