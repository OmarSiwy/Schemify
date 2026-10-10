selfcheck ok

## Concurrent-edit merges (1000-object design, 50 random trials each, `git merge-file`)

Cell = clean&correct / conflict / clean-but-WRONG (merged file parses to something other than both edits applied).

| format | move inst X ∥ param on inst Y | insert inst ∥ insert inst | insert wire ∥ delete wire | move inst X ∥ param on same X | rename net ∥ move wire on it | 5 wire moves ∥ 5 wire deletes | 10 moves ∥ 10 param edits |
|---|---|---|---|---|---|---|---|
| sexpr-1line | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 46 / 4 / 0 | 35 / 15 / 0 |
| sexpr-kicad | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 50 / 0 / 0 |
| kdl | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 46 / 4 / 0 | 50 / 0 / 0 |
| xschem | 50 / 0 / 0 | 0 / 50 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 0 / 50 / 0 | 45 / 5 / 0 | 33 / 17 / 0 |
| jsonl | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 46 / 4 / 0 | 35 / 15 / 0 |
| sect | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 46 / 4 / 0 | 35 / 15 / 0 |
| sect-seqid | 50 / 0 / 0 | 0 / 50 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 45 / 5 / 0 | 33 / 17 / 0 |
| sect-hashid | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 0 / 0 / **50** | 50 / 0 / 0 | 41 / 9 / 0 | 16 / 33 / **1** |
| sect-wirenet | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 0 / 50 / 0 | 46 / 4 / 0 | 35 / 15 / 0 |
| sect-nettable | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 46 / 4 / 0 | 35 / 15 / 0 |
| sect-offsetidx | 50 / 0 / 0 | 1 / 49 / 0 | 0 / 50 / 0 | 0 / 50 / 0 | 47 / 3 / 0 | 43 / 7 / 0 | 35 / 15 / 0 |
| sect-blank | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 47 / 3 / 0 |
| sect-block | 50 / 0 / 0 | 50 / 0 / 0 | 48 / 2 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 46 / 4 / 0 | 50 / 0 / 0 |
| sect + merge driver | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 |

## Concurrent-edit merges (200-object design, 50 random trials each, `git merge-file`)

Cell = clean&correct / conflict / clean-but-WRONG (merged file parses to something other than both edits applied).

| format | move inst X ∥ param on inst Y | insert inst ∥ insert inst | insert wire ∥ delete wire | move inst X ∥ param on same X | rename net ∥ move wire on it | 5 wire moves ∥ 5 wire deletes | 10 moves ∥ 10 param edits |
|---|---|---|---|---|---|---|---|
| sexpr-1line | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 25 / 25 / 0 | 3 / 47 / 0 |
| sexpr-kicad | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 37 / 13 / 0 | 50 / 0 / 0 |
| kdl | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 25 / 25 / 0 | 50 / 0 / 0 |
| xschem | 49 / 1 / 0 | 0 / 50 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 0 / 50 / 0 | 28 / 22 / 0 | 7 / 43 / 0 |
| jsonl | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 25 / 25 / 0 | 3 / 47 / 0 |
| sect | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 25 / 25 / 0 | 3 / 47 / 0 |
| sect-seqid | 49 / 1 / 0 | 0 / 50 / 0 | 48 / 2 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 28 / 22 / 0 | 7 / 43 / 0 |
| sect-hashid | 45 / 5 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 0 / 3 / **47** | 50 / 0 / 0 | 17 / 33 / 0 | 0 / 50 / 0 |
| sect-wirenet | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 0 / 50 / 0 | 25 / 25 / 0 | 3 / 47 / 0 |
| sect-nettable | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 25 / 25 / 0 | 3 / 47 / 0 |
| sect-offsetidx | 50 / 0 / 0 | 1 / 49 / 0 | 0 / 50 / 0 | 0 / 50 / 0 | 46 / 4 / 0 | 18 / 32 / 0 | 3 / 47 / 0 |
| sect-blank | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 0 / 50 / 0 | 50 / 0 / 0 | 37 / 13 / 0 | 26 / 24 / 0 |
| sect-block | 50 / 0 / 0 | 50 / 0 / 0 | 49 / 1 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 25 / 25 / 0 | 50 / 0 / 0 |
| sect + merge driver | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 | 50 / 0 / 0 |
