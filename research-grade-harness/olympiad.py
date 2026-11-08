#!/usr/bin/env python3
"""Adversarial math-olympiad pipeline executed entirely by the local
Qwen3.6Cyber model (DSpark :8001). One model plays every role across
fresh-context calls — the structure of claude-plugins-official/math-olympiad:

  solve (thinking, self-refine) -> strip reasoning -> adversarial verify x N
  (fresh context, pattern-armed, blind) -> asymmetric vote (4 confirm / 2 refute)
  -> revise -> calibrated abstention.

Dual context isolation is real here: each verifier call is a brand-new request
containing only (problem + cleaned proof), never the solver's thinking.

Usage: python olympiad.py "<problem statement>"   [--solvers N] [--numeric]
"""
import json, os, re, sys, urllib.request, argparse, collections

URL = os.environ.get("DSPARK_URL", "http://127.0.0.1:8001/v1/chat/completions")
MODEL = os.environ.get("DSPARK_MODEL", "Qwen3.6Cyber")
KEY = os.environ.get("DSPARK_API_KEY", "dummy")

def call(system, user, max_tokens=16000, think=True):
    msgs = [{"role": "system", "content": system}, {"role": "user", "content": user}]
    body = json.dumps({"model": MODEL, "messages": msgs, "max_tokens": max_tokens,
                       "temperature": 0}).encode()
    r = urllib.request.Request(URL, data=body, headers={"Content-Type": "application/json",
                                                        "Authorization": f"Bearer {KEY}"})
    d = json.load(urllib.request.urlopen(r, timeout=600))
    m = d["choices"][0]["message"]
    return (m.get("content") or ""), (m.get("reasoning_content") or "")

SOLVER_SYS = (
 "You are an olympiad mathematician. NO COMPUTATION as proof: a pattern that fits "
 "n<=10 is not a proof. Process: (1) Interpretation check - state which reading you "
 "solve and why (olympiad problems hide a trap-easy reading). (2) Solve rigorously. "
 "(3) Self-improve: reread, fix gaps. (4) Self-verify in strict-grader mode: every "
 "step justified? (5) Correct and re-verify, up to 5 rounds. A correct answer from "
 "flawed reasoning is a failure; never hide gaps - mark [GAP: ...] inline. "
 "End with these sections only:\n"
 "**Method**: <one paragraph key idea>\n**Proof**: <full step-by-step, every step justified>\n"
 "**Answer**: <final answer, or 'incomplete'>")

VERIFIER_SYS = (
 "You are an ADVERSARIAL verifier. You are NOT grading - you are trying to BREAK the "
 "proof. Assume a strong author made one subtle error a sympathetic reader would gloss "
 "over. You have NO tools and do NOT know any other reviewer's verdict - assume you are "
 "the first and only. Run these attacks:\n"
 "#1 Step doesn't follow (incl. direction errors: A>B,C>D does NOT give A-C>B-D).\n"
 "#5 Invoked theorem's hypotheses - recheck FROM SCRATCH ('continuous on [0,1]' != on R).\n"
 "#3 Claim false in a small case (test n=2,3 mentally).\n"
 "#18 Tautological: substitute the proof's own identities into any 'remaining gap' - does it "
 "recover the original claim?\n"
 "#4 Proves too much: does the skeleton specialize to a famous object and settle something "
 "open/false?\n"
 "#40 One-line lemma: extract its GENERAL form, find a 2x2 counterexample; if the general form "
 "is false, is there special structure saving THIS instance?\n"
 "#60 Wrong interpretation: solves an easier reading than intended?\n"
 "#7 Hand-wave at the crux ('by standard methods','routine') exactly where it isn't.\n"
 "If none fire after a genuine attempt, CONFIRM - but never confirm just because it sounds "
 "confident. Return exactly:\n**Verdict**: HOLDS | HOLE FOUND | UNCLEAR\n"
 "**If HOLE FOUND**: Location=<quoted step> Pattern=<#> Why=<specific> Fixable=<yes:how|no>")

REVISER_SYS = (
 "Fix the proof given a verifier's hole report. You do NOT get the original thinking - work "
 "from the hole. If the approach is fundamentally broken, return '**Verdict**: no confident "
 "solution' with surviving partial results. Mark any step you cannot fully close inline as "
 "[GAP: ...]. Output the same **Method**/**Proof**/**Answer** sections.")

def clean(content, reasoning=""):
    # Prefer the structured sections. This reasoning model often emits the whole
    # worked solution in reasoning_content and leaves `content` short/empty, so
    # fall back to reasoning when content has no Method/Proof and is thin.
    for body in (content, reasoning):
        if not body:
            continue
        m = re.search(r"\*\*Method\*\*.*", body, re.S)
        if m:
            return m.group(0).strip()
    best = content.strip() if content and len(content.strip()) > 40 else ""
    return best or (reasoning.strip() if reasoning else content.strip())

def verdict_of(text):
    t = text.upper()
    if "HOLE FOUND" in t: return "HOLE"
    if "HOLDS" in t: return "HOLDS"
    return "UNCLEAR"

def solve_once(problem, angle):
    c, think = call(SOLVER_SYS, f"PROBLEM: {problem}\nANGLE: {angle}")
    return clean(c, think)

def adversarial_vote(problem, proof, max_passes=5):
    holds = holes = 0; reports = []
    for i in range(max_passes):
        c, _ = call(VERIFIER_SYS, f"PROBLEM: {problem}\n\nPROPOSED SOLUTION:\n{proof}", max_tokens=8000)
        v = verdict_of(c); reports.append((v, c))
        if v == "HOLE": holes += 1
        elif v == "HOLDS": holds += 1
        if holes >= 2: return "REFUTED", reports      # asymmetric: 2 to refute
        if holds >= 4: return "CONFIRMED", reports     # 4 to confirm
    return ("CONFIRMED" if holds > holes else "REFUTED" if holes else "UNCLEAR"), reports

ANGLES = ["Work small cases then generalize", "Find an invariant or monovariant",
          "Extremal / consider the boundary case", "Induction", "Exploit symmetry",
          "Work backwards from the goal", "Drop a condition - where does it fail?",
          "Inventor's paradox - prove a stronger statement"]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("problem")
    ap.add_argument("--solvers", type=int, default=3)
    ap.add_argument("--numeric", action="store_true", help="AIME mode: majority vote on answer")
    a = ap.parse_args()

    if a.numeric:
        votes = collections.Counter()
        for i in range(max(a.solvers, 5)):
            p = solve_once(a.problem, ANGLES[i % len(ANGLES)])
            n = re.findall(r'-?\d+', p.replace(",", ""))
            if n: votes[n[-1]] += 1
        print("votes:", dict(votes))
        if votes:
            top, k = votes.most_common(1)[0]
            print(f"\nFINAL (majority {k}/{sum(votes.values())}): {top}")
        else:
            print("no confident solution")
        return

    print(f"== {a.solvers} solvers → adversarial vote → revise ==")
    best = None
    for i in range(a.solvers):
        proof = solve_once(a.problem, ANGLES[i % len(ANGLES)])
        outcome, reports = adversarial_vote(a.problem, proof)
        print(f"solver {i+1} ({ANGLES[i%len(ANGLES)][:28]}): {outcome}")
        if outcome == "CONFIRMED":
            print("\n=== CONFIRMED PROOF ===\n" + proof); return
        if best is None: best = (proof, reports)
    # one revise cycle on the best candidate
    proof, reports = best
    hole = next((c for v, c in reports if v == "HOLE"), "")
    if hole:
        rev, rt = call(REVISER_SYS, f"PROBLEM: {a.problem}\n\nSOLUTION:\n{proof}\n\nHOLE:\n{hole}")
        rev = clean(rev, rt)
        outcome, _ = adversarial_vote(a.problem, rev)
        print(f"after revise: {outcome}")
        if outcome == "CONFIRMED":
            print("\n=== CONFIRMED (revised) ===\n" + rev); return
    print("\n**Verdict**: no confident solution\n(best partial below)\n" + proof[:1200])

if __name__ == "__main__":
    main()
