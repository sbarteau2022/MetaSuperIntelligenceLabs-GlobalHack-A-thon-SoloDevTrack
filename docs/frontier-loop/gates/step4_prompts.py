"""Held-out prompts for the Step 4 gate. Not used to pick K (K is chosen on step3_prompts.PROMPTS)."""

HELDOUT_PROMPTS = [
    [{"role": "user", "content": "Describe how TCP sets up and tears down a connection, in plain language."}],
    [{"role": "user", "content": "Write a SQL query that returns the top 5 customers by total order value, and explain it."}],
    [{"role": "user", "content": "Explain the difference between a process and a thread, with one example of each."}],
]
