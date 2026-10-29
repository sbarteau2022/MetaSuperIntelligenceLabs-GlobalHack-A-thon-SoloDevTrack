---
name: goal
description: Set, show or steer a durable goal for the current folder that Codex keeps working on across turns until it is done, blocked, paused or out of rounds. Use when the user types /goal, or asks Codex to keep going on one objective until it is finished.
argument-hint: "[<objective> [--max-rounds N] | edit <objective> | pause | resume | clear]"
---

# /goal

One objective per working folder, stored in `~/Desktop/OPS_BOARD/goals/` and logged on the OPS_BOARD ledger.
While a goal is active, a Stop hook (`~/.Codex/settings.json`) sends Codex back to work each time it would stop,
up to the round cap (default 20). The tool is `~/Desktop/OPS_BOARD/tools/goal/goal.py`.

Arguments: `$ARGUMENTS`

## Run the matching command (from the current working folder)

| Arguments | Command |
|---|---|
| empty | `python3 ~/Desktop/OPS_BOARD/tools/goal/goal.py show` |
| exactly `pause`, `resume` or `clear` | `python3 ~/Desktop/OPS_BOARD/tools/goal/goal.py <word>` |
| `edit <objective>` | `python3 ~/Desktop/OPS_BOARD/tools/goal/goal.py edit <objective>` |
| anything else | `python3 ~/Desktop/OPS_BOARD/tools/goal/goal.py set <objective> [--max-rounds N]` |

A control word counts only when it is the whole argument: `/goal pause after tests pass` sets that sentence as a new
objective. Quote the objective so the shell passes it intact. Show the user the command's output.

## After setting a goal
Start on it in this same turn. Work the way you would on any task; the hook brings you back if you stop early.

## Finishing a goal (only Codex does this, never on a plan or a partial result)
Run `show` first and copy the exact id and revision.
- Done and verified (tests pass, the file exists, the command's output shows it):
  `python3 ~/Desktop/OPS_BOARD/tools/goal/goal.py complete --id <id> --rev <rev> --note "<what proves it>"`
- Can't go further without the user (a decision, access, missing input):
  `python3 ~/Desktop/OPS_BOARD/tools/goal/goal.py block --id <id> --rev <rev> --reason "<exactly what you need>"`

A wrong revision is refused: the user changed the goal since you read it, so read it again. Never edit the state
files by hand, and never complete a goal just to make the hook stop.

## Stopping it
- The user interrupts and types `/goal pause` (or `/goal clear`).
- The round cap stops it and says so; `/goal resume` adds 20 more rounds.
- `ELLE_GOAL_DRIVER=off` in the environment turns continuation off everywhere.
