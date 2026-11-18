import sys

# Test bundle for CLEAN task T-CLEAN-1.
# Submission format: line 1 "VERDICT: clean|defect", optional "DEFECT: <id>" claims, then evidence text.
# Decoy X1 = the `return -1` line. It is correct behaviour, not a defect. Claiming it is a false positive.
lines = open(sys.argv[1]).read().splitlines()
verdict_ok = bool(lines) and lines[0].strip() == "VERDICT: clean"
print("TEST t_verdict " + ("PASS" if verdict_ok else "FAIL"))
claims = [l.strip() for l in lines if l.strip().startswith("DEFECT:")]
print("TEST t_decoy_X1 " + ("FAIL" if any("X1" in c for c in claims) else "PASS"))
evidence = " ".join(lines[1:]).lower()
print("TEST t_evidence " + ("PASS" if ("enumerate" in evidence or "return" in evidence) else "FAIL"))
