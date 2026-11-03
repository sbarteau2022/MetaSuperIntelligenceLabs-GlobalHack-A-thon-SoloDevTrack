# Live model seat attestation for benchmark setup

Date: 2026-11-03. The backend's model list must describe loaded serving state, not a hardcoded Qwen name. The Rust native server exposes `GET /v1/info` as one size-prefixed AMST metadata frame containing its 64-hex model identity and profile. The backend already validates this frame format for generation.

`/api/models` will probe the configured native MTP base URL, parse exactly one metadata frame, reject a missing/invalid identity or wrong frame kind, and report the attested identity and profile only when the probe succeeds. A down native server is `up=false` with no invented model. oMLX and gateway seats remain reported from their own live endpoints; they are not treated as interchangeable native forward implementations. A successful info probe proves only the serving seat identity, not model output, tokenizer parity, K/V reuse, or Glimmer support.

Gate: parser tests for valid metadata, wrong kind, truncated frame and identity mismatch; route output must not hardcode `Qwen3.6-MoE-Combined`. A model swap creates a fresh mission executor and invalidates incompatible prefix/K/V pages. The first benchmark is a frozen native baseline; TPPE Q/K replacement is separately ablated because the Glimmer checkpoint was trained with RoPE/NoPE.
