//! Exact, model-neutral one-head causal attention over an ordered K/V span.
//! Q/K normalization and position transforms are owned by the model adapter.

/// Return one attention head before the model's output gate/projection.
/// `window_start` is an index into the ordered token-major K/V arrays.
pub fn attend_one_head_f32(
    query: &[f32],
    keys: &[f32],
    values: &[f32],
    head_dim: usize,
    window_start: usize,
    scale: f32,
) -> Result<Vec<f32>, String> {
    const MAX_TOKENS: usize = 1_000_000;
    const MAX_SCALARS: usize = 64 * 1024 * 1024;
    if head_dim == 0 || query.len() != head_dim || keys.len() != values.len() {
        return Err("exact attention has inconsistent head dimensions".into());
    }
    if !keys.len().is_multiple_of(head_dim) {
        return Err("exact attention K/V is not token-major".into());
    }
    let token_count = keys.len() / head_dim;
    if token_count == 0
        || token_count > MAX_TOKENS
        || keys.len() > MAX_SCALARS
        || window_start >= token_count
    {
        return Err("exact attention has an invalid causal window".into());
    }
    if !scale.is_finite()
        || scale <= 0.0
        || !query
            .iter()
            .chain(keys)
            .chain(values)
            .all(|x| x.is_finite())
    {
        return Err("exact attention received a non-finite tensor or scale".into());
    }

    let eligible = token_count - window_start;
    let mut scores = vec![0.0f32; eligible];
    let mut max_score = f32::NEG_INFINITY;
    for (offset, score) in scores.iter_mut().enumerate() {
        let token = window_start + offset;
        let key = &keys[token * head_dim..(token + 1) * head_dim];
        let mut dot = 0.0f32;
        for coordinate in 0..head_dim {
            dot += query[coordinate] * key[coordinate];
        }
        *score = dot * scale;
        if !score.is_finite() {
            return Err("exact attention score overflowed".into());
        }
        if *score > max_score {
            max_score = *score;
        }
    }
    let mut denominator = 0.0f32;
    for score in &mut scores {
        *score = (*score - max_score).exp();
        denominator += *score;
    }
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err("exact attention softmax denominator is invalid".into());
    }
    for score in &mut scores {
        *score /= denominator;
    }

    let mut output = vec![0.0f32; head_dim];
    for (offset, weight) in scores.iter().enumerate() {
        let token = window_start + offset;
        let value = &values[token * head_dim..(token + 1) * head_dim];
        for coordinate in 0..head_dim {
            output[coordinate] += weight * value[coordinate];
        }
    }
    if !output.iter().all(|x| x.is_finite()) {
        return Err("exact attention output overflowed".into());
    }
    Ok(output)
}
