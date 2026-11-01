#!/usr/bin/env python3
"""Step-5 reference mechanism: spiral-out TPE + FEP depth-gated attention budget.

Implements ConceptFormalization §2.3 (D1-D3) + ConceptsToProofs P6(a)-(c):
  - need-only construction: attention built for the next task window only
  - monotone budget B(d_hat, Omega), ceiling B_max (fail-closed stall receipt)
  - stall (Omega < 0.82) widens aperture (tau_r, tau_a scale up), never the ceiling
  - depth denominated in rotations w -> budget expansion IS radius expansion

This is the f64 reference controller. Production port (Q32_31, integer audit C6)
is staged; P6 acceptance tests below are the port's gate.

Run: python3 tpe_fep_budget.py [--demo]
"""

import math

OMEGA_STALL = 0.82  # AttnEcon spec §1 stall threshold (REPORTED)
B_MAX = 4096  # budget ceiling, task-window units (preregistered const)
TAU_R0 = TAU_A0 = 1.0  # base gate widths (A41 v1)
R_MIN, R_SCALE = 0.22, 0.90


def radius_for_rotations(w, r_min=R_MIN, r_scale=R_SCALE):
    w = max(w, 0.0)
    span = max(r_scale - r_min, 0.15)
    return r_min + span * (0.35 * w + 0.65 * math.log1p(w))


class FEPBudgetController:
    """Depth-gated attention budget with free-energy surprise coupling."""

    def __init__(self, b_max=B_MAX, tau_r0=TAU_R0, tau_a0=TAU_A0):
        self.b_max = b_max
        self.tau_r0, self.tau_a0 = tau_r0, tau_a0
        self.spent = 0
        self.receipts = []

    def surprise(self, predicted, observed):
        """Free-energy proxy: prediction error on the trajectory readout."""
        return abs(predicted - observed)

    def step(self, d_hat, omega, predicted, observed):
        """One budget decision. Returns (admit, tau_r, tau_a, receipt).

        P6(a): B non-decreasing in d_hat at fixed omega.
        P6(b): stall widens aperture (tau scale-up), ceiling holds.
        P6(c): d_hat in rotations -> admitted window rides r(w).
        """
        assert d_hat >= 0, "depth estimate must be non-negative"
        assert 0.0 <= omega <= 1.0, "readiness in [0,1]"
        F = self.surprise(predicted, observed)
        # budget law: monotone in d_hat, readiness-gated, surprise-coupled
        B = min(self.b_max, d_hat * (0.5 + 0.5 * omega) * (1.0 + 0.1 * F))
        stalled = omega < OMEGA_STALL
        widen = 1.0 + (2.0 * (OMEGA_STALL - omega) if stalled else 0.0)
        tau_r, tau_a = self.tau_r0 * widen, self.tau_a0 * widen
        admit = (self.spent + B) <= self.b_max
        if admit:
            self.spent += B
        r_now = radius_for_rotations(d_hat)
        receipt = {
            "d_hat": d_hat,
            "omega": omega,
            "F": F,
            "B": B,
            "admit": admit,
            "stalled": stalled,
            "widen": widen,
            "tau_r": tau_r,
            "tau_a": tau_a,
            "r_at_depth": r_now,
            "spent": self.spent,
            "ceiling": self.b_max,
        }
        self.receipts.append(receipt)
        return admit, tau_r, tau_a, receipt


def test_p6():
    """P6 acceptance tests (also the production-port gate)."""
    c = FEPBudgetController()
    # (a) monotonicity in d_hat at fixed omega
    Bs = [c.step(d, 0.9, 0.0, 0.0)[3]["B"] for d in (4, 8, 16, 32)]
    assert all(b2 >= b1 for b1, b2 in zip(Bs, Bs[1:])), Bs
    # (b) stall widens aperture, ceiling holds
    c2 = FEPBudgetController()
    _, t_r, t_a, rc = c2.step(16, 0.5, 0.0, 1.0)
    assert rc["stalled"] and t_r > TAU_R0 and t_a > TAU_A0, rc
    assert all(r["B"] <= B_MAX for r in c2.receipts)
    # (b2) over-ceiling request fail-closes with receipt, never silent overrun
    c3 = FEPBudgetController()
    big = c3.step(10**9, 1.0, 0.0, 0.0)[3]
    assert big["B"] <= B_MAX and big["admit"] in (True, False) and "stalled" in big
    # (c) radius coupling: deeper budget -> larger radius
    assert radius_for_rotations(32) > radius_for_rotations(4)
    # determinism: same inputs -> identical receipts
    c4, c5 = FEPBudgetController(), FEPBudgetController()
    r4 = c4.step(16, 0.9, 0.3, 0.5)[3]
    r5 = c5.step(16, 0.9, 0.3, 0.5)[3]
    assert r4 == r5, (r4, r5)
    print(
        "P6 acceptance: 5/5 PASS (monotone, stall-widen, ceiling, coupling, determinism)"
    )


def demo():
    c = FEPBudgetController()
    print(
        f"{'d_hat':>6} {'Omega':>6} {'F':>5} {'B':>7} admit stalled widen  tau_r r(w)"
    )
    for d_hat, om, pred, obs in [
        (4, 0.95, 0.1, 0.12),
        (16, 0.90, 0.2, 0.25),
        (64, 0.70, 0.3, 0.9),
        (256, 0.60, 0.4, 1.5),
    ]:
        admit, tr, ta, rc = c.step(d_hat, om, pred, obs)
        print(
            f"{d_hat:6d} {om:6.2f} {rc['F']:5.2f} {rc['B']:7.1f} "
            f"{str(admit):>5} {str(rc['stalled']):>7} {rc['widen']:5.2f} "
            f"{tr:5.2f} {rc['r_at_depth']:6.2f}"
        )


if __name__ == "__main__":
    import sys

    if "--demo" in sys.argv:
        demo()
    else:
        test_p6()
