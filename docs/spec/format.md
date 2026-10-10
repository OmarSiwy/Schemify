# File format v1 (normative)

Implements D9, D10, D11, D13, D17, D19, D20, D29, D30, D33, D35, D40. Rationale and measurements: [../research/file-format.md](../research/file-format.md). Where this spec and the research doc differ, this spec wins.

## 1. Files

| File | Header kind | Purpose |
|---|---|---|
| `<cell>.shprim` | `shprim` | symbol + ≥1 leaf impl (device binding, Verilog-A, Verilog, SPICE text) |
| `<cell>.shcomp` | `shcomp` | symbol + exactly one schematic impl + 0..N leaf impls + docs |
| `<cell>.shbench` | `shbench` | schematic + docs + bench tables; no symbol |
| `schemify.project` | `project` | project manifest: libraries, plugins, settings |
| `schemify.lock` | `lock` | resolved sources + content hashes (written by the tool only) |
| `config` (user, `$XDG_CONFIG_HOME/schemify/config`) | `config` | settings, keymap, simulators, plugin order |

A library is a directory; one cell per file; the cell name is the file stem. Two files with the same stem and different kinds in one library is an error (`E020`).

`ponytail:` v1 allows exactly one schematic impl per `.shcomp`. Multiple schematic variants would need impl-scoped rank-3 records; add when a user needs it.

## 2. Lexical layer

```abnf
file      = header *line
header    = "schemify" WS version WS kind EOL          ; line 1, always
version   = 1*DIGIT                                     ; this spec: 1
kind      = "shprim" / "shcomp" / "shbench" / "project" / "lock" / "config"
line      = blank / comment / record / attr / verbatim
blank     = *WSP EOL
comment   = *WSP "#" *CHAR EOL                          ; whole-line only; attaches to the next record
record    = word *(WS value) *WSP EOL                   ; starts in column 0
attr      = 1*WSP word *(WS value) *WSP EOL             ; indented: belongs to the previous record
verbatim  = 1*WSP "|" [SP *CHAR] EOL                    ; raw text line, content after "| " (or after "|" if line is exactly "|")
word      = ALPHA *(ALPHA / DIGIT / "_" / "-" / "." / ":")
value     = bare / quoted
bare      = bstart *bchar                                ; bchar: any char except WSP, CR, LF, DQUOTE, "\", C0 controls, DEL
bstart    = bchar - ("#" / "|")
quoted    = DQUOTE *(qchar / escape) DQUOTE              ; qchar: any char except DQUOTE, "\", CR, LF
escape    = "\" (DQUOTE / "\" / "n" / "t" / "u{" 1*6HEXDIG "}")
WS        = 1*WSP
EOL       = LF / CRLF
```

- Encoding UTF-8. A BOM or invalid UTF-8 is an error.
- **Writer canonical form:** LF only, final newline, single space separators, attributes indented by exactly two spaces, verbatim lines as `  | text` (or `  |` for an empty line), no trailing spaces, one blank line between record kinds, quoting only when a value is not a legal `bare`. The same model always produces the same bytes.
- Comments attach to the next record and are written back immediately before it (indentation dropped). Comments at end of file attach to EOF.
- Line length is unbounded; a file is capped at 2^30 bytes (`E021`).

## 3. Numbers, coordinates, orientation

- **Number literal (file level):** `-?digits(.digits)?` with no exponent and no SI suffix. Anything else in a numeric position is an error. (SI suffixes belong to expressions, §6.)
- **Coordinates** are written in xschem units, y pointing down, with at most 3 fractional digits. In memory they are `i32` milli-units (`value × 1000`, exact). A coordinate needing more digits or outside `[-2^31, 2^31-1]` milli-units is an error (`E022`). Writer: shortest decimal (`-60`, `12.5`, `0.156`).
- **Orientation:** `R0 R90 R180 R270 MR0 MR90 MR180 MR270`. Transform of a symbol-local point `(x, y)` placed at `(X, Y)`:
  1. if `M`: `(x, y) ← (−x, y)`
  2. rotate clockwise on screen (y down) by k·90°: `R90: (x, y) → (−y, x)`, `R180: (−x, −y)`, `R270: (y, −x)`
  3. translate: `(x + X, y + Y)`

## 4. Sections and records

Records appear in rank order; a record of lower rank after a higher rank is an error (`E023`). Within rank 3 the writer emits kinds in the order listed and each kind sorted by id (bytewise); the reader accepts any order within rank 3. Ranks 1, 2, 4, 5 keep file order (it is semantic). Rank 6 is written sorted by (kind, name).

Ids: 1–13 chars of lowercase Crockford base32 (`0-9 a-h j k m n p-t v-z`); the writer generates 6. Unique per file across all id-bearing kinds; a duplicate on load is re-id'd with warning `W001` (later occurrence). Names (pins, params, impls, vars, outputs, corners, configs, code blocks) are unique within their kind (`E024`).

### Rank 0 — header

`schemify 1 <kind>` with attributes:

| Attr | Kinds | Meaning |
|---|---|---|
| `doc <text>` | cells | one-line description |
| `default <impl>` | shprim, shcomp | default impl name (required when >1 impl) |
| `prefix <letter>` | shprim, shcomp | default instance-name prefix in the editor (`M`, `R`, `X`) |
| `category <word>` | cells | library browser grouping |
| `provenance <text>` | cells | importer source note |
| `corner <name>` | shbench | default corner |
| `config <name>` | shbench | default config |

### Rank 1 — symbol interface (shprim, shcomp only)

| Record | Attributes | Notes |
|---|---|---|
| `pin <name> <dir> <x> <y>` | `domain analog\|digital\|real` (default `analog`), `inherit <net>`, `number <n>` | `dir` ∈ `in out inout`. File order = port order. `name` may be a bus `D[3:0]`. `inherit` makes the pin an inherited connection (D29): it is not drawn-connected; its net is resolved by name (§5 of netlist spec). |
| `param <name> <default>` | `kind number\|int\|bool\|string\|choice` (default `number`), `unit <u>`, `min <expr>`, `max <expr>`, `step <expr>`, `grid <expr>`, `choices <v>…`, `computed <expr>`, `show <expr>`, `doc <text>`, `cached <value> <hash>` | `default` and `computed` are expressions for number/int/bool, template strings for string/choice. A `computed` param cannot be overridden (`E030`). `cached` holds the last value of an expression that calls a plugin function (D20). |
| `impl <name> <lang>` | `format <dialect> <template>` (repeatable, one per dialect), `model <name>`, `file <relpath>` | `lang` ∈ `schematic spice veriloga verilog` or any plugin tag. `dialect` ∈ `spice xyce cdl spectre verilog` (`spice` is the fallback for `xyce`). `file` is relative to the cell file. |

Kind rules: `shprim` has ≥1 pin and ≥1 impl, none with lang `schematic`. `shcomp` has exactly one impl with lang `schematic` (`E025`). Every impl of a cell matches the cell's pins (D11): enforced on save (`E002`).

### Rank 2 — symbol graphics (shprim, shcomp only)

| Record | Attributes |
|---|---|
| `line <x1> <y1> <x2> <y2>` | `layer <n>`, `dash <n>`, `width <n>` |
| `rect <x1> <y1> <x2> <y2>` | `layer`, `fill true\|false` |
| `arc <cx> <cy> <r> <start_deg> <sweep_deg>` | `layer`, `fill` |
| `circle <cx> <cy> <r>` | `layer`, `fill` |
| `poly <x> <y> <x> <y> …` (≥ 2 points) | `layer`, `fill`, `closed true\|false` |
| `stext <x> <y> <orient> <template>` | `size <s>`, `hcenter`, `vcenter`, `layer` |

`stext` templates may reference `name`, params and `cell` (render-time only).

### Rank 3 — schematic (shcomp, shbench)

Kinds in writer order:

| Record | Attributes | Notes |
|---|---|---|
| `inst <id> <cellref>` | `at <x> <y> <orient>` (required, first), `name <instname>` (required), `<param> <value>` overrides, `inherit <pin> <net>` | `cellref` = `lib:cell` or `./cell` (same directory). Only overrides are stored (D40). `name` may be iterated `X1[3:0]`. |
| `wire <id> <x1> <y1> <x2> <y2>` | `bus true` | Writer normalizes so `(x1, y1) < (x2, y2)` lexicographically. Zero-length wires are an error (`E026`). |
| `label <id> <x> <y> <orient> <netexpr>` | | Net names live only on labels and ports. |
| `port <id> <x> <y> <orient> <pin>` | | Binds a schematic net to interface pin `<pin>` (shcomp only). |
| `noconn <id> <x> <y>` | | Marks an intentionally unconnected pin. |
| `text <id> <x> <y> <orient> <text>` | `size <s>` | Annotation only. |
| `constraint <id> <type>` | `member <id>` (repeatable), any `<key> <value>` | D30. `type` is a plugin-defined word; core stores, diffs, revalidates members. |
| `waive <id> <inst_id> <param>` | `reason <text>` (required) | D13 waiver for one instance param. |

### Rank 4 — docs (cells)

Document order is preserved.

| Record | Attributes | Notes |
|---|---|---|
| `prose <id>` | verbatim lines | Markdown + `$…$` / `$$…$$` LaTeX math (D21). |
| `var <name> <expr>` | `unit <u>`, `min <expr>`, `max <expr>`, `doc <text>`, `cached <value> <hash>` | The cell's variables table (D19). Bench vars are the sweep/optimizer variables. |

### Rank 5 — impl bodies

| Record | Attributes |
|---|---|
| `body <impl>` | verbatim lines: the definition text emitted once per netlist (SPICE subckt text, Verilog-A source, Verilog source) |

### Rank 6 — bench (shbench only)

| Record | Attributes | Notes |
|---|---|---|
| `corner <name>` | `lib <path> <section>` (repeatable), `include <path>` (repeatable), `set <var> <expr>` (repeatable), `temp <expr>` | Model sections and var overrides. Paths may contain `${NAME}` expanded from project `set` entries, then the environment; nothing executes. |
| `config <name>` | `order <impl>…`, `cell <cellref> <impl>` (repeatable), `inst <path> <impl>` (repeatable) | D11a impl selection. |
| `sweep <name>` | `var <v>`, and one of `values <expr>…` / `lin <from> <to> <n>` / `log <from> <to> <n>` | |
| `montecarlo <name>` | `samples <n>`, `seed <n>` | |
| `optimize <name>` | `algo <word>`, `goal <output> min\|max\|target <expr>` (repeatable), `vary <var>` (repeatable; bounds come from the var) , `budget <n>` | |
| `output <name> <expr>` | `unit <u>` | Column-mode expression over results. |
| `spec <output>` | `min <expr>`, `max <expr>` | |
| `code <name> <lang>` | verbatim lines | `lang` `spice` = all SPICE dialects; `ngspice`/`xyce`/`spectre` = that dialect only; anything else goes to a runner plugin. |

### Project manifest (`schemify.project`)

| Record | Attributes |
|---|---|
| `lib <name> path <relpath>` / `lib <name> git <url> <rev>` / `lib <name> url <url> <sha256>` | `subdir <relpath>` |
| `plugin <name> path\|git\|url …` (same forms) | `grant <capability>` (repeatable) |
| `set <key> <value>` | |

`lib core builtin` is implicit (the bundled core library).

### Lockfile (`schemify.lock`)

`locked <lib|plugin> <name> <kind> <resolved> <sha256>`, sorted by (type, name). Tool-written only.

### User config

`set <key> <value>`, `key <chord> <command>`, `sim <name>` with `path`, `args`, `plugin <name>` (order = load order, D8). The app edits single lines in place and never rewrites hand-written config.

## 5. Unknowns and forward compatibility

- Unknown attributes are preserved verbatim, in order, after known ones.
- An unknown record word is an error (`E027`) unless the header version is newer than the reader's, in which case the record (with its attributes) is preserved verbatim at its position.

## 6. Values inside records

The file layer only splits values. Their meaning:

- **Expression positions** (`param` default for number/int/bool kinds, `computed`, `min`, `max`, `step`, `grid`, `show`, `var`, `output`, `corner set`, `sweep` bounds, instance overrides of number/int/bool params): parsed by [expr.md](expr.md).
- **Template positions** (`format`, `stext`, string/choice param defaults and overrides): template strings ([expr.md §6](expr.md)).
- Everything else: literal text.

## 7. Error and warning codes

`E0xx` codes are shared with the checker (netlist spec §8). Format-level codes:

| Code | Meaning |
|---|---|
| E020 | same cell stem with two kinds in one library |
| E021 | file too large |
| E022 | coordinate not representable |
| E023 | record out of rank order |
| E024 | duplicate name within a kind |
| E025 | impl count rule for the kind violated |
| E026 | zero-length wire |
| E027 | unknown record |
| E028 | malformed line (lexical) |
| E029 | wrong arity / non-numeric where numeric required |
| E030 | override of a computed param |
| W001 | duplicate id re-assigned |
