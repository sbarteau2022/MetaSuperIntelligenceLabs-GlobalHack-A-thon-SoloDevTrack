#!/usr/bin/env python3
"""Hard code+math eval against the live Qwen3.6Cyber DSpark server.
Math: integer answers computed by in-script ground truth (no grading error).
Code: the model's function is extracted and EXECUTED against inline test asserts
      in an isolated `python -I` subprocess with a timeout.
Deterministic grading; temp 0.
"""
import json, re, os, sys, subprocess, textwrap, urllib.request, math

URL = os.environ.get("DSPARK_URL", "http://127.0.0.1:8001/v1/chat/completions")
MODEL = os.environ.get("DSPARK_MODEL", "Qwen3.6Cyber")
KEY = os.environ.get("DSPARK_API_KEY", "dummy")

def chat(system, user, max_tokens=6000):
    body = json.dumps({"model": MODEL, "messages": [
        {"role": "system", "content": system}, {"role": "user", "content": user}],
        "max_tokens": max_tokens, "temperature": 0}).encode()
    r = urllib.request.Request(URL, data=body, headers={"Content-Type": "application/json",
        "Authorization": f"Bearer {KEY}"})
    d = json.load(urllib.request.urlopen(r, timeout=300))
    return d["choices"][0]["message"].get("content") or ""

def _fib(n):
    a, b = 1, 1
    for _ in range(n - 2): a, b = b, a + b
    return b

# ---------- MATH: (question, ground-truth lambda) ----------
MATH = [
    ("How many positive integers less than 1000 are divisible by neither 5 nor 7?",
     lambda: sum(1 for n in range(1, 1000) if n % 5 and n % 7)),
    ("What is the remainder when 7^100 is divided by 13?", lambda: pow(7, 100, 13)),
    ("What is the sum of the digits of 2^50?", lambda: sum(int(c) for c in str(2 ** 50))),
    ("How many trailing zeros are in 200 factorial?", lambda: sum(200 // 5 ** k for k in range(1, 5))),
    ("How many diagonals does a convex 20-gon have?", lambda: 20 * 17 // 2),
    ("What is the least common multiple of 24, 36, and 90?", lambda: math.lcm(24, 36, 90)),
    ("How many ways can you choose 4 items from 15 (combinations)?", lambda: math.comb(15, 4)),
    ("What is the 12th Fibonacci number, with F(1)=1 and F(2)=1?", lambda: _fib(12)),
    ("What is the sum of all prime numbers below 50?",
     lambda: sum(p for p in range(2, 50) if all(p % d for d in range(2, int(p ** 0.5) + 1)))),
    ("What is 15! divided by 13! ?", lambda: math.factorial(15) // math.factorial(13)),
]

def last_int(text):
    nums = re.findall(r'-?\d+', text.replace(",", ""))
    return int(nums[-1]) if nums else None

MATH_SYS = "You are a careful mathematician. Reason step by step, then end with: FINAL <integer> and nothing after it."

# ---------- CODE: (spec, inline test source using the function) ----------
CODE = [
    ("Write a Python function `two_sum(nums, target)` returning indices [i,j] (i<j) of two numbers summing to target, or None.",
     "assert sorted(two_sum([2,7,11,15],9))==[0,1]\nassert sorted(two_sum([3,2,4],6))==[1,2]\nassert sorted(two_sum([3,3],6))==[0,1]"),
    ("Write a Python function `lcs(a, b)` returning the length of the longest common subsequence of strings a and b.",
     "assert lcs('abcde','ace')==3\nassert lcs('abc','abc')==3\nassert lcs('abc','def')==0"),
    ("Write a Python function `is_balanced(s)` returning True iff the brackets ()[]{} in s are balanced.",
     "assert is_balanced('([]{})')==True\nassert is_balanced('([)]')==False\nassert is_balanced('')==True\nassert is_balanced('(((')==False"),
    ("Write a Python function `nth_prime(n)` returning the n-th prime (1-indexed; nth_prime(1)=2).",
     "assert nth_prime(1)==2\nassert nth_prime(6)==13\nassert nth_prime(25)==97"),
    ("Write a Python function `edit_distance(a, b)` returning the Levenshtein distance between a and b.",
     "assert edit_distance('kitten','sitting')==3\nassert edit_distance('','abc')==3\nassert edit_distance('abc','abc')==0"),
    ("Write a Python function `covered_length(intervals)` returning total length covered by a list of [start,end] intervals (overlaps merged).",
     "assert covered_length([[1,3],[2,6],[8,10]])==7\nassert covered_length([[1,4],[4,5]])==4\nassert covered_length([])==0"),
]
CODE_SYS = "You are an expert programmer. Return ONLY one Python function in a ```python code block. No explanation, no tests, no prints."

def extract_code(text):
    m = re.search(r"```(?:python)?\s*(.*?)```", text, re.S)
    return (m.group(1) if m else text).strip()

def run_code(src, tests):
    probe = src + "\n" + tests + "\nprint('RESULT:PASS')\n"
    try:
        out = subprocess.run([sys.executable, "-I", "-c", probe],
            capture_output=True, text=True, timeout=15)
        return "RESULT:PASS" in out.stdout
    except Exception:
        return False

def main():
    print(f"== model {MODEL} @ {URL} ==\n")
    mp = 0
    print("MATH")
    for q, gt in MATH:
        truth = gt()
        try: ans = last_int(chat(MATH_SYS, q))
        except Exception: ans = None
        ok = ans == truth; mp += ok
        print(f"  [{'PASS' if ok else 'FAIL'}] truth={truth} got={ans}  {q[:52]}")
    cp = 0
    print("\nCODE (executed)")
    for spec, tests in CODE:
        try: src = extract_code(chat(CODE_SYS, spec))
        except Exception: src = ""
        ok = run_code(src, tests); cp += ok
        fn = re.search(r'`(\w+)\(', spec)
        print(f"  [{'PASS' if ok else 'FAIL'}] {fn.group(1) if fn else spec[:30]}")
    tot, n = mp + cp, len(MATH) + len(CODE)
    print(f"\nSCORE  math {mp}/{len(MATH)}   code {cp}/{len(CODE)}   total {tot}/{n}  ({100*tot//n}%)")

if __name__ == "__main__":
    main()
