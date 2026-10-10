# Schemify on-disk text format: measurements and recommendation

Status: research, 2026-10-10. Scope: one text syntax for `.shprim` (symbol only), `.shcomp` (symbol + schematic), `.shbench` (schematic + testbench code) and config files. The baseline is xschem's `.sch`/`.sym` format (`xschem-feature-inventory.md` §5). The relevant user pain points are in `xschem-complaints.md` §5–6: noisy diffs, quoting hell, and `tcleval` inside design data.

Hard requirements:

1. **Minimal diff, clean merges.** Moving an instance or wire, renaming a net, editing one parameter, or adding/deleting an object touches as few lines as possible. Independent concurrent edits merge in git without conflicts.
2. **Fast I/O and "know where things are".** A library browser or netlister gets a symbol's pins and parameters without parsing the schematic. Designs with 100k+ objects load fast.

## TL;DR

Recommendation: **line records with indented attributes** (benchmarked as `sect-block`).

- **Fixed section order.** Symbol interface first (`pin`, `param`, `netlist`), then symbol graphics, then the schematic, then testbench code.
- **One record per line.** An instance is a header line (`inst <id> <symref>`) followed by indented attribute lines: `at x y R90`, then one line per parameter. Wires, labels, pins and shapes are single lines.
- **Records sorted by (kind, id).** Each object has a **random 6-char base32 id** that is unique within the file.
- **Net names live only on labels and ports.** Connectivity is derived from geometry, as KiCad does.
- **Nothing derived is stored.** No offsets, counts, hashes or cached net names in the file, and no committed index or sidecar.
- **One escaping layer.** Values are bare words or `"quoted"` with `\" \\ \n \t`. There is no executable content.
- **Ship a semantic git merge driver** (`schemify merge-driver`, enabled through `.gitattributes`). It merged **350/350** concurrent-edit cases cleanly and correctly, at both design sizes tested.

Key numbers, on a 1M-object design unless noted:

| | Result |
|---|---|
| File size | 62 B/object, 10% above the single-line variant and about xschem's size. KiCad-style S-expressions and JSONL are 105–111. |
| Full parse | 327 ms on 1 thread and **59 ms on 16 threads** for the recommended layout. The single-line variant is 268 and 53 ms. At **100k objects: 34 ms** on 1 thread. |
| Symbol interface only | **5 µs at any file size**. xschem's order-free spec forces a full parse: 387 ms. |
| Find one record by id, no index | **19–26 µs** binary search, against 28 ms for read + scan. |
| Diff per edit | Move, param, delete: 1 line. Net rename: only the label lines (3 here, against xschem's 50). |
| Driverless git merge, 10 moves ∥ 10 param edits | **50/50 clean** on 200- and 1000-object designs. One-line-per-object formats: 3/50 and 35/50. |

## 1. Method

The benchmark is a standalone crate. It has no dependencies, all parsers are hand-written, and timings are release builds.

- **Crate:** `tools/fmtbench/` (≈1.7k lines; raw logs alongside).
- **Run:** `cargo run --release -- all`. Other modes are `perf`, `diff`, `merge` and `sample`.
- **Machine:** i9-14900HX (32 threads), Linux 7.0, rustc 1.95, page cache warm. Times are best-of-N.

**Synthetic design.** n objects: 35% instances, 45% wires, 12% labels, 8% text.

- **Coordinates:** grid-snapped, spread over √n·150 units.
- **Instances:** 5 symbol types, each with `name` plus 5 params, e.g. `w=1u l=0.18u nf=2 m=1 model=nfet_01v8`. Vsource `tran` holds a quoted value with spaces.
- **Text:** every 25th text contains a quote and a newline.
- **Symbol:** 8 pins, 6 params (one has an embedded quote), a SPICE binding, and 12 graphic lines.
- **Rename target:** net `VBIAS` has exactly 50 users, 47 wires and 3 labels.

**Fairness.** All formats parse into the same owned model (`String` per param key and value), so absolute parse times are allocation-dominated. A zero-copy row shows the floor for the lexing itself. Every format except `xschem` writes records grouped by kind and sorted by the same random id, so syntax is compared without ordering-policy confounds. The ordering policies are compared separately through the `sect-*` variants.

**Correctness.** A self-check asserts `write(parse(write(d))) == write(d)` and that symbol-only extraction returns the symbol, for every format.

**Formats.** One line per object unless stated:

| Name | What it is |
|---|---|
| `sexpr-1line` | KiCad S-expression syntax, `(inst (lib_id "nmos4") (at x y 90) (uuid "…") (prop "w" "1u") …)` |
| `sexpr-kicad` | Same tree pretty-printed the way KiCad does it: one child per line, tab indent, `)` closer |
| `kdl` | KDL v2 style: `inst "id" sym=nmos4 x= y= rot= {` with children `w "1u"` and `}`. Wires/labels/text are one node per line |
| `xschem` | xschem records (`N`, `C {sym} x y r f {props}`, `K`, `B 5`). Insertion order, no ids, `lab=` cached on wires, labels as `lab_pin` instances, two escaping layers |
| `jsonl` | One JSON object per line |
| `sect` | Proposed line records: `inst <id> <sym> x y R90 name=M1 w=1u …`, keyword per line, fixed section order, sorted by random id, net names on labels only |
| `sect-seqid` | `sect` with sequential ids (new object = max+1, so it lands at the end of its section) |
| `sect-hashid` | `sect` with ids = hash of the record content |
| `sect-wirenet` | `sect` with the net name on every wire (xschem-style) |
| `sect-nettable` | `sect` with a `net <id> <name>` table; wires and labels reference net ids |
| `sect-offsetidx` | `sect` plus a header line of byte offsets for each section (the "O(1) index" idea) |
| `sect-blank` | `sect` with an empty line after every record |
| `sect-block` | **Recommended.** `sect` with instances written as a header line plus indented `at` and param lines |

**Edits and merges.** Each edit is applied to the in-memory model, and the whole file is re-serialized with the canonical writer, as an editor would save.

- **Diffs:** counted with GNU `diff`.
- **Merges:** `git merge-file -p A base B`, 50 random trials per scenario. The same edits are used for every format.
- **Grading:** a clean merge is graded by parsing it and comparing it with the canonical serialization of "both edits applied". A clean merge that parses to something else counts as **clean-but-WRONG**.

## 2. Results

### 2.1 Size and speed

Single-thread parse into the owned model.

| format | B/obj | lines/obj | parse 1k ms | parse 10k ms | parse 100k ms | parse 1M ms | write 1M ms | symbol-only, 1M file |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| sexpr-1line | 105 | 1.00 | 0.41 | 3.88 | 39.7 | 401 | 343 | 5 µs |
| sexpr-kicad | 111 | 6.54 | 0.41 | 3.97 | 39.6 | 410 | 273 | 5 µs |
| kdl | 77 | 3.45 | 0.38 | 3.63 | 36.7 | 354 | 252 | 4 µs |
| xschem | 61 | 1.00 | 0.29 | 2.68 | 27.7 | 281 | 243 | **387 ms** (full parse required) |
| jsonl | 105 | 1.00 | 0.73 | 7.25 | 71.9 | 712 | 268 | 12 µs |
| sect | **56** | 1.00 | 0.29 | 2.74 | 27.8 | **268** | 274 | 5 µs |
| sect-seqid | 56 | 1.00 | 0.25 | 2.35 | 24.2 | 235 | 121 | 5 µs |
| sect-hashid | 56 | 1.00 | 0.29 | 2.74 | 27.7 | 267 | 317 | 5 µs |
| sect-wirenet | 60 | 1.00 | 0.30 | 2.88 | 29.1 | 284 | 290 | 5 µs |
| sect-nettable | 61 | 1.08 | 0.34 | 3.14 | 32.2 | 321 | 357 | 5 µs |
| sect-offsetidx | 56 | 1.00 | 0.29 | 2.74 | 28.9 | 268 | 275 | 5 µs |
| sect-blank | 57 | 2.00 | 0.30 | 2.84 | 28.9 | 277 | 275 | 5 µs |
| **sect-block** | 62 | 3.45 | 0.36 | 3.37 | 34.0 | 327 | 278 | 5 µs |

Other loading paths, measured on 1M objects in `sect` (56 MB):

| Path | Result |
|---|---|
| Parallel parse (lines are self-describing, so chunks split at any LF that precedes a column-0 record) | 1 / 2 / 4 / 8 / 16 threads: 326 / 169 / 95 / 73 / **53 ms**. `sect-block`, same file content: 391 / 241 / 147 / 91 / **59 ms**. The 1-thread figures include chunk concatenation. |
| Zero-copy loader (values borrowed from the read buffer, keys interned) | **177 ms** on 1 thread (319 MB/s). This is the format's own lexing cost; the rest of the 268 ms is allocation in the owned model. |
| Lookup one record by id, with no index: binary search over byte offsets with `pread` on the (kind, id)-sorted file | **17–19 µs**. In `sect-block`, skipping indented lines: **26 µs**. Over 200 lookups and two runs. |
| Lookup one record by id: read the whole file, then substring search | 28 ms |

Symbol-only extraction reads the first 8 KB and stops at the first schematic keyword: 4–12 µs for every fixed-order format at every size. The xschem spec allows records in any order (§5.1/§5.9 of the inventory), so a reader must parse the whole file: 0.35 ms at 1k objects, 37 ms at 100k and 387 ms at 1M.

Writing 1M objects takes 274 ms for random-id formats and 121 ms for `sect-seqid`. The difference is random-order memory traversal when emitting in id order, not the format. An editor that keeps objects in id order (a sorted `Vec` or a `BTreeMap`) gets the faster number.

### 2.2 Diff size per edit

10k-object design, `diff` lines removed/added. "(n hunks)" is shown when the change is not contiguous.

| format | move inst | move wire | rename net (50 users) | change 1 param | insert inst | delete wire |
|---|---:|---:|---:|---:|---:|---:|
| sexpr-1line | -1 +1 | -1 +1 | -3 +3 (3 hunks) | -1 +1 | +1 | -1 |
| sexpr-kicad | -1 +1 | -1 +1 | -3 +3 (3 hunks) | -1 +1 | +12 | -4 |
| kdl | -1 +1 | -1 +1 | -3 +3 (3 hunks) | -1 +1 | +8 | -1 |
| xschem | -1 +1 | -1 +1 | **-50 +50 (48 hunks)** | -1 +1 | +1 | -1 |
| jsonl | -1 +1 | -1 +1 | -3 +3 (3 hunks) | -1 +1 | +1 | -1 |
| sect | -1 +1 | -1 +1 | -3 +3 (3 hunks) | -1 +1 | +1 | -1 |
| sect-seqid | -1 +1 | -1 +1 | -3 +3 | -1 +1 | +1 | -1 |
| sect-hashid | -1 +1 (2 hunks) | -1 +1 (2 hunks) | -3 +3 (6 hunks) | -1 +1 (2 hunks) | +1 | -1 |
| sect-wirenet | -1 +1 | -1 +1 | **-50 +50 (50 hunks)** | -1 +1 | +1 | -1 |
| sect-nettable | -1 +1 | -1 +1 | **-1 +1** | -1 +1 | +1 | -1 |
| sect-offsetidx | -1 +1 | -1 +1 | -4 +4 (4 hunks) | -2 +2 (2 hunks) | -1 +2 (2 hunks) | -2 +1 (2 hunks) |
| sect-blank | -1 +1 | -1 +1 | -3 +3 (3 hunks) | -1 +1 | +2 | -2 |
| **sect-block** | -1 +1 | -1 +1 | -3 +3 (3 hunks) | -1 +1 | +8 | -1 |

### 2.3 Concurrent-edit merges

Cell = clean&correct / conflict / **clean-but-WRONG**, out of 50 trials. `∥` separates branch A's edit from branch B's.

**1000-object design**

| format | move X ∥ param Y | insert ∥ insert | insert wire ∥ delete wire | move X ∥ param on same X | rename net ∥ move wire on it | 5 wire moves ∥ 5 wire deletes | 10 moves ∥ 10 param edits |
|---|---|---|---|---|---|---|---|
| sexpr-1line | 50/0/0 | 50/0/0 | 48/2/0 | 0/50/0 | 50/0/0 | 46/4/0 | 35/15/0 |
| sexpr-kicad | 50/0/0 | 50/0/0 | 48/2/0 | 50/0/0 | 50/0/0 | 48/2/0 | 50/0/0 |
| kdl | 50/0/0 | 50/0/0 | 48/2/0 | 50/0/0 | 50/0/0 | 46/4/0 | 50/0/0 |
| xschem | 50/0/0 | **0/50/0** | 49/1/0 | 0/50/0 | **0/50/0** | 45/5/0 | 33/17/0 |
| jsonl | 50/0/0 | 50/0/0 | 48/2/0 | 0/50/0 | 50/0/0 | 46/4/0 | 35/15/0 |
| sect | 50/0/0 | 50/0/0 | 48/2/0 | 0/50/0 | 50/0/0 | 46/4/0 | 35/15/0 |
| sect-seqid | 50/0/0 | **0/50/0** | 49/1/0 | 0/50/0 | 50/0/0 | 45/5/0 | 33/17/0 |
| sect-hashid | 50/0/0 | 50/0/0 | 50/0/0 | **0/0/50** | 50/0/0 | 41/9/0 | 16/33/**1** |
| sect-wirenet | 50/0/0 | 50/0/0 | 48/2/0 | 0/50/0 | **0/50/0** | 46/4/0 | 35/15/0 |
| sect-nettable | 50/0/0 | 50/0/0 | 48/2/0 | 0/50/0 | 50/0/0 | 46/4/0 | 35/15/0 |
| sect-offsetidx | 50/0/0 | **1/49/0** | **0/50/0** | 0/50/0 | 47/3/0 | 43/7/0 | 35/15/0 |
| sect-blank | 50/0/0 | 50/0/0 | 48/2/0 | 0/50/0 | 50/0/0 | 48/2/0 | 47/3/0 |
| **sect-block** | 50/0/0 | 50/0/0 | 48/2/0 | **50/0/0** | 50/0/0 | 46/4/0 | **50/0/0** |
| **sect + merge driver** | 50/0/0 | 50/0/0 | 50/0/0 | 50/0/0 | 50/0/0 | 50/0/0 | **50/0/0** |

**200-object design** (a typical analog cell sheet, 70 instances and 90 wires)

| format | move X ∥ param Y | insert ∥ insert | insert wire ∥ delete wire | move X ∥ param on same X | rename net ∥ move wire on it | 5 wire moves ∥ 5 wire deletes | 10 moves ∥ 10 param edits |
|---|---|---|---|---|---|---|---|
| sexpr-1line | 50/0/0 | 50/0/0 | 49/1/0 | 0/50/0 | 50/0/0 | 25/25/0 | 3/47/0 |
| sexpr-kicad | 50/0/0 | 50/0/0 | 49/1/0 | 50/0/0 | 50/0/0 | 37/13/0 | 50/0/0 |
| kdl | 50/0/0 | 50/0/0 | 49/1/0 | 50/0/0 | 50/0/0 | 25/25/0 | 50/0/0 |
| xschem | 49/1/0 | 0/50/0 | 48/2/0 | 0/50/0 | 0/50/0 | 28/22/0 | 7/43/0 |
| jsonl | 50/0/0 | 50/0/0 | 49/1/0 | 0/50/0 | 50/0/0 | 25/25/0 | 3/47/0 |
| sect | 50/0/0 | 50/0/0 | 49/1/0 | 0/50/0 | 50/0/0 | 25/25/0 | **3/47/0** |
| sect-seqid | 49/1/0 | 0/50/0 | 48/2/0 | 0/50/0 | 50/0/0 | 28/22/0 | 7/43/0 |
| sect-hashid | 45/5/0 | 50/0/0 | 50/0/0 | 0/3/**47** | 50/0/0 | 17/33/0 | 0/50/0 |
| sect-wirenet | 50/0/0 | 50/0/0 | 49/1/0 | 0/50/0 | 0/50/0 | 25/25/0 | 3/47/0 |
| sect-nettable | 50/0/0 | 50/0/0 | 49/1/0 | 0/50/0 | 50/0/0 | 25/25/0 | 3/47/0 |
| sect-offsetidx | 50/0/0 | 1/49/0 | 0/50/0 | 0/50/0 | 46/4/0 | 18/32/0 | 3/47/0 |
| sect-blank | 50/0/0 | 50/0/0 | 49/1/0 | 0/50/0 | 50/0/0 | 37/13/0 | 26/24/0 |
| **sect-block** | 50/0/0 | 50/0/0 | 49/1/0 | **50/0/0** | 50/0/0 | 25/25/0 | **50/0/0** |
| **sect + merge driver** | 50/0/0 | 50/0/0 | 50/0/0 | 50/0/0 | 50/0/0 | 50/0/0 | **50/0/0** |

The "10 moves ∥ 10 param edits" scenario has branch A move 5 instances and 5 wires, and branch B change a param on 10 random instances.

### 2.4 What the numbers say

1. **Syntax barely matters for diff size.** Any format with stable ids, one record per line, and a deterministic writer gives 1-line diffs for move and param edits. The differences come from *policy*: where net names live, how ids are made, and whether derived data (offsets) is stored.
2. **Git conflicts on adjacent lines.** Verified on a 5-line file: changes on lines 2 and 3 conflict, while changes on lines 2 and 4 merge. With one object per line, two edits to neighbouring records in sort order conflict, and so do two edits to different fields of one object. This dominates driverless merges on small sheets: 3/50 for `sect` at 200 objects. Layouts that put an unchanged line between independently edited fields (`sect-block`, KDL, KiCad multi-line) merge 50/50 there. Blank separator lines only half work (26/50), because diff slides hunks across the identical blank lines.
3. **Insertion policy decides insert∥insert.** Appending at the end of a section (sequential ids, or xschem's insertion order) makes every pair of concurrent inserts conflict (0/50). With random ids and a sorted section, new records land at random positions (50/50).
4. **Content-hash ids silently corrupt.** The id changes on every edit. Two branches that each edit object X both delete the same old line (an identical change, so git takes it) and insert the new line in different places. The result is two copies of X with no conflict reported: 50/50 clean-but-WRONG.
5. **Byte-offset headers dirty every structural edit.** One extra hunk per insert, delete or param-length change, and insert∥insert or insert∥delete then always conflict (0–1/50). That kills the "O(1) index in the file" idea.
6. **Net names on wires are the worst net storage.** Rename costs 50 lines in 48 hunks and conflicts with any concurrent edit of any wire on that net (0/50). Labels-only costs one line per label (3 here). A net table costs 1 line but adds an indirection (§3.4).
7. **A semantic merge driver removes every text-level artifact.** It reaches 350/350 at both sizes, including same-object different-field edits and adjacent-record edits. It is about 80 lines of Rust (`merge3` in the bench's `model.rs`).
8. **Speed is not a differentiator between line formats**, but early exit and order are. Into an owned model, every format parses at 190–280 MB/s except JSONL (148 MB/s). JSONL is 2.6× slower than `sect` (every token quoted; generic value tree) and S-expressions 1.5×. The block layout costs +22% parse and +10% size over `sect`. What matters is (a) a guaranteed section order (5 µs against 387 ms for symbol-only), and (b) self-describing lines, which give parallel loading (53–59 ms per 1M objects on 16 threads) and index-free binary search (17–26 µs).

## 3. Design decisions

### 3.1 Layout: line records with indented attributes

Every line is `word value…`. A line in column 0 starts a **record**. An indented line is an **attribute** of the preceding record. Fixed-arity geometry goes on the record line: `wire`, `label`, `port`, `pin`, `line`, `rect`, `poly`. Open-ended key/value data goes on attribute lines: instance placement and parameters, parameter metadata, netlist templates.

Why block layout for instances, not one-line:

- **Driverless merges.** It is the only measured change that fixes same-object different-field merges and adjacent-instance edit merges without a merge driver: 50/50 against 0/50 and 3/50. This matters for GitHub web merges, where custom drivers do not run.
- **Real PDK devices are wide.** sky130 MOS instances have 10–20 params (`W L nf mult ad as pd ps nrd nrs sa sb sd model`). One line would be over 250 characters. One param per line is readable, greppable, and diffs crisply.
- **The cost is small:** +10% size and +22% parse. At 100k objects that is 34 ms against 28 ms.
- **The useful properties survive.** Chunk boundaries for parallel loading are "LF followed by a column-0 character", and binary search skips indented lines (26 µs).

Wires stay single-line. They are 45% of objects, have no open-ended fields, and a block layout would not fix their adjacency conflicts anyway: deleting record k+1 still touches the line after record k's last line. The merge driver covers them.

### 3.2 Ordering: fixed sections, then (kind, id)

Section rank, enforced by the reader (a lower-rank record after a higher-rank one is a parse error):

| Rank | Contents | Records |
|---|---|---|
| 0 | header | `schemify <ver> <kind>` + attributes |
| 1 | symbol interface | `pin`, `param`, `netlist`, in declaration order (pin order = port order, so it is semantic) |
| 2 | symbol graphics | `line`, `rect`, `arc`, `circle`, `poly`, `stext`, in draw order |
| 3 | schematic | `inst`, `wire`, `label`, `port`, `noconn`, `text`. Each kind contiguous, sorted by id |
| 4 | testbench | `code` blocks, sorted by name |

What this gives:

- A library browser or netlister reads until the first rank-2/3 keyword: one 8 KB read, 5 µs.
- Within rank 3, the canonical writer sorts by id. The reader accepts any order within a kind, so hand edits stay legal; the next save canonicalizes.
- On canonical files, records can be found by byte-level binary search with no index. `grep -n '^inst 0gsrp0'` also works for humans and tools.

### 3.3 Object ids

- **Format:** 6 characters of lowercase Crockford base32 (`0-9a-z` without `i l o u`), so 30 bits. Fixed width makes string order equal numeric order.
- **Scope:** unique **per file** across kinds. They are not global UUIDs: KiCad's 36-char UUIDs would add about 30 B/object (about +50% here) and buy nothing at file scope.
- **Generation:** the editor draws a random id and redraws if it already exists in the file. Paste and duplicate get fresh ids. Undo restores the old ids.
- **Importers:** use a *deterministic* initial id, e.g. `hash(source path, record ordinal)`, so re-importing an unchanged xschem library yields an empty diff. The id is stored and never recomputed. Recomputing from content is exactly the rejected content-hash scheme.

**Collisions.** Local collisions are impossible because of the redraw check. Across branches, both adding k objects: P ≈ 1−exp(−k²/2³⁰), which is 9e-6 at k=100, 9e-4 at k=1000, and 9% at k=10,000. A collision is detected, not silent. Ids are sorted, so duplicates are adjacent and checking costs nothing. The loader re-ids the later duplicate with a warning, and the merge driver re-ids B's addition. Ids are file-internal (cross-file references use instance names and net names), so re-iding is harmless. If scripted generation of 10k+ objects per branch becomes common, the writer can switch to 8 characters (+2 B/obj); readers accept 1–13 characters and compare bytewise.

**Rejected id schemes:**

- **Sequential:** insert∥insert 0/50, and every concurrent insert collides on id.
- **Content hash:** silent duplication (50/50 clean-but-WRONG), and references break on every edit.
- **Instance name as key:** renames move lines, and wires have no names.

### 3.4 Net names

Names are stored only where the user typed them: `label` (local or global) and `port` (bound to a symbol pin). Wires carry geometry only. Connectivity is derived on load with a union-find over wire endpoints, pin positions and label anchors, which is linear-ish and cheap next to parsing.

- **Rename** = the label lines for that net (3 lines in the test). Merges with any concurrent wire edit.
- **Wire-cached names** (xschem `lab=`) are rejected: 50 lines and 48 hunks per rename, rename∥move-wire 0/50, and the cache can go stale against geometry.
- **A net table** (`net <id> <name>`, with wires holding net ids) gives a 1-line rename but is rejected:
  - it duplicates what geometry already says, so the file can contradict itself;
  - connecting two nets or splitting one rewrites every wire reference, so routine wiring produces large diffs;
  - labels become ids, which hurts hand-editing.

  Store a net table only in an in-memory or derived cache.

### 3.5 No index in the file; no sidecar in v1

**Nothing derived is committed.** No offsets, counts, sizes or hashes appear in the file. Every one of them dirties a header line on unrelated edits (§2.4.5).

**A sidecar cache is not justified by the numbers:**

| Need | Measured |
|---|---|
| Symbol interface | 5 µs |
| Lookup by id | 19 µs |
| Full load, 100k objects | 34 ms on 1 thread |
| Full load, 1M objects | about 60 ms on 16 threads |

If a project-wide cache is needed later (e.g. "which designs use symbol X", or connectivity for 10k files), put it in `.schemify/cache/`. It must be gitignored and keyed by path + size + mtime, validated by content hash, rebuildable at any time, and never authoritative.

### 3.6 Merge driver

The driver works without any extra file in the repo except `.gitattributes`.

1. Commit `.gitattributes`: `*.shprim *.shcomp *.shbench merge=schemify`.
2. Each clone needs `git config merge.schemify.driver "schemify merge-driver %O %A %B %P"`. Git never takes this from the repo, for security. `schemify git-setup` writes it, and the app warns when `.gitattributes` names the driver but it is not configured.

The algorithm:

- Parse O, A and B. Key records by (kind, id), and by name for `pin`, `param`, `netlist` and `code`.
- Merge 3-way per field: placement; each param; wire geometry; label position and text.
- Handle adds, deletes and modify/delete. Re-id new∥new id collisions.
- Write canonical output. Only a genuinely conflicting record is wrapped in standard conflict markers, and then the driver exits 1.
- Measured: 350/350 clean and correct.

Where the driver does not run (GitHub web merge, fresh clones), the text format still merges as in the `sect-block` rows. `schemify resolve <file>` re-runs the semantic merge from git's index stages `:1:`, `:2:` and `:3:` for files that were text-merged with conflicts.

### 3.7 Values, numbers and safety

- **Values.** A value is a *bare word*: any run of non-whitespace characters, excluding `"`, `\` and control characters, and not starting with `#` or `|`. Otherwise it is a *quoted string* with escapes `\" \\ \n \t \u{…}`. There is exactly one escaping layer: no `{}` braces, no `;` separators, no second attribute-string layer. The writer quotes only when needed, so `w 1u` and `model nfet_01v8` stay bare.
- **Numbers.**
  - Grammar: `-?digits(.digits)?`. The canonical writer emits the shortest round-trip form, with integers lacking `.0`.
  - Coordinates: xschem units with y down, so import is 1:1.
  - Orientation: `R0 R90 R180 R270`, clockwise on screen. An `M` prefix (`MR90`) mirrors before rotating, matching xschem's flip.
- **Canonical writer.**
  - UTF-8, LF, final newline, no trailing spaces.
  - One blank line between kinds.
  - Wire endpoints normalized so that (x1,y1) < (x2,y2).
  - Attribute order: `at` first, then params in the symbol's declaration order, then unknown params by key.
  - The same model always produces the same bytes.
- **Comments.** Full-line `#` comments are legal for hand-editing. They attach to the next record and are preserved through saves.
- **Forward compatibility.** Unknown attribute lines are preserved verbatim. An unknown record keyword in a known section is an error, unless the header version is newer than the reader's, in which case it is kept verbatim.
- **No executable content.** No `tcleval`, no generator symrefs (`gen.tcl(args)`), no launchers. Parameter expressions, defined separately with the netlist-template language, are pure and sandboxed. Commands to run simulators exist only in user config, never in design files.
- **Symbol references.** `lib:cell` resolves through `lib` entries in config, which fixes xschem's flat `XSCHEM_LIBRARY_PATH` collisions. A relative path like `./ota5t` tries `.shcomp`, then `.shprim`. There are no absolute paths and no embedded symbols.

## 4. Grammar sketch

```abnf
file      = header *line
header    = "schemify" WS version WS kind EOL          ; line 1, always
version   = 1*DIGIT
kind      = "shprim" / "shcomp" / "shbench" / "config"
line      = blank / comment / record / attr / verbatim
blank     = *WSP EOL
comment   = *WSP "#" *CHAR EOL                         ; whole-line only; attaches to next record
record    = word *(WS value) *WSP EOL                  ; column 0
attr      = 1*WSP word *(WS value) *WSP EOL            ; indented: belongs to previous record
verbatim  = 1*WSP "|" [SP *CHAR] EOL                   ; indented raw text line (code blocks)
word      = ALPHA *(ALPHA / DIGIT / "_" / "-" / "." / ":")
value     = bare / quoted
bare      = bstart *bchar          ; bchar: any char except WSP, CR, LF, DQUOTE, "\", controls
bstart    = bchar - ("#" / "|")
quoted    = DQUOTE *(qchar / escape) DQUOTE
escape    = "\" (DQUOTE / "\" / "n" / "t" / "u{" 1*6HEXDIG "}")
WS        = 1*WSP                                      ; writer: exactly one SP
EOL       = LF / CRLF                                  ; writer: LF
```

Record schemas. Positional fields are on the record line. Indented attributes are listed under "Attributes"; any record may also carry unknown attributes, which are preserved.

**Rank 1: symbol interface**

| Record | Attributes |
|---|---|
| `pin <name> <dir> <x> <y>` (`dir` = `in` / `out` / `inout`) | `number`, `bus` |
| `param <name> <default>` | `kind`, `unit`, `min`, `max`, `choices`, `computed`, `doc` |
| `netlist <tool>` | `format`, `prefix`, `once`, `ignore` |

**Rank 2: symbol graphics**

| Record | Attributes |
|---|---|
| `line <x1> <y1> <x2> <y2>` | `layer`, `dash`, `width` |
| `rect <x1> <y1> <x2> <y2>` | `layer`, `fill` |
| `arc <cx> <cy> <r> <start> <sweep>` | `layer`, `fill` |
| `circle <cx> <cy> <r>` | `layer`, `fill` |
| `poly <x> <y> <x> <y> …` | `layer`, `fill`, `closed` |
| `stext <x> <y> <orient> <text>` | `size`, `hcenter`, `vcenter`, `role` |

**Rank 3: schematic**

| Record | Attributes |
|---|---|
| `inst <id> <symref>` | `at <x> <y> <orient>`, then `<param> <value>` lines |
| `wire <id> <x1> <y1> <x2> <y2>` | `bus` |
| `label <id> <x> <y> <orient> <net>` | `global` |
| `port <id> <x> <y> <orient> <pin>` | |
| `noconn <id> <x> <y>` | |
| `text <id> <x> <y> <orient> <text>` | `size` |

**Rank 4: testbench**

| Record | Attributes |
|---|---|
| `code <name> <lang>` | `|` verbatim lines |

**Config files**

| Record | Purpose |
|---|---|
| `lib <name> <path>` | library location |
| `set <key> <value>` | setting |
| `key <chord> <command>` | key binding |
| `sim <name>` | simulator, with attributes `path`, `args`, … |

Reader algorithm: split on LF (memchr). A non-indented line starts a record; dispatch on the first word through a `match`. An indented line goes to the current record. Values are borrowed from the buffer unless they contain escapes. Every line is self-describing, so a file can be cut at any `LF + non-WSP` boundary and parsed in parallel, and a sorted file can be binary-searched by (rank, kind, id).

## 5. Examples

### 5.1 Annotated `.shcomp` (5-transistor OTA)

```text
schemify 1 shcomp
  doc "5-transistor OTA, NMOS input pair"
  pdk sky130

# --- rank 1: symbol interface. A library browser or netlister stops reading here (5 µs). ---
# pin <name> <dir> <x> <y>; file order = port order
pin INP in -80 -20
pin INN in -80 20
pin OUT out 80 0
pin IBIAS in -80 40
pin VDD inout 0 -60
pin VSS inout 0 60
param Wp 2
  unit um
  doc "input pair width"
param Lp 0.5
  unit um
netlist spice
  format subckt

# --- rank 2: symbol graphics, draw order ---
poly -60 -50 60 0 -60 50 -60 -50
line -80 -20 -60 -20
line -80 20 -60 20
line 60 0 80 0
stext -55 -28 R0 "+"
stext -55 12 R0 "-"

# --- rank 3: schematic. Each kind contiguous, sorted by id. ---
# Moving M2 changes only its "at" line; editing W changes only the W line.
# Those two lines are not adjacent, so both edits merge even without the driver.
inst 0gsrp0 sky130:nfet_01v8
  at -60 -300 R0
  name M1
  W 2
  L 0.5
  nf 2
inst 1j446r sky130:nfet_01v8
  at 120 -300 MR0
  name M2
  W 2
  L 0.5
  nf 2
inst 4szjc8 sky130:pfet_01v8
  at -60 -480 R0
  name M3
  W 4
  L 1
inst 4tyd1p sky130:pfet_01v8
  at 120 -480 MR0
  name M4
  W 4
  L 1
inst 9vdm2q sky130:nfet_01v8
  at 30 -160 R0
  name M5
  W 4
  L 1

# Wires are geometry only: no net names, so renaming a net never touches them.
wire 0d2k1m -40 -450 100 -450
wire 3fk2a1 -40 -270 -40 -190
wire 7q3vxa 100 -270 100 -190
wire b8nq0t -40 -190 100 -190
wire h2w9cs 100 -430 100 -330

# Net names live only here. Renaming "tail" is a 1-line diff.
label 2ma7xe 30 -190 R0 tail
label kq1p4z -40 -380 R0 mirror

# A port connects a schematic net to the rank-1 pin of the same name.
port 5c0d2r -120 -300 R0 INP
port 8tj4wh 200 -300 R0 INN
port c7rm1e 140 -380 R0 OUT
port fa3k0q -60 -160 R0 IBIAS
port mx0k2p 30 -560 R90 VDD
port qz81ty 30 -100 R270 VSS

text r0m2ve -150 -620 R0 "Gm stage: keep M1/M2 common-centroid"
```

### 5.2 `.shprim` (PDK device; fields from `pdk-xschem-devices.md` §5.2)

```text
schemify 1 shprim
  library sky130_fd_pr
  category nmos
  prefix X
  provenance "xschem 3.4.4 sky130_fd_pr/nfet_01v8.sym"

pin D inout 20 -30
pin G in -20 0
pin S inout 20 30
pin B in 20 0
param L 0.15
  kind number
  unit um
param W 1
  kind number
  unit um
param nf 1
  kind int
param ad 0
  computed "int((nf+1)/2)*W/nf*0.29"
param model nfet_01v8
  choices nfet_01v8 nfet_01v8_lvt
netlist spice
  format "X{name} {D} {G} {S} {B} sky130_fd_pr__{model} L={L} W={W} nf={nf} ad={ad}"

line 20 -30 20 -15
line 20 15 20 30
line 4 -15 4 15
line -20 0 4 0
stext 25 -25 R0 "{name}"
  size 0.2
```

The `{…}` placeholders stand for the netlist template syntax, which is a separate decision. The format only guarantees that the template is a single string with one escaping layer.

### 5.3 `.shbench`

```text
schemify 1 shbench
  doc "OTA open-loop AC gain"

inst 2kq9vd ./ota5t
  at 0 0 R0
  name X1
  Wp 4
inst 6pz0rt core:vsource
  at -300 100 R0
  name V1
  dc 1.8

wire 5mhv0q -300 60 -300 -60

label 0zx2fh 80 0 R0 out

code control spice
  | .control
  |   ac dec 20 1 1G
  |   meas ac gbw when vdb(out)=0
  | .endc
```

Each `|` line is one line of code, so editing the testbench script diffs line by line.

### 5.4 Config (same syntax)

```text
schemify 1 config
# Hand-written. The app edits single lines in place (set/key) and never rewrites the file,
# so comments survive. ${VAR} expands from the environment; nothing is executed while loading.
lib core ./lib/core
lib sky130 ${PDK_ROOT}/sky130A/libs.tech/schemify
set grid 10
set snap 5
key ctrl+s file.save
key w wire.draw
sim ngspice
  path ngspice
  args -b -r {raw} {netlist}
```

## 6. Rejected alternatives

| Option | Why not (measured) |
|---|---|
| **KiCad-style S-expressions**, multi-line (`sexpr-kicad`) | Merges as well as the block layout, but costs 111 B/obj (1.8×), 6.5 lines/object, and 410 ms/1M parse (1.25×). Paren nesting makes the per-line structure stateful, and a 4-line `(wire …)` block with identical `(wire` / `)` lines makes diff hunks slide (37/50 on wire edits at 200 objects). |
| **S-expressions, one line** | All the cost of S-expression syntax with the merge behaviour of `sect`. |
| **KDL v2** | Closest in spirit and merges as well (50/50). But it is 77 B/obj; the full KDL v2 spec (raw strings, multi-line strings, slashdash, type annotations, number forms) needs a real parser or a crate dependency; and hand-written KDL can contain constructs a subset parser rejects. A one-page grammar of our own gets the same layout benefits. |
| **xschem-like records** | No ids, so identity is file position and insert∥insert merges 0/50. Order-free records forbid early exit (387 ms for a symbol on 1M objects). `lab=` on wires gives 50-line renames and 0/50 merges. Two escaping layers ("quoting hell"), plus `tcleval`. It stays an **import** format only. |
| **JSON Lines** | 105 B/obj, 2.6× slower parse (712 ms/1M), every key and value quoted, multi-line text escaped onto one line. Merge behaviour equals `sect`. The only gain is off-the-shelf tooling (`jq`), which a `schemify dump --json` command gives without making JSON the storage format. |
| **Byte-offset index header** | Every insert, delete or param-length change rewrites the header (+1 hunk). insert∥insert 1/50 clean and insert∥delete 0/50. Binary search on sorted records already gives 19 µs lookups without it. |
| **Sequential ids** | insert∥insert 0/50, and concurrent inserts reuse the same id. |
| **Content-hash ids** | 50/50 clean-but-WRONG (duplicated objects), and the id changes on every edit. |
| **Net names on wires** | 50-line renames, rename∥wire-edit 0/50, and a stale cache. |
| **Net table** | 1-line renames, but redundant with geometry, large diffs on connect/split, and ids instead of names in labels. Better kept as a derived in-memory structure. |
| **One line per object everywhere** (`sect`) | Smallest and fastest (56 B/obj, 268 ms/1M), but driverless merges conflict on adjacent records and on same-object edits (3/50 at 200 objects). Still the right choice for small fixed-arity records (wires, labels, pins). |
| **Blank line after every record** | Halves line density but only partially fixes adjacency (26/50 at 200 objects), because diff slides hunks across the identical blank lines. |
| **Committed index or binary cache, or a sidecar in v1** | Not needed at the measured speeds. If added later, it must be gitignored, rebuildable and non-authoritative (§3.5). |
| **Global UUIDs (KiCad)** | +30 B/obj (about +50%), with no benefit when references are file-scoped. |

## 7. Open questions

- **Instance parameters: all params, or only overrides?** Storing only values that differ from symbol defaults makes files smaller, but a PDK update to a default silently changes old designs. The lean: store every netlist-relevant param that the user set or that came from the template at placement time (xschem and Cadence behaviour), and skip display-only defaults.
- **Coordinate unit.** Keeping xschem units (snap 10, decimals allowed) makes import trivial. An integer database unit (e.g. 1/16 of an xschem unit) would remove decimals entirely. Decide together with the xschem importer, since xschem text offsets such as `3.90625` need 1/32-unit resolution.
- **Symbol graphics and the merge driver.** Graphics have no ids, so concurrent edits to one symbol's drawing conflict at the record level. That is acceptable for v1, because symbols usually have a single author.
- **Template / expression syntax** inside `netlist format` and `computed`: deferred to the netlist-template research (`xschem-complaints.md` §5).
