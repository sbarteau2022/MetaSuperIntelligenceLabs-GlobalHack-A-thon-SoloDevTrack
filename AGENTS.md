# Workspace rule — read first, applies to every agent, every session

CANONICAL WORKSPACE: this folder, and only this folder.
It is the git repo. If you are not working inside it, you are in the
wrong place.

ONE PLACE TO SUBMIT WORK: this repo folder. Submission means the file
shows up in `git status` inside this repo. A file anywhere else is
not submitted, no matter what was claimed about it.

NEVER write, create, or stage files in:
- any other folder under ~/Desktop/GitHub (this repo now lives directly under ~/Desktop/GitHub; the old container folder no longer exists),
- ~/Desktop, ~/Desktop/AtomaApp, ~/Desktop/Special Ops,
- ~/workspace, /tmp, or any other agent scratch area,
  except as a temporary draft you explicitly hand back for placement.

If a path, a tool, or a previous session points you outside this repo,
STOP and ask Stewart. Do not silently place the file outside and do
not invent a new home for it.

Build outputs are never staged: target/, node_modules/, dist/, *.log.

Before reporting anything done, run `git status` in this repo and
show it. No git status proof, no "done."
