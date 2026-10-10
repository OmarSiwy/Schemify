# Expression language v1 (normative)

Implements D12, D19, D20, D23. One language for parameter defaults and derived params, conditions, netlist templates, labels, docs variables, outputs/specs and the waveform calculator. Pure and total: no IO, no loops, no recursion, no user-defined functions, bounded size. Seed implementation: UW-ASIC `crates/wave/src/expr.rs` (lexer → Pratt parser → AST → eval over columns).

## 1. Values

| Type | Notes |
|---|---|
| `num` | f64. NaN/±inf are errors at the point they are produced (`X010`), never stored. |
| `int` | i64; produced by int params and integer functions; promotes to `num` in arithmetic. |
| `bool` | `true`/`false`. |
| `str` | UTF-8, ≤ 1 MiB. |
| `cnum` | complex f64 (AC results). |
| `col` / `ccol` | real/complex column over the sweep axis of one result set (calculator mode only). |

Scalar mode (params, vars, templates, specs) has no `col`/`ccol`. Calculator mode (outputs, graph traces) has all types; scalar operands broadcast over columns. Column operands must share the sweep axis (`X011` otherwise).

## 2. Lexical

- **Number:** `digits[.digits][(e|E)[+-]digits][suffix]` or `.digits…`. Suffix, case-insensitive: `f`=1e-15 `p`=1e-12 `n`=1e-9 `u`/`µ`=1e-6 `m`=1e-3 `k`=1e3 `meg`=1e6 `g`=1e9 `t`=1e12 `mil`=25.4e-6. Any letters after a valid suffix are an error (`X001`); SPICE's "ignore trailing letters" is **not** accepted (the importer strips them).
- **String:** `"…"` with escapes `\" \\ \n \t \u{…}`.
- **Identifier:** `[A-Za-z_][A-Za-z0-9_]*`; namespaced call `ns.name(…)` (plugin functions, D20).
- **Reserved:** `true false pi`.
- **Operators:** `+ - * / % ^ == != < <= > >= && || ! ? :` and `( ) ,`. `^` is power, right-associative.

## 3. Grammar and precedence (low → high)

```
expr    = ternary
ternary = or ["?" expr ":" expr]
or      = and *("||" and)
and     = cmp *("&&" cmp)
cmp     = add [("=="|"!="|"<"|"<="|">"|">=") add]     ; non-associative
add     = mul *(("+"|"-") mul)
mul     = unary *(("*"|"/"|"%") unary)
unary   = ("-"|"!") unary | pow
pow     = call ["^" unary]                             ; right-assoc
call    = primary ["(" [expr *("," expr)] ")"]
primary = number | string | ident | ns "." ident | "(" expr ")"
```

Limits: ≤ 65 536 AST nodes and nesting depth ≤ 256 per expression (`X002`). These bounds make evaluation total.

## 4. Semantics

- Arithmetic on `int` stays `int` for `+ - * %` without overflow; overflow or `/` promotes to `num`. `%` follows the sign of the dividend.
- `&&`/`||` short-circuit; `?:` evaluates one branch. Comparisons of mixed `int`/`num` compare as `num`; `str` compares bytewise; other mixes are `X003`.
- `+` on two `str` concatenates; any other `str` arithmetic is `X003`.
- Identifier resolution order, by context:
  - **Cell scope** (vars, param defaults, instance overrides inside a cell): the cell's own params (effective values for the instance being elaborated), then the cell's vars. Unknown → `X004`.
  - **Bench scope:** bench vars, after corner `set` overrides and sweep/MC/optimizer assignments.
  - **Calculator:** result signals via `v("net")`, `i("dev")`, `x()` (sweep axis), plus bench vars.
- Dependency graph: every identifier is an edge; a cycle is `X005` and names the cycle.

## 5. Built-in functions

Scalar (also element-wise on columns):
`abs sqrt exp ln log10 sin cos tan asin acos atan atan2(y,x) sinh cosh tanh floor ceil round(x) int(x)` (truncate toward zero) `min(a,b,…) max(a,b,…) clamp(x,lo,hi) sign pow(x,y) if(c,a,b) snap(x,grid)` (round to nearest multiple, ties away from zero) `eng(x[,digits])` → `str` engineering notation, `str(x)` canonical number text (§7), `num(s)` parse with SI suffix, `len(s)`.

Complex: `re im mag ph` (degrees) `db` (20·log10|x|) `conj cplx(re,im)`.

Calculator-only (columns): `v(net) i(dev) x() deriv(y) integ(y) avg(y) rms(y) ymax(y) ymin(y) pp(y) at(y, x0)` (linear interpolation) `cross(y, level[, edge, n])` → x of nth crossing (`edge` = `1` rising, `-1` falling, `0` either) `delay(y1, l1, y2, l2)` `settle(y, tol)` `fft(y)` `gain_margin(h) phase_margin(h)`. Functions operate per sweep step; `deriv`/`fft` never cross step boundaries.

Physical constants live under `phys.`: `phys.k phys.q phys.h phys.c phys.eps0 phys.kelvin0` (273.15).

Plugin functions: `ns.f(args)` resolved through the host's registry. If the plugin is missing and the record has `cached <value> <hash>`, the cached value is used and the value is flagged *unverified* (D20); otherwise `X006`.

## 6. Templates

A template is a string with `{expr}` holes; `{{` and `}}` are literal braces. Each hole's value is rendered by §7. In netlist context these extra identifiers exist:

| Identifier | Value |
|---|---|
| `name` | the instance name as emitted for the dialect |
| `cell` | subckt / module name of the selected impl |
| `model` | the impl's `model` attr, else the `model` param |
| `pins` | space-joined net names in pin order |
| `<pin name>` | the net connected to that pin (bus pins: space-joined bits) |
| `params` | `k=v` pairs for every param marked for the dialect (default: all non-computed params whose value differs from the subckt default; see netlist spec) |
| `file` | absolute path of the impl's `file` |

Pin names shadow params of the same name inside templates (`X007` warns at load when they collide).

## 7. Canonical number text

Used by templates, netlists and `str()`, identical on every platform:
- `int`, or `num` that is an integer with |x| < 1e15: plain integer (`2`, `-40`).
- otherwise if 1e-4 ≤ |x| < 1e15: shortest round-trip decimal (`0.15`, `2.5e-3` is written `0.0025`).
- otherwise: shortest round-trip mantissa with exponent `e-12` form (`1e-12`, `2.5e-15`, `1.2e+15` is `1.2e15`).

## 8. Error codes

| Code | Meaning |
|---|---|
| X001 | bad number literal / suffix |
| X002 | expression too large or too deep |
| X003 | type error |
| X004 | unknown identifier |
| X005 | dependency cycle |
| X006 | unknown function / missing plugin without cache |
| X007 | pin/param name shadowing in a template (warning) |
| X008 | wrong arity |
| X009 | parse error |
| X010 | NaN or infinity produced |
| X011 | column length / axis mismatch |
