---
name: deduplicate-disperse
description: Cryptographically deduplicates and disperses messy directories into structured vaults using hash maps.
---

# Deduplicate and Disperse Workflow

When the user asks to "clean up", "organize", "dedupe", or "disperse" a directory into a larger workspace, you MUST follow this strict cryptographic workflow:

1. **Do not use `mv` or `cp` blindly.**
2. **Build a Hash Map:** Write and execute a Python script that walks the entire destination vault and computes the MD5 or SHA256 hash of every single file. Store these hashes in a Set.
3. **Compare and Skip:** Walk the source directory. Hash each file. If the hash exists in the destination Set, it is a duplicate. Log it as skipped and DO NOT move it.
4. **Structured Routing:** For unique files, route them into strictly numbered categorical sub-trees (e.g., `01_documentation`, `02_assets`) based on file extension or semantic content.
5. **Safe Copy:** Use `shutil.copy2` to preserve metadata and automatically append numerical suffixes if a filename collision occurs with a unique hash.
6. **Cleanup:** Only after the script reports 100% success should you delete the original source directory.
