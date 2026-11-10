//! Bounded, model-neutral target proposal frames. Selection is not a gate pass.

const MAX_CANDIDATES: usize = 16;
const MAX_TARGET_BYTES: usize = 64;
const MAX_EXCERPT_BYTES: usize = 2048;
const MAX_RATIONALE_BYTES: usize = 1024;

pub struct Candidate<'a> {
    pub target: &'a str,
    pub evidence_sha256: &'a str,
    pub excerpt: &'a [u8],
}

pub struct Choice {
    pub target: String,
    pub model_identity_claim: String,
    pub rationale: String,
}

fn lower_hex(bytes: &[u8], len: usize) -> bool {
    bytes.len() == len
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn valid_target(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len() <= MAX_TARGET_BYTES
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn take<'a>(bytes: &'a [u8], offset: &mut usize, len: usize) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(len)
        .ok_or("proposal frame length overflow")?;
    let result = bytes.get(*offset..end).ok_or("proposal frame truncated")?;
    *offset = end;
    Ok(result)
}

pub fn encode_request(source: &str, candidates: &[Candidate<'_>]) -> Result<Vec<u8>, String> {
    if !lower_hex(source.as_bytes(), 40) {
        return Err("proposal source invalid".into());
    }
    if candidates.is_empty() || candidates.len() > MAX_CANDIDATES {
        return Err("proposal candidate count invalid".into());
    }
    let mut frame = Vec::new();
    frame.extend_from_slice(b"FRQ1");
    frame.extend_from_slice(source.as_bytes());
    frame.push(u8::try_from(candidates.len()).map_err(|e| e.to_string())?);
    for candidate in candidates {
        if !valid_target(candidate.target.as_bytes())
            || !lower_hex(candidate.evidence_sha256.as_bytes(), 64)
            || candidate.excerpt.len() > MAX_EXCERPT_BYTES
        {
            return Err("proposal candidate invalid".into());
        }
        frame.push(u8::try_from(candidate.target.len()).map_err(|e| e.to_string())?);
        frame.extend_from_slice(candidate.target.as_bytes());
        frame.extend_from_slice(candidate.evidence_sha256.as_bytes());
        frame.extend_from_slice(
            &u16::try_from(candidate.excerpt.len())
                .map_err(|e| e.to_string())?
                .to_le_bytes(),
        );
        frame.extend_from_slice(candidate.excerpt);
    }
    Ok(frame)
}

pub fn decode_response(
    bytes: &[u8],
    expected_source: &str,
    candidates: &[Candidate<'_>],
) -> Result<Choice, String> {
    let mut offset = 0;
    if take(bytes, &mut offset, 4)? != b"FRS1" {
        return Err("proposal response magic invalid".into());
    }
    let source = take(bytes, &mut offset, 40)?;
    if !lower_hex(source, 40) || source != expected_source.as_bytes() {
        return Err("proposal response source mismatch".into());
    }
    let target_len = usize::from(take(bytes, &mut offset, 1)?[0]);
    let target_bytes = take(bytes, &mut offset, target_len)?;
    if !valid_target(target_bytes) {
        return Err("proposal response target invalid".into());
    }
    let target = std::str::from_utf8(target_bytes).map_err(|e| e.to_string())?;
    if !candidates
        .iter()
        .any(|candidate| candidate.target == target)
    {
        return Err("proposal response target was not admitted".into());
    }
    let identity = take(bytes, &mut offset, 64)?;
    if !lower_hex(identity, 64) {
        return Err("proposal response model identity claim invalid".into());
    }
    let rationale_len = u16::from_le_bytes(
        take(bytes, &mut offset, 2)?
            .try_into()
            .map_err(|_| "proposal rationale length invalid")?,
    ) as usize;
    if rationale_len == 0 || rationale_len > MAX_RATIONALE_BYTES {
        return Err("proposal rationale length invalid".into());
    }
    let rationale = take(bytes, &mut offset, rationale_len)?;
    if offset != bytes.len() {
        return Err("proposal response has trailing bytes".into());
    }
    Ok(Choice {
        target: target.to_string(),
        model_identity_claim: std::str::from_utf8(identity)
            .map_err(|e| e.to_string())?
            .to_string(),
        rationale: std::str::from_utf8(rationale)
            .map_err(|e| e.to_string())?
            .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{decode_response, encode_request, Candidate};

    const SOURCE: &str = "26dfa6ddf073f986fecb4037b1aa20d9f7b5633c";
    const DIGEST: &str = "539f1567994ecf9ed3926c30157d851df79fe092e516380d7f3710f9cdf46a49";

    fn candidate() -> Candidate<'static> {
        Candidate {
            target: "ares-backend-tests",
            evidence_sha256: DIGEST,
            excerpt: b"real gate output excerpt",
        }
    }

    fn response(target: &str) -> Vec<u8> {
        let mut bytes = b"FRS1".to_vec();
        bytes.extend_from_slice(SOURCE.as_bytes());
        bytes.push(target.len() as u8);
        bytes.extend_from_slice(target.as_bytes());
        bytes.extend_from_slice(DIGEST.as_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(b"ok");
        bytes
    }

    #[test]
    fn request_and_admitted_response_validate() {
        let encoded = encode_request(SOURCE, &[candidate()]).expect("valid request");
        assert!(encoded.starts_with(b"FRQ1"));
        let choice = decode_response(&response("ares-backend-tests"), SOURCE, &[candidate()])
            .expect("admitted response");
        assert_eq!(choice.target, "ares-backend-tests");
        assert_eq!(choice.rationale, "ok");
    }

    #[test]
    fn stale_unknown_and_extended_responses_fail_closed() {
        let valid = response("ares-backend-tests");
        assert!(decode_response(
            &valid,
            "0000000000000000000000000000000000000000",
            &[candidate()]
        )
        .is_err());
        assert!(decode_response(&response("unseen"), SOURCE, &[candidate()]).is_err());
        let mut extended = valid;
        extended.push(0);
        assert!(decode_response(&extended, SOURCE, &[candidate()]).is_err());
        let mut empty_rationale = response("ares-backend-tests");
        empty_rationale.truncate(empty_rationale.len() - 4);
        empty_rationale.extend_from_slice(&0u16.to_le_bytes());
        assert!(decode_response(&empty_rationale, SOURCE, &[candidate()]).is_err());
    }
}
