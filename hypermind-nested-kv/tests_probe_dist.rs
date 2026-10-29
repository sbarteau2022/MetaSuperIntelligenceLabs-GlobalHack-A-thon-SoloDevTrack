//! Probe: pairwise cosh-distance distribution under the new embedder.
//! Run: cargo test -- --test-threads=1 --nocapture probe_dist
#[cfg(test)]
mod probe_dist_tests {
    use super::*;
    #[test]
    fn probe_dist() {
        let texts: Vec<String> = (0..30)
            .map(|i| format!("swarm seed S_code lane 0 routing marker {}", i * 3 + 11))
            .collect();
        let coords: Vec<[f64; 4]> = texts.iter().map(|t| coord_from_text(t, 0.5).0).collect();
        let mut ds: Vec<f64> = Vec::new();
        for i in 0..coords.len() {
            for j in (i + 1)..coords.len() {
                let d = hyperbolic_dist(&coords[i], &coords[j]);
                ds.push(d.cosh() - 1.0);
            }
        }
        ds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("n={} pairs={}", texts.len(), ds.len());
        println!(
            "min={:.3e} p5={:.3e} p25={:.3e} med={:.3e} p75={:.3e} max={:.3e}",
            ds[0],
            ds[ds.len() / 20],
            ds[ds.len() / 4],
            ds[ds.len() / 2],
            ds[ds.len() * 3 / 4],
            ds[ds.len() - 1]
        );
        // distinct-topic control
        let c: Vec<[f64; 4]> = [
            "the quick brown fox jumps over",
            "quantum field theory renormalization group flow equations",
        ]
        .iter()
        .map(|t| coord_from_text(t, 0.5).0)
        .collect();
        println!(
            "distinct-topic cosh-1 = {:.3e}",
            hyperbolic_dist(&c[0], &c[1]).cosh() - 1.0
        );
    }
}
