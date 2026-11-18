class TechDebtItem {
  evidenced: boolean;
  priced: boolean;
  ranked: boolean;
  pinned: boolean;
  constructor(evidenced: boolean, priced: boolean, ranked: boolean, pinned: boolean) {
    this.evidenced = evidenced;
    this.priced = priced;
    this.ranked = ranked;
    this.pinned = pinned;
  }
}
function techdebt_gate(cfg: TechDebtItem): boolean {
  if (cfg.evidenced) {
    if (cfg.priced) {
      if (cfg.ranked) {
        if (cfg.pinned) {
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
function make_techdebt(evidenced: boolean, priced: boolean, ranked: boolean, pinned: boolean): TechDebtItem {
  return new TechDebtItem(evidenced, priced, ranked, pinned);
}
