# HyperMind in Plain Language (Goal ab6d8eed)

**What it is.** A memory map for AI agents. You put knowledge in as text.
The map stores each piece at a coordinate — like a library where every book
has a GPS position, and books about similar things sit near each other.
You ask questions; the map returns what's nearby. Every answer carries a
receipt (a fingerprint proving what was returned and when).

## The pieces, plainly

**Atoms — the books.** One ingested text = one atom. Each atom has:
- the text itself,
- a coordinate (4 numbers placing it in a curved space called H³,
  plus 2 phase numbers placing it on a donut called T²),
- a salience (how important: high salience sits near the center),
- a momentum (which direction it "moves": phase velocity, radial drift,
  rapidity — the symplectic half),
- provenance (where it came from), and a receipt chain.

**Ingest — shelving a book.** `map.ingest` runs three gates before shelving:
1. **Judge.** Is this worth keeping? Rejects empty text, 1-character
   fragments, repetitive sludge (entropy floor). A caller can also veto.
   No pass, no atom — nothing is stored.
2. **Dedup.** Is this already on the shelf? Same text (or same words) at
   the same spot reinforces the existing atom (salience +0.05) instead of
   storing a duplicate. No swamp.
3. **Embed.** Which shelf? Content statistics (word weights learned from
   the map's own vocabulary) pick the direction; salience picks the radius.

**Query — finding books.** Two ways, for two questions:
- `map.query`: "what is near this point?" Geometric search (VP-tree,
  exact, fast) fused with word-match scoring (BM25). Ask in your own
  words; paraphrase works when the words overlap the topic.
- `map.lexical`: "which document says X?" Pure word-match over every
  atom. Slower (scans all), more precise for document lookup.

**Attend — reading with a budget.** `map.attend` retrieves, then picks how
many to actually read using a cost rule: reading more costs more, but
quality (score mass) pays back. Returns the count plus proof the shortlist
matched the exact answer.

**Traverse — walking between books.** `map.traverse` checks BEFORE moving
from atom A to atom B:
- **Form:** is there actually a path, or would this be teleporting?
  (Triangle rule of the curved space.)
- **Energy:** can we afford the walk? (Cost vs budget.)
- **Frame:** do we arrive intact, or sheared? (Momentum continuity.)
Fail any check and you get `fail-explained` — the reason, the numbers,
and which budget to raise — never a silent dead end. This is what the old
system couldn't do: it died at `mlir_op#26` with no explanation; the
envelope says *why not*.

**Reweight — learning the vocabulary.** `map.reweight` reads every atom,
learns which words are rare and discriminating (IDF), and re-shelves
everything. Run it after a big ingest. Takes one pass; all future embeds
use the learned weights.

**Snapshot/recover — the save file.** `map.snapshot` freezes the whole map
with a fingerprint. `map.recover` verifies the fingerprint and restores.
`map.export_passport` ships the map elsewhere.

## What it proves (measured, not claimed)

- 100,000 pieces shelved in one pass, every one retrievable by exact text.
- 2,068 live pieces (design docs + code + task knowledge), all receipted.
- Unseen topics: 64% of retrieved pieces useful, every test question
  answered (held-out A39: p_useful=0.639, coverage 12/12).
- Traversal failures explain themselves with cost/budget numbers.

## What it doesn't do (honest limits)

- Cross-importance recall fails: shelve at high importance, ask flatly,
  and the map misses (0%). Match the importance or normalize first.
- The geometric search alone doesn't understand paraphrase — the word-match
  arm carries document search. A GPU encoder (Baseten) closes this gap.
- Usefulness scores on home-turf vocabulary (1.000) are circular; the
  held-out 0.639 is the real number.
