class ReviewInput {
  spec_traced: boolean;
  edges_done: boolean;
  repro_present: boolean;
  verdict: string;
  constructor(spec_traced: boolean, edges_done: boolean, repro_present: boolean, verdict: string) {
    this.spec_traced = spec_traced;
    this.edges_done = edges_done;
    this.repro_present = repro_present;
    this.verdict = verdict;
  }
}
function verdict_valid(verdict: string): boolean {
  if (verdict === "approve") {
    return true;
  }
  if (verdict === "approve-with-changes") {
    return true;
  }
  if (verdict === "do-not-merge") {
    return true;
  }
  return false;
}
function review_gate(cfg: ReviewInput): boolean {
  if (cfg.spec_traced) {
    if (cfg.edges_done) {
      if (cfg.repro_present) {
        if (verdict_valid(cfg.verdict)) {
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
function make_review(spec_traced: boolean, edges_done: boolean, repro_present: boolean, verdict: string): ReviewInput {
  return new ReviewInput(spec_traced, edges_done, repro_present, verdict);
}
