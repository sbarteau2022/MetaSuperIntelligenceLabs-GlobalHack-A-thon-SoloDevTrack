use rgah::active_inference::{BoundedResourcePolicy, BudgetedCandidate, Q32_31, ResourceUnit};

fn q(raw: i64) -> Q32_31 {
    Q32_31::from_raw(raw)
}

fn candidate(id: &str, quality: i64, gain: i64, cost: i64) -> BudgetedCandidate {
    BudgetedCandidate {
        id: id.into(),
        quality_lower_bound: q(quality),
        information_gain: q(gain),
        cost_upper_bound: q(cost),
        unit: ResourceUnit::Token,
        reversible: true,
    }
}

#[test]
fn quality_and_budget_filter_before_cost_ordering() {
    let p = BoundedResourcePolicy {
        unit: ResourceUnit::Token,
        remaining: q(10),
        minimum_quality: q(5),
        require_reversible: true,
    };
    let mut c = vec![
        candidate("cheap-low-quality", 4, 0, 0),
        candidate("expensive", 6, 0, 11),
        candidate("winner", 5, 0, 2),
    ];
    c.push(BudgetedCandidate {
        reversible: false,
        ..candidate("irreversible", 5, 0, 1)
    });
    assert_eq!(p.select(&c).unwrap().id, "winner");
    assert!(p.select(&c[..2]).is_err());
}

#[test]
fn q32_31_boundaries_and_invalid_inputs_fail_closed() {
    let p = BoundedResourcePolicy {
        unit: ResourceUnit::Token,
        remaining: q(0),
        minimum_quality: Q32_31::ONE,
        require_reversible: false,
    };
    assert_eq!(
        p.select(&[candidate("zero", Q32_31::ONE.raw(), 0, 0)])
            .unwrap()
            .id,
        "zero"
    );
    for bad in [
        candidate("negative-cost", 0, 0, -1),
        candidate("quality-over-one", Q32_31::ONE.raw() + 1, 0, 0),
        candidate("gain-negative", 0, -1, 0),
        candidate("", 0, 0, 0),
    ] {
        assert!(p.select(&[bad]).is_err());
    }
    let wrong_unit = BudgetedCandidate {
        unit: ResourceUnit::ToolCall,
        ..candidate("wrong", 0, 0, 0)
    };
    assert!(p.select(&[wrong_unit]).is_err());
    assert!(
        p.select(&[candidate("same", 0, 0, 0), candidate("same", 0, 0, 0)])
            .is_err()
    );
    let invalid_budget = BoundedResourcePolicy {
        remaining: q(-1),
        ..p
    };
    assert!(invalid_budget.select(&[candidate("a", 0, 0, 0)]).is_err());
}

#[test]
fn small_grid_matches_independent_enumeration() {
    let p = BoundedResourcePolicy {
        unit: ResourceUnit::Token,
        remaining: q(2),
        minimum_quality: q(1),
        require_reversible: false,
    };
    let mut candidates = Vec::new();
    for cost in 0..=3 {
        for gain in 0..=3 {
            for quality in 0..=3 {
                candidates.push(candidate(
                    &format!("{cost}-{gain}-{quality}"),
                    quality,
                    gain,
                    cost,
                ));
            }
        }
    }
    let expected = candidates
        .iter()
        .filter(|c| c.quality_lower_bound.raw() >= 1 && c.cost_upper_bound.raw() <= 2)
        .map(|c| {
            (
                c.cost_upper_bound.raw(),
                -c.information_gain.raw(),
                -c.quality_lower_bound.raw(),
                c.id.as_str(),
            )
        })
        .min()
        .unwrap();
    let actual = p.select(&candidates).unwrap();
    assert_eq!(
        (
            actual.cost_upper_bound.raw(),
            -actual.information_gain.raw(),
            -actual.quality_lower_bound.raw(),
            actual.id.as_str()
        ),
        expected
    );
}
