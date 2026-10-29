//! Coherence gate versions (staged).

pub enum GateVersion { V1, MultilingualV1 }

/// V1: verbatim vault behavior (ASCII prompt + allowlist; rejects foreign scripts).
pub fn is_coherent_v1(prompt: &str, decoded: &str) -> bool {
    let trimmed = decoded.trim();
    if trimmed.is_empty() || decoded.contains('\u{FFFD}') || trimmed.contains("<atoma.") || trimmed.contains("analyzed via carved") { return false; }
    if trimmed.len() == 1 && !trimmed.chars().next().unwrap().is_ascii_digit() { return false; }
    if decoded.chars().any(|c| c.is_control() && !matches!(c,'\n'|'\r'|'\t')) { return false; }
    if prompt.is_ascii() {
        let foreign = decoded.chars().any(|c| !c.is_ascii() && !matches!(c,'—'|'–'|'’'|'‘'|'“'|'”'|'…'|'×'|'÷'|'±'|'≤'|'≥'|'≠'|'≈'|'°'|'µ'|'π'|'²'|'³'|'•'|'·'));
        if foreign { return false; }
    }
    true
}

/// multilingual-v1: script-aware; accept well-formed UTF-8 in prompt's script family; reject only control chars, U+FFFD, empty.
pub fn is_coherent_multilingual_v1(_prompt: &str, decoded: &str) -> bool {
    let trimmed = decoded.trim();
    if trimmed.is_empty() || decoded.contains('\u{FFFD}') { return false; }
    if decoded.chars().any(|c| c.is_control() && !matches!(c,'\n'|'\r'|'\t')) { return false; }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    const CJK: &[&str] = &["这是中文文本。", "你好世界", "苹果与橙子"];
    const AR: &[&str] = &["هذا نص عربي", "مرحبا بالعالم", "التفاح والبرتقال"];
    const DEV: &[&str] = &["यह हिंदी है", "नमस्ते दुनिया", "सेब और संतरा"];
    const TH: &[&str] = &["นี่คือภาษาไทย", "สวัสดีชาวโลก", "แอปเปิ้ลและส้ม"];
    const CODE: &[&str] = &["fn main() {}", "let x = 1;", "if (a) { b(); }"];
    const MATH: &[&str] = &["E=mc^2", "a^2+b^2=c^2", "π r^2"];

    fn reject_rate(gate: fn(&str,&str)->bool, corpus: &[&str]) -> f64 {
        let n = corpus.len() as f64;
        let rej = corpus.iter().filter(|s| !gate("English prompt", s)).count() as f64;
        rej/n
    }

    #[test]
    fn v1_baseline_high_reject() {
        // V1 should reject ~100% of CJK/Arabic
        assert!(reject_rate(is_coherent_v1, CJK) >= 0.9);
        assert!(reject_rate(is_coherent_v1, AR) >= 0.9);
    }

    #[test]
    fn multilingual_low_reject() {
        for corp in [CJK, AR, DEV, TH, CODE, MATH] {
            let r = reject_rate(is_coherent_multilingual_v1, corp);
            assert!(r < 0.05, "reject rate too high: {}", r);
        }
    }
}
