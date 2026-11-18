class DocPlan {
  quickstart_tested: boolean;
  api_typed: boolean;
  examples_runnable: boolean;
  decisions_logged: boolean;
  constructor(quickstart_tested: boolean, api_typed: boolean, examples_runnable: boolean, decisions_logged: boolean) {
    this.quickstart_tested = quickstart_tested;
    this.api_typed = api_typed;
    this.examples_runnable = examples_runnable;
    this.decisions_logged = decisions_logged;
  }
}
function doc_gate(cfg: DocPlan): boolean {
  if (cfg.quickstart_tested) {
    if (cfg.api_typed) {
      if (cfg.examples_runnable) {
        if (cfg.decisions_logged) {
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
function make_doc(quickstart_tested: boolean, api_typed: boolean, examples_runnable: boolean, decisions_logged: boolean): DocPlan {
  return new DocPlan(quickstart_tested, api_typed, examples_runnable, decisions_logged);
}
