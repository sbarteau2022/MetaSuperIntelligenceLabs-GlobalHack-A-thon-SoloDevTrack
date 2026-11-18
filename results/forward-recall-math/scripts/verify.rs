// Exact integer counterexample for reward/free-energy equivalence.
// Let C = H + cN be positive cost in common integer units.
fn main() {
    // Q_A=9/10, C_A=3; Q_B=2/10, C_B=1.
    // R_A>R_B iff 9*1>2*3; F_A>F_B iff 30-9>10-2.
    assert!(9 * 1 > 2 * 3);
    assert!(30 - 9 > 10 - 2);
    // Exhaust small grids to show the two orderings can disagree generally.
    let mut witnesses = 0;
    for qa in 1..=10 {
        for qb in 1..=10 {
            for ca in 1..=10 {
                for cb in 1..=10 {
                    let ratio_prefers_a = qa * cb > qb * ca;
                    let difference_prefers_b = 10 * ca - qa > 10 * cb - qb;
                    if ratio_prefers_a && difference_prefers_b {
                        witnesses += 1;
                    }
                }
            }
        }
    }
    assert!(witnesses > 0);
    println!("exact counterexample verified; small-grid witnesses={witnesses}");
}
