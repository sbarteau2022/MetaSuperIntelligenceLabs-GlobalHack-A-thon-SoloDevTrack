"""Fixed prompt set for the Step 3 measurement. Shared by the gate and the DSpark baseline bench."""

PROMPTS = [
    [{"role": "user", "content": "Explain how a CPU executes one instruction, in five short paragraphs."}],
    [{"role": "user", "content": "Write a Python function that checks whether a string is a palindrome, and include unit tests."}],
    [{"role": "user", "content": "List the stages of the software development lifecycle and what happens in each."}],
]
MAX_NEW = 128
