class WorkbenchConfig {
  dockview_css: boolean;
  editor_reused: boolean;
  layout_mode: string;
  constructor(dockview_css: boolean, editor_reused: boolean, layout_mode: string) {
    this.dockview_css = dockview_css;
    this.editor_reused = editor_reused;
    this.layout_mode = layout_mode;
  }
}
function css_present(dockview_css: boolean): boolean {
  if (dockview_css) {
    return true;
  }
  return false;
}
function editor_valid(editor_reused: boolean): boolean {
  if (editor_reused) {
    return true;
  }
  return false;
}
function layout_valid(layout_mode: string): boolean {
  if (layout_mode === "responsive") {
    return true;
  }
  if (layout_mode === "fixed-workbench") {
    return true;
  }
  return false;
}
function workbench_gate(cfg: WorkbenchConfig): boolean {
  if (css_present(cfg.dockview_css)) {
    if (editor_valid(cfg.editor_reused)) {
      if (layout_valid(cfg.layout_mode)) {
        return true;
      }
      return false;
    }
    return false;
  }
  return false;
}
function make_workbench(dockview_css: boolean, editor_reused: boolean, layout_mode: string): WorkbenchConfig {
  return new WorkbenchConfig(dockview_css, editor_reused, layout_mode);
}
