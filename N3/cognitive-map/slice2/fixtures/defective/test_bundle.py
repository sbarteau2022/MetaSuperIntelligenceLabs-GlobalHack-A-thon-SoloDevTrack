import sys

# Test bundle for DEFECTIVE task T-DEFECT-1.
# Required defect D1 = range walks one past the end (IndexError on miss).
# Submission format: line 1 "VERDICT: clean|defect", optional "DEFECT: <id>" claims, then evidence text.
lines = open(sys.argv[1]).read().splitlines()
verdict_ok = bool(lines) and lines[0].strip() == "VERDICT: defect"
print("TEST t_verdict " + ("PASS" if verdict_ok else "FAIL"))
claims = [l.strip() for l in lines if l.strip().startswith("DEFECT:")]
print("TEST t_loc_D1 " + ("PASS" if any("D1" in c for c in claims) else "FAIL"))
evidence = " ".join(lines[1:]).lower()
print("TEST t_evidence " + ("PASS" if ("index" in evidence or "range" in evidence or "end" in evidence) else "FAIL"))
