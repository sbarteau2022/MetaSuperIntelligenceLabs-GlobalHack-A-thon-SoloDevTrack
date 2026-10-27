import json, hashlib, pathlib
outdir = pathlib.Path.home() / "workspace" / "a39-canonical"
def tc(prompt, target, case_sensitive=False):
    return {"prompt": prompt, "expected_output": target,
            "judge": {"check": "token_contains", "target": target, "case_sensitive": case_sensitive}}
def ln(prompt, target):
    return {"prompt": prompt, "expected_output": target,
            "judge": {"check": "last_number_eq", "target": target}}
coding = [
 ("code_001", "rust::synthesize::geodesic_ring_and_meta_gate fn qmatvec_fused_zero_copy simd kernel", "fn"),
 ("code_002", "In plain language, recursion is", "function"),
 ("code_003", "In plain language, a loop is", "repeat"),
 ("code_004", "In plain language, a variable is", "store"),
 ("code_005", "In plain language, an array is", "list"),
 ("code_006", "In plain language, a conditional is", "if"),
 ("code_007", "In plain language, a compiler is", "translate"),
 ("code_008", "In plain language, debugging is", "error"),
 ("code_009", "In Python, a list is ordered and", "mutable"),
 ("code_010", "In Rust, ownership means each value has a single", "owner"),
 ("code_011", "In SQL, SELECT is used to", "retrieve"),
 ("code_012", "A hash function maps input to a fixed", "output"),
 ("code_013", "In plain language, a stack is", "last"),
 ("code_014", "In plain language, a queue is", "first"),
 ("code_015", "In plain language, an algorithm is", "steps"),
 ("code_016", "The capital of France is", "Paris"),
]
security = [
 ("sec_001", "python::synthesize::cwe_and_entropy_scanner cwe-787 out-of-bounds write adversarial scanner", "CWE"),
 ("sec_002", "In plain language, encryption is", "key"),
 ("sec_003", "In plain language, a firewall is", "block"),
 ("sec_004", "In plain language, phishing is", "email"),
 ("sec_005", "In plain language, a password is", "secret"),
 ("sec_006", "In plain language, malware is", "malicious"),
 ("sec_007", "In plain language, authentication is", "identity"),
 ("sec_008", "In plain language, a vulnerability is", "weakness"),
 ("sec_009", "In plain language, a certificate is", "trust"),
 ("sec_010", "In plain language, ransomware is", "ransom"),
 ("sec_011", "In plain language, a sandbox is", "isolate"),
 ("sec_012", "In plain language, a patch is", "update"),
 ("sec_013", "In plain language, two-factor authentication is", "code"),
 ("sec_014", "In plain language, a brute-force attack is", "password"),
 ("sec_015", "In plain language, zero trust is", "verify"),
 ("sec_016", "In plain language, an intrusion is", "unauthorized"),
]
math_def = [
 ("math_001", "cpp20::synthesize::hyperbolic_lorentz_kernel hyperbolic manifold lorentz boost with cosh threshold", "Lorentz"),
 ("math_002", "In plain language, a prime number is", "divisible"),
 ("math_003", "In plain language, gravity is a", "force"),
 ("math_004", "In plain language, a triangle has", "three"),
 ("math_005", "In plain language, pi is the ratio of a circle's circumference to its", "diameter"),
 ("math_006", "In plain language, a square has", "four"),
 ("math_007", "In plain language, an even number is", "two"),
 ("math_008", "In plain language, a fraction is", "part"),
]
math_arith = [
 ("math_009", "A bakery sells muffins for $3 each. It sold 47 muffins in the morning and 38 in the afternoon. Ingredients for the day cost $85. What is the day's profit in dollars?", "170"),
 ("math_010", "What is 12 times 7 plus 5?", "89"),
 ("math_011", "What is 100 minus 37?", "63"),
 ("math_012", "What is 15 times 4?", "60"),
 ("math_013", "What is 20 plus 30, divided by 5?", "10"),
 ("math_014", "What is 9 times 9?", "81"),
 ("math_015", "What is 200 divided by 8?", "25"),
 ("math_016", "What is 7 plus 8 plus 9?", "24"),
]
tasks = []
for tid, prompt, target in coding:
    tasks.append({"task_id": tid, "domain": "coding_intensive", "prompt": prompt,
                  "expected_output": target,
                  "judge": {"check": "token_contains", "target": target, "case_sensitive": False},
                  "seed": "0xC0DE", "verification_status": "candidate"})
for tid, prompt, target in security:
    tasks.append({"task_id": tid, "domain": "security_cyber", "prompt": prompt,
                  "expected_output": target,
                  "judge": {"check": "token_contains", "target": target, "case_sensitive": False},
                  "seed": "0xSEC1", "verification_status": "candidate"})
for tid, prompt, target in math_def:
    tasks.append({"task_id": tid, "domain": "physics_math", "prompt": prompt,
                  "expected_output": target,
                  "judge": {"check": "token_contains", "target": target, "case_sensitive": False},
                  "seed": "0xMATH", "verification_status": "candidate"})
for tid, prompt, target in math_arith:
    tasks.append({"task_id": tid, "domain": "physics_math", "prompt": prompt,
                  "expected_output": target,
                  "judge": {"check": "last_number_eq", "target": target},
                  "seed": "0xMATH", "verification_status": "candidate"})
# arithmetic sanity: recompute each arithmetic answer from its prompt numbers is done by hand above;
# assert counts and uniqueness
assert len(tasks) == 48, len(tasks)
assert len({t["task_id"] for t in tasks}) == 48
from collections import Counter
assert Counter(t["domain"] for t in tasks) == {"coding_intensive": 16, "security_cyber": 16, "physics_math": 16}
doc = {"corpus_id": "a39_candidate_v1", "protocol": "CS-ABLATION-01",
       "status": "CANDIDATE_NOT_FROZEN",
       "note": "Tasks are candidates until probe_corpus.py verifies each expected check against decoded output on a vault copy. A task enters the counted set only if at least one arm produces its expected check in the probe, or it errors in neither arm with a coherent decoded text. The counted set must be >=40 to run A39; if probe drops below 40, expand the corpus, do not lower the bar.",
       "tasks": tasks}
text = json.dumps(doc, indent=2, sort_keys=False) + "\n"
p = outdir / "corpus_a39_candidate_v1.json"
p.write_text(text)
h = hashlib.sha256(text.encode()).hexdigest()
(outdir / "corpus_a39_candidate_v1.sha256").write_text(f"{h}  corpus_a39_candidate_v1.json\n")
print(f"tasks=48 sha256={h} bytes={len(text)}")
