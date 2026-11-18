"""Measure the DSpark drafter baseline on the Step 3 prompts through the running serve.sh server.

Start the server first:  bash /Users/stewartbarteau/Desktop/Qwen3.6Cyber/serve.sh   (port 8001)
Then:  python3 docs/frontier-loop/gates/bench_dspark_baseline.py
Writes gates/step3_dspark_baseline.json with p_dspark = (accept_len - 1) / cap, averaged over prompts.
"""

import json
import pathlib
import sys
import time
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from step3_prompts import MAX_NEW, PROMPTS  # noqa: E402

URL = "http://127.0.0.1:8001/v1/chat/completions"
OUT = pathlib.Path(__file__).resolve().parent / "step3_dspark_baseline.json"


def ask(messages):
    body = json.dumps({
        "model": "Qwen3.6Cyber", "messages": messages, "max_tokens": MAX_NEW,
        "temperature": 0, "top_p": 1, "top_k": 1, "stream": False,
    }).encode()
    req = urllib.request.Request(URL, data=body, headers={
        "Content-Type": "application/json", "Authorization": "Bearer dummy"})
    t0 = time.perf_counter()
    with urllib.request.urlopen(req, timeout=900) as r:
        data = json.loads(r.read())
    return data, time.perf_counter() - t0


def main() -> int:
    rows = []
    for i, msgs in enumerate(PROMPTS):
        data, wall = ask(msgs)
        em = data.get("x_mlx_dspark", {})
        n = data["usage"]["completion_tokens"]
        cap = em.get("cap", 4)
        acc_len = em["accept_len"]
        row = dict(prompt=i, completion_tokens=n, wall_s=wall, tps=n / wall,
                   engine_tps=em.get("tokens_per_sec"), accept_len=acc_len, cap=cap,
                   p=(acc_len - 1) / cap, mode=em.get("mode"))
        rows.append(row)
        print(json.dumps(row))
    p = sum(r["p"] for r in rows) / len(rows)
    tps = sum(r["tps"] for r in rows) / len(rows)
    result = {"p_dspark": p, "dspark_tps": tps, "rows": rows,
              "note": "p = (accept_len-1)/cap from engine metrics; tps = completion tokens / wall time"}
    OUT.write_text(json.dumps(result, indent=2))
    print(f"p_dspark {p:.3f}  dspark_tps {tps:.1f}  -> {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
