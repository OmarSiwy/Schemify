# PDK XSchem device libraries: survey for `tools/devicereader`

> **Clean-room rule (DECISIONS.md #1):** Schemify is permissively licensed; xschem is GPL. Nothing below may be *ported* or copied from xschem source. Where this doc says "port", read: write a behaviour spec from the documented semantics, then implement it independently and verify it against golden outputs produced by *running* xschem (netlists, evaluated values). Source-file citations are kept only as provenance for behaviour claims.

Status: research, 2026-10-10. Scope: how sky130, GF180MCU and IHP SG13G2 ship XSchem primitive symbols and connect them to ngspice. The goal is to define what `tools/devicereader` has to do to convert them into Schemify `.shprim` files.

## Sources (pinned commits)

| Short | Repo @ commit | Path surveyed |
|---|---|---|
| SKY | [StefanSchippers/xschem_sky130 @ fef70fa](https://github.com/StefanSchippers/xschem_sky130/tree/fef70fad3804d1c298c70f78de769825d7dfea19) (open_pdks installs it as `sky130A/libs.tech/xschem`, see `XSCHEM_URL` in [open_pdks sky130/Makefile.in](https://github.com/RTimothyEdwards/open_pdks/blob/master/sky130/Makefile.in)) | `sky130_fd_pr/` (79 sym), `sky130_stdcells/` (437), `sky130_tests/` (77 sch), `xschemrc` |
| GF | [fossi-foundation/globalfoundries-pdk-libs-gf180mcu_fd_pr @ e11a8c9](https://github.com/fossi-foundation/globalfoundries-pdk-libs-gf180mcu_fd_pr/tree/e11a8c98dfc905561d38813b1416e65a4b9280e9/cells/xschem) (open_pdks copies `cells/xschem/*` and runs [fix_xschemrc.py](https://github.com/RTimothyEdwards/open_pdks/blob/master/gf180mcu/custom/scripts/fix_xschemrc.py), see [gf180mcu/Makefile.in](https://github.com/RTimothyEdwards/open_pdks/blob/master/gf180mcu/Makefile.in) target `xschem-%`) | `cells/xschem/gf180mcu_fd_pr/` (68 sym), `tests/` (59 sch), `xschemrc`, `xschem-menu` |
| IHP | [IHP-GmbH/IHP-Open-PDK @ 5e6d592](https://github.com/IHP-GmbH/IHP-Open-PDK/tree/5e6d592e4002946a4616f798c357f0f3c06cf3b6/ihp-sg13g2/libs.tech) | `libs.tech/xschem/sg13g2_pr/` (45 sym), `libs.ref/sg13g2_stdcell/sym/xschem/` (84), `sg13g2_tests/` (55 sch), `xschemrc`, `xschem-drc`, `libs.tech/ngspice/.spiceinit`, `libs.tech/verilog-a/` |
| XS | [StefanSchippers/xschem @ efc04f4](https://github.com/StefanSchippers/xschem/tree/efc04f496485f112fd97b69ce907131bc63e4dab) (local 3.4.7 tree at `/nix/store/rhvgzl9m…-source` matches except where noted) | `src/eval_expr.y`, `src/token.c`, `src/save.c`, `src/xschem.tcl`, `doc/xschem_man/symbol_property_syntax.html`, `developer_info.html`, `xschem_library/{devices,ngspice,ngspice_verilog_cosim}` |

The counts in this document come from a Python pass over every `.sym` and `.sch` file listed above. It uses a brace-block tokenizer with xschem's escaping rules, a `key=value` parser, and regexes per idiom. Counts are numbers of symbols unless marked otherwise.

---

## 1. XSchem symbol anatomy (what the reader must parse)

File format ([developer_info.html](https://github.com/StefanSchippers/xschem/blob/efc04f496485f112fd97b69ce907131bc63e4dab/doc/xschem_man/developer_info.html)):

| Record | Meaning | Notes for import |
|---|---|---|
| `v {xschem version=… file_version=1.x}` | version header | PDK files mix 1.0, 1.2 and 1.3. Comment lines after the header are allowed. |
| `K {…}` | symbol netlisting attributes | Files older than format 1.2 put the same data in `G {}`. No PDK file in the survey uses `G`, but 23 files in xschem's `devices/` do. Read `K` and fall back to `G`. |
| `G/V/S/E/F {}` | schematic-global props (VHDL/Verilog/SPICE/tEDAx/Spectre) | Always empty in primitives |
| `L layer x1 y1 x2 y2 {props}` | line | `dash=`, `bus=` |
| `B layer x1 y1 x2 y2 {props}` | rectangle. **Layer 5 means a pin.** | Pin props: `name dir pinnumber sim_pinnumber propag goto clock` |
| `P layer n x1 y1 … {fill=true}` | polygon | Arrowheads on MOS/BJT |
| `A layer cx cy r start sweep {}` | arc, angles in degrees | |
| `T {text} x y rot flip xscale yscale {props}` | text | `layer= hcenter= vcenter= hide=`. Text can contain `@tokens`, `tcleval(...)`, `expr(...)`. |

There are three escaping levels, and all three show up in real PDK files:
1. File level: `\{ \} \\` inside `{…}` blocks.
2. Attribute level: `key="…"` with `\"`.
3. Nested template values, written `\\"expr('…')\\"` in the file.

Two more forms are tokens rather than escapes. `\\\\` in the file (`\\` in the attribute) ends an `@token`: `@prefix\\a21o_1` becomes `sg13g2_a21o_1`. `\\$` emits a literal `$`, as in IHP `lvs_format`.

### Netlist-time substitution order (xschem `print_spice_element`, [src/token.c](https://github.com/StefanSchippers/xschem/blob/efc04f496485f112fd97b69ce907131bc63e4dab/src/token.c))
1. Each `@tok` in `format` is looked up first in the instance props, then in the symbol `template`, then in the parent instance and parent template (for hierarchical parameter passing). Tokens that are still unresolved become empty.
2. `@spiceprefix` expands to its value only while the Tcl var `spiceprefix` is 1 (the default; the LVS menu can toggle it). It is emitted as a tagged prefix so that vector instance names expand properly (`XM1[3]…`).
3. `@pinlist` gives the nets in **pin order**. Pin order is file order of `B 5` records, or ascending `sim_pinnumber` when *every* pin has one ([save.c `sort_symbol_pins`](https://github.com/StefanSchippers/xschem/blob/efc04f496485f112fd97b69ce907131bc63e4dab/src/save.c)). `pinnumber` does not affect order. It is the package pin for tEDAx, and has been since xschem 3.4.0 (Changelog: "Use sim_pinnumber for port ordering"). `@@D` gives one pin's net explicitly.
4. If the substituted value contains `expr(`, `expr_eng(` or `expr_eng4(`, the **C** evaluator in [src/eval_expr.y](https://github.com/StefanSchippers/xschem/blob/efc04f496485f112fd97b69ce907131bc63e4dab/src/eval_expr.y) runs. **This is not Tcl.** When parsing fails (for example a non-numeric `W_N`), the `expr(` wrapper is removed and the text is left in place, so it reaches SPICE as an expression.
5. If the whole attribute is wrapped in `tcleval(…)`, the Tcl proc `tclpropeval2` runs `uplevel #0 subst {…}` on it ([src/xschem.tcl](https://github.com/StefanSchippers/xschem/blob/efc04f496485f112fd97b69ce907131bc63e4dab/src/xschem.tcl)). That means `$var` and `[cmd]` substitution in global scope.

`eval_expr.y` grammar (master): `+ - * / % ^`, unary `-`, `== != < > <= >=`, `?:`, parentheses. Functions: `int sin cos tan asin acos atan log(=log10) ln exp sqrt round round1 round6`. Constants: `pi e k h echarge abszero c`. Numbers go through `atof_eng`, so SI suffixes like `0.15u` are accepted. `expr(` prints `%.15g`. `expr_eng(` prints engineering notation at `ev_precision`; `expr_eng4(` uses 4 digits. The `expr_eng` forms are master/3.4.8RC only and are absent from the local 3.4.7 tree, which IHP files (saved with 3.4.8RC) already rely on. 3.4.7 also lacks comparisons and `?:`.

---

## 2. Per-PDK findings

### 2.1 SkyWater sky130 (`xschem_sky130/sky130_fd_pr`)

**Model hookup.** `xschemrc` resolves `PDK_ROOT` from `$env(PDK_ROOT)`, falling back to `/usr/share/pdk`, `/usr/local/share/pdk`, then `~/share/pdk`. `PDK` comes from `$env(PDK)` and defaults to `sky130A`. It then sets two Tcl globals:
```tcl
set SKYWATER_MODELS   ${PDK_ROOT}/${PDK}/libs.tech/combined
set SKYWATER_STDCELLS ${PDK_ROOT}/${PDK}/libs.ref/sky130_fd_sc_hd/spice
```
Testbenches use `corner.sym`, which is a `netlist_commands` symbol, or a `code.sym` block containing `format="tcleval( @value )"`:
```
K {type=netlist_commands
template="name=CORNER only_toplevel=false corner=tt"
format="tcleval(.lib $::SKYWATER_MODELS/sky130.lib.spice @corner )"}
```
Corner names are `.lib` sections of `sky130.lib.spice`: `tt ff ss sf fs` and mismatch variants such as `tt_mm`. In `sky130_tests`: 18 `corner.sym` instances (tt 12, tt_mm 5, ff 1), 23 tcleval code blocks, and 40 refs to `$::SKYWATER_MODELS`. ngspice also needs `.spiceinit` containing `set ngbehavior=hsa` and `set ng_nomodcheck` ([xschem sky130 tutorial](https://github.com/StefanSchippers/xschem/blob/efc04f496485f112fd97b69ce907131bc63e4dab/doc/xschem_man/tutorial_xschem_sky130.html)).

**Model naming.** `model=nfet_01v8` is the short name. `format` adds the prefix `sky130_fd_pr__@model`, and `spiceprefix=X` makes the instance a call to the PDK `.subckt`. The six `res_generic_*` symbols have no `@spiceprefix`; they are native `R` devices.

**nfet: `sky130_fd_pr/nfet_01v8.sym`** ([file](https://github.com/StefanSchippers/xschem_sky130/blob/fef70fad3804d1c298c70f78de769825d7dfea19/sky130_fd_pr/nfet_01v8.sym))
```
K {type=nmos
lvs_format="@spiceprefix@name @pinlist sky130_fd_pr__@model L=@L W=@W nf=@nf m=@mult"
format="@spiceprefix@name @pinlist sky130_fd_pr__@model L=@L W=@W
+ nf=@nf ad=@ad as=@as pd=@pd ps=@ps
+ nrd=@nrd nrs=@nrs sa=@sa sb=@sb sd=@sd
+ mult=@mult m=@mult"
template="name=M1
W=1
L=0.15
nf=1
mult=1
ad=\\"expr('int((@nf + 1)/2) * @W / @nf * 0.29')\\"
pd=\\"expr('2*int((@nf + 1)/2) * (@W / @nf + 0.29)')\\"
as=\\"expr('int((@nf + 2)/2) * @W / @nf * 0.29')\\"
ps=\\"expr('2*int((@nf + 2)/2) * (@W / @nf + 0.29)')\\"
nrd=\\"expr('0.29 / @W ')\\" nrs=\\"expr('0.29 / @W ')\\"
sa=0 sb=0 sd=0
model=nfet_01v8
spiceprefix=X
"
drc="sky130_fet_drc \{@name\\\\\} \{@symname\\\\\} \{@model\\\\\} \{@W\\\\\} \{@L\\\\\} \{@nf\\\\\}"}
B 5 17.5 -32.5 22.5 -27.5 {name=D dir=inout}
B 5 -22.5 -2.5 -17.5 2.5 {name=G dir=in}
B 5 17.5 27.5 22.5 32.5 {name=S dir=inout}
B 5 19.921875 -0.078125 20.078125 0.078125 {name=B dir=in}
T {@mult x @W / @L} 31.25 13.75 0 0 0.2 0.2 { layer=13}
T {id=@spice_get_node i(\\@m.@path@spiceprefix@name\\.msky130_fd_pr__@model\\[id])} 30 -30 0 0 0.15 0.15 {layer=17}
T {vgs=expr(@#1:spice_get_voltage - @#2:spice_get_voltage ) …} 5 17.5 0 1 0.07 0.07 {layer=15}
```
Notes:
- Units are µm without suffix (`W=1 L=0.15`). The models run with `scale=1e-6`.
- `ad/as/pd/ps/nrd/nrs` are **computed defaults** in `expr()`. That is 36 FET symbols with 6 each, 216 expressions in all.
- The `*_nf` variants use a SPICE-side expression instead: `W='@W * @nf '` (11 symbols).
- 3-terminal FETs (`nfet3_*`, `pfet3_*`) take the bulk net from a parameter: `@pinlist @body` with `body=GND`.

**Resistor: `res_high_po_0p35.sym`**
```
K {type=high_precision_poly_resistor
format="@spiceprefix@name @pinlist sky130_fd_pr__@model L=@L mult=@mult m=@mult"
template="name=R1
L=0.35
model=res_high_po_0p35
spiceprefix=X
mult=1"}
B 5 -2.5 27.5 2.5 32.5 {name=M dir=inout propag=1 pinnumber=2}
B 5 -2.5 -32.5 2.5 -27.5 {name=P dir=inout propag=0 pinnumber=1}
B 5 -22.5 -2.5 -17.5 2.5 {name=B dir=inout pinnumber=1}
```
The width is part of the model name (`_0p35`), so each width has its own symbol. The pin order is M, P, B because that is the file order; the `pinnumber` values have no effect on it.

**BJT / diode**
```
npn_05v5.sym:  K {type=npn
 lvs_format="@spiceprefix@name @pinlist sky130_fd_pr__@model NE=1 m=@m"
 format="@spiceprefix@name @pinlist sky130_fd_pr__@model m=@m mult=@m"
 template="name=Q1 model=npn_05v5_w1p00l2p00 m=1 spiceprefix=X"}
 pins: c(in-out,1) b(in,2) e(3) s(4)
diode.sym:     K {type=diode
 format="@spiceprefix@name @pinlist sky130_fd_pr__@model area=@area perim=@perim"
 template="name=D1 model=diode_pw2nd_05v5 area=1e12 perim=4e6 spiceprefix=X"
 available_models="sky130_fd_pr__diode_pw2nd_05v5 … (13 names)"}
```
`available_models` is free text that lists model choices; it appears in 2 sky130 symbols and 1 GF180 symbol.

**Verilog-A device: `reram.sym`** (the only Verilog-A primitive in the three PDK libs)
```
K {type=reram
format="@spiceprefix@name @pinlist @model Tfilament_0=@Tfilament_0"
template="name=R1 model=sky130_fd_pr_reram__reram_cell spiceprefix=X Tfilament_0=3.8e-9"
device_model="tcleval(
.subckt sky130_fd_pr_reram__reram_cell TE BE Tfilament_0=3.3e-9 area_ox=0.1024e-12
N1 TE BE nFilament sky130_fd_pr_reram__reram_model
.ic v(nFilament)=\{Tfilament_0*1.0e9\}
.ends sky130_fd_pr_reram__reram_cell
.model sky130_fd_pr_reram__reram_model sky130_fd_pr_reram__reram_module
+ area_ox = 0.1024e-12 …
.control
pre_osdi $SKYWATER_MODELS/sky130_fd_pr_reram__reram_module.osdi
.endc
)"}
```
This uses a `device_model` (printed once per netlist) that contains an `N` instance of the OSDI module, a `.model` that binds it, and a `.control pre_osdi <path>` line. The path comes from the Tcl global inside `tcleval`.

**Tcl in `xschemrc`:**
- `sky130_fet_drc` is called from `drc=` on 40 symbols. It holds min W/L rules per model regex.
- `sky130_display_fet_params` is called from a label in `annotate_fet_params.sym`.
- `sky130_sch_expand` and `sky130_save_fet_params` generate `.save` lines.
- `sky130_menupdk` adds a menu.

None of these procs is needed to produce a simulation netlist.

### 2.2 GF180MCU (`cells/xschem/gf180mcu_fd_pr`)

**Model hookup.** `xschemrc` sets `180MCU_MODELS ${PDK_ROOT}/models/ngspice`. open_pdks' `fix_xschemrc.py` rewrites that to `${PDK_ROOT}/<tech>/libs.tech/ngspice` and appends library paths. Every test bench (58 of 59) has the same `code_shown` block:
```
format="tcleval( @value )"
value=".include $::180MCU_MODELS/design.ngspice
.lib $::180MCU_MODELS/sm141064.ngspice typical
.lib $::180MCU_MODELS/sm141064.ngspice res_typical   (also moscap_typical, bjt_typical, diode_typical, cap_mim, mimcap_typical)"
```
The tests contain 536 refs to `$::180MCU_MODELS`. Corner sections in `sm141064` are `typical ff ss fs sf`, plus per-device-class sections.

**nfet: `nfet_03v3.sym`**
```
K {type=nmos
format="@spiceprefix@name @pinlist @model L=@L W=@W
+ nf=@nf ad=@ad as=@as pd=@pd ps=@ps
+ nrd=@nrd nrs=@nrs sa=@sa sb=@sb sd=@sd
+ m=@m"
lvs_format="@name @pinlist @model L=@L W=@W nf=@nf m=@m"
template="name=M1
L=0.28u
W=0.22u
nf=1
m=1
ad=\\"'int((nf+1)/2) * W/nf * 0.18u'\\"
pd=\\"'2*int((nf+1)/2) * (W/nf + 0.18u)'\\"
as=\\"'int((nf+2)/2) * W/nf * 0.18u'\\"
ps=\\"'2*int((nf+2)/2) * (W/nf + 0.18u)'\\"
nrd=\\"'0.18u / W'\\" nrs=\\"'0.18u / W'\\"
sa=0 sb=0 sd=0
model=nfet_03v3
spiceprefix=X"}
pins D G S B (same geometry as sky130)
T {tcleval(gm=[to_eng [ngspice::get_node [subst -nocommand \{\\@m.$\{path\}@spiceprefix@name\\.m0\\[gm]\}]]])} … {layer=15}
```
Units use SI suffixes (`0.28u`). The computed geometry is a **SPICE-side quoted expression** that ngspice evaluates, so no import-time evaluation is needed. That covers 19 FET symbols, about 114 expressions.

**Resistor: `ppolyf_u_1k.sym`**
```
K {type=res
format="@spiceprefix@name @pinlist @model r_width=@W r_length=@L m=@m"
template="name=R1 W=1e-6 L=1e-6 model=ppolyf_u_1k spiceprefix=X m=1"}
pins M(pinnumber=2) P(1) B(1)   -> pin order M P B
```

**Diode (native `D`, no X): `diode_nd2ps_03v3.sym`**
```
K {type=diode
format="@name @pinlist @model area='@r_w * @r_l ' pj='2*@r_w + 2*@r_l ' m=@m"
template="name=D1 model=diode_nd2ps_03v3 r_w=1u r_l=1u m=1"}
B 5 … {name=p dir=inout pinnumber=1 propag=1 goto=1}
B 5 … {name=m dir=inout pinnumber=2 goto=0}
```
The SPICE device letter comes from the default instance name `D1`. Like `M1`, `R1` and `Q1`, the instance name carries the letter.

**BJT: `npn_10p00x10p00.sym`.** `type=vertical_npn`, `format="@spiceprefix@name @pinlist @model m=@m"`, pins C B E S.

**Tcl.** The 19 display-only annotation labels use `ngspice::get_node`, `subst` and `to_eng`. `xschem-menu` holds the `.save`/display procs. There is no `drc=` and no Tcl at all in any netlist attribute.

### 2.3 IHP SG13G2 (`libs.tech/xschem/sg13g2_pr`)

**Model hookup.** `xschemrc` sets these Tcl globals: `MODELS_NGSPICE = SG13G2_MODELS = ${PDK_ROOT}/${PDK}/libs.tech/ngspice/models`, and `MODELS_XYCE` / `SG13G2_MODELS_XYCE` the same way for Xyce. It also sources `cap_cmomi.tcl`, `xschem-drc` and `xschem-menu`, and globs `${PDK_ROOT}/ihp-sg13_modules/*`. Test benches load models in two ways:
- Bare `.lib cornerMOSlv.lib mos_tt`. This works because `libs.tech/ngspice/.spiceinit` sets ngspice `sourcepath` to `$PDK_ROOT/$PDK/libs.tech/ngspice/models`.
- `tcleval` with `$::MODELS_NGSPICE/cornerCAP.lib cap_typ` (21 blocks).

Corner files are `cornerMOSlv.lib cornerMOShv.lib cornerHBT.lib cornerRES.lib cornerCAP.lib cornerDIO.lib cornerMOSCAP.lib`. Sections are `mos_{tt,ss,ff,sf,fs}[_mismatch]` + `mos_tt_stat`, `hbt_/cap_/res_{typ,bcs,wcs}[_mismatch]` + `*_typ_stat`/`res_stat`, `dio_{tt,ss,ff}`, `moscap_tt*`.

**OSDI.** `libs.tech/verilog-a/openvaf-compile-va.sh` compiles `psp103.va psp103_nqs.va r3_cmc.va mosvar.va cap_cmomi.va cap_cmomf.va` into `libs.tech/ngspice/osdi/*.osdi` using `openvaf-r` or `openvaf` with `-D__NGSPICE__`. The `.osdi` files are build artifacts and are **not committed**. ngspice loads them at startup from `.spiceinit`:
```
osdi '$PDK_ROOT/$PDK/libs.tech/ngspice/osdi/psp103.osdi'      (… psp103_nqs, r3_cmc, mosvar, cap_cmomi, cap_cmomf)
```
Model libraries bind those modules with ordinary `.model` lines and wrap them in `.subckt`:
```
sg13g2_moslv_parm.lib: .model sg13g2_lv_nmos_psp psp103va  type=+1 …
sg13g2_moslv_mod.lib:  .subckt sg13_lv_nmos d g s b …  Nsg13_lv_nmos d g s b sg13g2_lv_nmos_psp …
resistors_mod.lib:     .model res_rppd r3_cmc
cap_cmomi.lib:         .subckt cap_cmomi PLUS MINUS w=… ; Ncap PLUS MINUS 0 cap_cmomi_mod … ; .model cap_cmomi_mod cap_cmomi
```
So **every IHP MOSFET, poly resistor, varactor and MOM cap is Verilog-A**, but only *behind* a SPICE subckt. Nothing in the symbol shows it, and **no IHP symbol uses `pre_osdi`**. HBTs (`npn13G2*`) use the built-in ngspice **VBIC** model (`level = 9`). There is no HICUM file in `libs.tech/ngspice` or `libs.tech/verilog-a` at this commit, so HICUM does not go through OpenVAF here. For Xyce, `adms-compile-va.sh` builds plugins instead.

**nfet: `sg13_lv_nmos.sym`**
```
K {type=nmos
lvs_format="M@name @pinlist @model w=@w l=@l ng=@ng m=@m"
format="@spiceprefix@name @pinlist @model w=@w l=@l ng=@ng m=@m mm_ok=@mm_ok"
template="name=M1
l=0.13u
w=0.15u
ng=1
m=1
mm_ok=1
model=sg13_lv_nmos
spiceprefix=X"
drc="fet_drc @name @symname @model @w @l @ng"}
pins D G S B (identical geometry to sky130)
```
There are no computed parameters; the PDK subckt computes geometry itself. `sg13_lv_rf_nmos` uses a separate `lvs_model=rfnmos` in `lvs_format`.

**Resistor: `rppd.sym`.** This is `expr_eng` in a *template value*, and the value feeds both the label and LVS:
```
K {type=res
lvs_format="R@name @pinlist @value \\$SUB=@body \\$[@model\\\\] w=@w l=@l b=@b m=@m"
format="@spiceprefix@name @pinlist @body @model w=@w l=@l m=@m b=@b mm_ok=@mm_ok"
template="name=R1 w=0.5e-6 l=0.5e-6 model=rppd body=sub! spiceprefix=X b=0 m=1 mm_ok=1
value=\\"expr_eng(  ( 70.0e-6 / @w + 260.0 * ( (@b + 1)* @l + ( 1.081*( @w + 6.0e-9 ) + 0.18e-6 )*@b ) / ( @w + 6.0e-9 ) ) / @m  )\\""
drc="res_drc @name @symname @model @w @l"}
B 5 … {name=P dir=inout propag=1 sim_pinnumber=1}
B 5 … {name=M dir=inout propag=0 sim_pinnumber=2}
```
`@body` is a **net passed as a parameter** (default `sub!`). The global net `sub!` comes from `sub.sym` (`type=label global=true format="*.alias @lab"`).

**HBT with Tcl arithmetic: `npn13G2l.sym`**
```
K {type=vertical_npn
lvs_format="tcleval(Q@name @pinlist @model le=[ev \{ @El * 1.0e-6 \} ] we=70.0n Nx=@Nx )"
format="@spiceprefix@name @pinlist @model Nx=@Nx El=@El mm_ok=@mm_ok"
template="name=Q1 model=npn13G2l spiceprefix=X Nx=1 El=1.0 mm_ok=1"
drc="hbt_drc @name @symname @model @Nx @El"}
B 5 … {name=C dir=inout sim_pinnumber=1}  B(2)  E(3)  S(4)
```
`ntap1`, `ptap1` and `pnpMPA` put `[ev7 {…}]` in the simulation `format`, for example `R=[ev7 \{ 1.0/(1.0/(9.8e-10/(@w*@l)) + 1.0/(9.8e-4/(2.0*(@w+@l)))) \}]`. `ev` and `ev7` are xschem procs: Tcl `expr` printed with `%.4g` / `%.7g`, returning the expression text on failure.

**Verilog-A-backed cap: `cap_cmomi.sym`**
```
K {type=capacitor
format="@spiceprefix@name @pinlist @model w=@w l=@l mmin=@mmin mmax=@mmax feed=@feed subblock=@subblock m=@m mm_ok=@mm_ok"
template="name=C1 model=cap_cmomi w=5e-6 l=5e-6 mmin=1 mmax=5 feed=double subblock=0 m=1 mm_ok=1 spiceprefix=X"}
T {tcleval(C=[cap_cmomi_C @m @l @w @mmin @mmax @feed ])} … {layer=13}   ;# display only, proc in cap_cmomi.tcl
```
The `feed` parameter is a string enum (`same|double|none`).

**Std cells: `sg13g2_stdcell/sym/xschem/sg13g2_a21o_1.sym`**, with Tcl deciding the symbol *type*:
```
K {type=tcleval([symtype])
function3="0 1 & 2 |"
format="@name @@X @@A1 @@A2 @@B1 @VDD @VSS @prefix\\\\a21o_1"
schematic="tcleval([hierarchy_config @symname])"
template="name=x1 VDD=VDD VSS=VSS prefix=sg13g2_ "
extra="VDD VSS prefix"}
```
`symtype` returns `subcircuit`, or `primitive` when the user picks "no hierarchy". `hierarchy_config` returns `libs.ref/sg13g2_stdcell/sch/xschem/<cell>[_custom].sch`. sky130 std cells follow the same pattern without Tcl: `type=primitive`, `extra=…`, `verilogprefix=sky130_fd_sc_hd__`.

---

## 3. Verilog-A and mixed-signal flows in ngspice and xschem

| Flow | How it is wired | Where seen |
|---|---|---|
| OSDI loaded at startup | `.spiceinit` lines `osdi '<path>.osdi'`. The PDK `.lib` holds `.model <name> <va_module>` and `.subckt` wrappers with `N…` instances. The symbol is an ordinary `X` subckt. | IHP: psp103, r3_cmc, mosvar, cap_cmomi/f |
| OSDI loaded per netlist | Symbol `device_model="tcleval(… .model M <va_module> … .control pre_osdi <path> .endc)"`, with the instance line on `N`, or `X` to a wrapper subckt | sky130 `reram.sym`; xschem [`ngspice/diff_amp.sym`](https://github.com/StefanSchippers/xschem/blob/efc04f496485f112fd97b69ce907131bc63e4dab/xschem_library/ngspice/diff_amp.sym) (`pre_osdi [abs_sym_path diff_amp.osdi]`, source `.va` sits next to the `.sym`, a launcher runs `openvaf`) |
| Spectre / OSDI | `spectre_device_model="load \"resistor.osdi\" …"` | xschem `devices/res.sym` (15 devices use `spectre_device_model`) |
| Verilog digital via `d_cosim` (ngspice ≥ 42) | `type=primitive`, `format="@name [ @@clk ] [ @@count[3..0] ] @model"` (XSPICE port vectors: inputs `[…]` then outputs `[…]`). Instance `device_model=".model counter d_cosim simulation=\"ivlng\" sim_args=[\"counter\"]"` for Icarus `.vvp`, or `simulation=\"./counter.so\"` for Verilator, built with `ngspice vlnggen counter.v`. Analog/digital bridges are inserted automatically and tuned with `pre_set auto_bridge_d_in/out` in `.control`. | xschem [`ngspice_verilog_cosim/`](https://github.com/StefanSchippers/xschem/tree/efc04f496485f112fd97b69ce907131bc63e4dab/xschem_library/ngspice_verilog_cosim) (counter, sar_adc) |
| XSPICE primitives | `format="@name [ @@s ] [ @@d ] @adc_bridge_model"` | xschem `devices/adc_bridge.sym` |
| Std cells for digital netlists | `verilogprefix` / `verilog_format`, `function<n>` RPN logic for xschem's built-in logic sim | sky130 / IHP std cells |

References: [ngspice OSDI](https://ngspice.sourceforge.io/osdi.html) (OpenVAF, `N` instances), [ngspice manual](https://ngspice.sourceforge.io/docs/ngspice-html-manual/manual.xhtml) (`pre_osdi`, d_cosim §8.4.25 and §10.3), [ngspice Verilog co-sim note](https://ngspice.sourceforge.io/docs/others/Verilog-CoSim.pdf).

---

## 4. Graphics conventions (needed for a faithful import)

| Item | Observation (5 PDK libs, 713 symbols, 2,639 pins) |
|---|---|
| Coordinates | Doubles. Y axis points down (X11 convention). Snap is 10, displayed grid is 20 (`CADSNAP 10`, `CADGRID 20` in `xschem.h`). Instance `rot` is 0–3 in 90° steps; `flip` mirrors X. |
| Pin hot-spot | Center of the `B 5` box. **100% of pin centers sit on the 10-unit grid.** Normal pins are 5×5 boxes (±2.5). |
| MOS bulk pin | A 0.156-wide box at `(20,0)` on the body line, effectively invisible (sky130 30, gf180 13, IHP 8). Keep it as a real pin. |
| Typical extents | 3- and 4-terminal devices: D/C at (20,−30), S/E at (20,30), G/B at (−20,0), bulk/sub at (20,0). Two-terminal devices: (0,±30). Std cells: pins at x=±80 on a 40-unit pitch. |
| Layers | 4 = body lines, arcs and polygons. 5 = pins, plus filled MOS arrow polygons. 7 = pin-name text (sky130/IHP). 13 = parameter-value text. 15/17 = OP annotations (display only). 8 = misc fills. 1 = `@body` text (IHP). Text with no `layer=` uses TEXTLAYER 3. |
| Text | Scale 1.0 means 40 units tall (`FONTHEIGHT 40`, `FONTWIDTH 20`). PDK labels use 0.2 for name/model/params and 0.15 for pin names. 0.07 appears on sky130 OP labels. |
| Label content | `@name`, `@model`, `@spiceprefix@name`, parameter echoes (`@mult x @W / @L`, `w=@w`), `@symname` (std cells). OP annotations use `@spice_get_*`, `@#n:spice_get_voltage` and `tcleval([ngspice::get_node …])`. |

---

## 5. `devicereader` requirements

### 5.1 Inputs
1. A **library directory of `.sym` files**: `sky130_fd_pr`, `gf180mcu_fd_pr` (open_pdks name: `symbols`), `sg13g2_pr`, and std-cell dirs. Read both `K` and the legacy `G`.
2. The **PDK `xschemrc`**, scanned for `set VAR <path-expr>` lines that define model roots (`SKYWATER_MODELS`, `SKYWATER_STDCELLS`, `180MCU_MODELS`, `MODELS_NGSPICE`, `SG13G2_MODELS`, `MODELS_XYCE`). Do **not** execute it.
3. **PDK root and PDK name**, from `--pdk-root`/`--pdk` flags or `$PDK_ROOT`/`$PDK`, used to resolve paths and locate model files.
4. Optional: **`.spiceinit`** (to collect `osdi` lines and `sourcepath`) and **model `.lib` files** (to find which subckts wrap OSDI modules, and to read the `.lib` section names that drive corner choices).
5. Optional: the xschem `devices/` library, if test benches are imported too. It contains `code.sym`, `vsource.sym` and others, with conditional `tcleval` (see 5.4).

### 5.2 Mapping table (XSchem → proposed `.shprim` fields)

| XSchem source | Proposed `.shprim` field | Rule |
|---|---|---|
| file stem, lib dir | `name`, `library` | `nfet_01v8`, `sky130_fd_pr` |
| `type` | `category` (enum: nmos, pmos, res, cap, diode, npn, pnp, inductor, varactor, other) + `xschem_type` (raw) | Normalize sky130's many resistor type strings. `annotator`, `launcher`, `netlist_commands` and `label` are **not primitives**: skip them or route them elsewhere (see 5.4). |
| `template` `name=M1` | `instance_prefix = "M"`, default designator | The first letter is the SPICE device letter for native devices. |
| `template` `spiceprefix=X` + `@spiceprefix` in format | `backing = spice_subckt`, `spice_prefix = "X"` | Absent means a native device (GF180 diodes, sky130 `res_generic_*`). |
| `template` other `k=v` | `params[] {name, default, kind}` | Infer `kind`: number (with or without SI suffix), string (`feed=double`), net (when listed in `extra` or used as `@body`). |
| `model` param + `format` prefix (`sky130_fd_pr__@model`) | `model.default`, `model.spice_name_template` | Store the template, not just the expanded name, so users can change `model`. |
| `available_models` | `params.model.choices` | Split on whitespace. |
| `format` | `netlist.spice.format` | Convert to Schemify's template syntax (5.3). Keep `+` continuation lines. |
| `lvs_format` | `netlist.lvs.format` | Same conversion. Contains `\$` escapes and `tcleval([ev …])`. |
| `spectre_format`, `verilog_format`, `verilogprefix`, `vhdl_format`, `tedax_format` | `netlist.<sim>.format` / `.prefix` | Pass through after conversion. |
| `device_model` | `netlist.spice.once_text` (emitted once per netlist) | Detect `pre_osdi <p>` → `backing = veriloga`, `osdi = <p>`, `va_module` from `.model X <module>`. Detect `d_cosim` → `backing = verilog`. |
| `spice_sym_def` | `netlist.spice.subckt_text` / include | Not used in the surveyed PDK libs (0). |
| `schematic` | — | Not valid on a primitive. Std cells: record `view.schematic` hint, flagged. |
| `spice_ignore` / `lvs_ignore` (true/false/short/open) | `netlist.spice.ignore` | 1 occurrence (GF launcher). |
| `extra="VDD VSS prefix"` | `implicit_pins[]` (net-valued params) + non-net params | Separate net names from plain params by checking `@x` usage position, or keep a list. |
| `drc="proc @args"` | `checks[]` | Declarative min/max per model, hand-ported or regex-scraped from the PDK `switch -regexp` procs. Otherwise keep raw and flag. |
| `function<n>`, pin `goto`, `clock` | `logic` (optional) | xschem's built-in logic sim. Skip for v1. |
| `highlight`, pin `propag` | — | GUI hints. Drop. |
| `B 5` pin | `pins[] {name, dir, order, pos, package_pin}` | `order` = file order, or `sim_pinnumber` sort when present on all pins. `package_pin` = `pinnumber`. `pos` = box center. |
| `L/B/P/A` (non-pin) | `symbol.shapes[] {kind, layer→role, coords, fill, dash}` | Map layers 4/5/7/8/13… to style roles. |
| `T` | `symbol.texts[] {text, pos, rot, flip, size, role}` | Convert `@tok` to field bindings. Drop or replace OP-annotation texts (role `op_annotation`). |
| `v {… version …}` | `provenance {xschem_version, file_version, source_path, source_sha}` | |

### 5.3 Template conversion (no Tcl needed)
- `@name`, `@model`, `@param` → field references. `@pinlist` → ordered pin nets. `@@P` → the net on pin P. `@#n:…` → pin *n* attribute (labels only).
- `\\` after a token → token terminator (emit nothing). `\$` → literal `$`. `[ ]` stay literal (XSPICE vectors).
- Unresolved `@tok` → empty string, matching xschem.
- SPICE-quoted expressions (`'…'`, `{…}`) → emit verbatim.

### 5.4 Hard cases

| Case | Example | Recommended handling |
|---|---|---|
| `expr('…')` computed defaults | sky130 `ad=expr('int((@nf + 1)/2) * @W / @nf * 0.29')` | Clean-room reimplementation of the `expr()` semantics in Schemify's expression language (§5.5). Store as `params.ad.computed = "int((nf+1)/2)*W/nf*0.29"`. At netlist time: if every operand is numeric, print `%.15g`; otherwise emit `'…'` (xschem's fallback). |
| `expr_eng(…)` | IHP `rppd/rhigh/rsil value=` | Same evaluator with engineering formatting. Mainly feeds labels and LVS. |
| `tcleval([ev …])` / `[ev7 …]` | IHP `ntap1/ptap1/pnpMPA` format; `npn13G2l/v`, `schottky_nbl1` lvs_format | The body is pure arithmetic over `@params`. Rewrite at import to the same expression form, with format `%.4g` / `%.7g`. |
| `tcleval(… $::VAR …)` / `$VAR` | `corner.sym`, `reram` `device_model`, all test-bench model blocks | Rewrite to a Schemify PDK variable (`${SKYWATER_MODELS}`) whose value is a `${PDK_ROOT}/${PDK}/…` template taken from `xschemrc`. Resolve at netlist time, not import time, so files stay portable. |
| `[abs_sym_path f]` | xschem `diff_amp.sym` `pre_osdi`, launchers | Resolve at import against the library path. Copy or reference the file (`.va`/`.osdi`/`.v`) next to the `.shprim` and store a relative path. |
| `type=tcleval([symtype])`, `schematic=tcleval([hierarchy_config @symname])` | 84 IHP std cells | Known proc: substitute the default (`subcircuit`, `<stdcell sch dir>/@symname.sch`). The std cell becomes a primitive with `backing = spice_subckt` (from `libs.ref/*/spice`) plus an optional Verilog view. |
| Conditional per-simulator format | xschem `res_noisy.sym` `[if {[sim_is_ngspice]} …]`, `simulator_commands.sym` `[sim_is_@simulator]` | Not in PDK libs. Split into `netlist.ngspice.format` / `netlist.xyce.format`. Anything else: flag. |
| Net-valued params | `@body` (sky130 `nfet3_*`, IHP `rppd`), `extra=` power pins (std cells) | Model as `implicit_pins` with a default net (`GND`, `sub!`, `VDD`). |
| Verilog-A hidden behind subckt | All IHP MOS, poly resistors, MOM caps | The symbol alone is insufficient. Scan `.spiceinit` `osdi` lines and model `.lib` `.model X <module>` to set `requires_osdi[]` on the primitive, or a PDK-wide `sim.preload` list. |
| Pin order vs `pinnumber` | sky130/GF resistors `M P B` with `pinnumber` 2/1/1 | Use file order unless every pin has `sim_pinnumber`. Ignore `pinnumber` for netlists. |
| Unit conventions | sky130 µm without suffix (`W=1`, `scale=1e-6`), GF/IHP SI suffix (`0.28u`, `0.5e-6`) | Keep strings as written. Record `pdk.length_unit` so UI and label evaluation agree. |
| Number formatting parity | `ev` `%.4g` (lossy for `m`), `ev7` `%.7g`, `expr` `%.15g` | Reproduce exactly when golden-diffing netlists against xschem output. |
| Display-only Tcl | `ngspice::get_node`, `subst -nocommand`, `to_eng`, `sky130_display_fet_params`, `cap_cmomi_C`, `@spice_get_*` | Never needed for netlisting. Drop, or map to Schemify's own OP-annotation feature. `cap_cmomi_C` holds a formula that could be ported by hand if the label matters. |
| `drc=` procs | `sky130_fet_drc` (40), IHP `fet/res/mim/hbt/diode/moscap/svaricap_drc` (24) | Table-driven `if {$w < X}` per model regex. Re-express by hand as schema rules in `checks[]` (from the PDK's own rule values), or regex-scrape the arms. Never block the import on these. |
| Three escaping levels | `\\"expr('…')\\"`, `\{@name\\\\\}` | The parser must implement xschem's `get_tok_value` unescaping exactly. Golden-test it against the files listed here. |

### 5.5 Strategy for Tcl without a Tcl runtime
1. **Do not embed Tcl.** Every netlist-relevant Tcl use in the three PDK primitive libraries falls into five fixed patterns (table below). None needs general Tcl.
2. **Import-time classification.** For every `tcleval(…)`, `[…]` and `$…`, classify it as: (a) a known arithmetic proc (`ev`, `ev7`); (b) a known variable from `xschemrc`; (c) a known path or structure proc (`abs_sym_path`, `symtype`, `hierarchy_config`, `sim_is_*`); (d) display-only; or (e) **unknown**. Cases (a)–(c) are rewritten into Schemify's expression and variable syntax. (d) is dropped, with a note in `provenance`. (e) **fails that field**: write the raw string into `needs_review[]` with source file and line, and leave no silently wrong netlist.
3. **One evaluator** shared by import and netlisting. It is Schemify's own expression language, implemented clean-room to match the documented `expr()` behaviour (operators and functions listed in §1) and verified against xschem's evaluated outputs, plus Tcl-`expr` parity for the `ev` bodies. Those bodies use only `+ - * / ( )` and numbers.
4. **Defer when non-numeric.** Evaluate only when all operands are numbers. Otherwise emit a SPICE expression, as xschem does. This keeps hierarchical parameter passing working (`W=W_N`).
5. **PDK manifest** (one per PDK, generated): model-root variables as `${PDK_ROOT}/${PDK}/…` templates, corner `.lib` file and section lists, `.spiceinit` requirements (`ngbehavior=hsa`, `osdi` preloads, `sourcepath`), length unit. Corner selection becomes a Schemify test-bench setting, not a primitive.

### 5.6 Tcl and expression idiom frequency (symbols containing the idiom)

| Idiom | sky130_fd_pr (79) | gf180mcu_fd_pr (68) | sg13g2_pr (45) | sky130 stdcells (437) | sg13g2_stdcell (84) | Total | Needs |
|---|---|---|---|---|---|---|---|
| Plain `@attr` substitution | 78 | 67 | 43 | 437 | 84 | 709 | template conv. |
| `@pinlist` | 77 | 66 | 42 | 0 | 0 | 185 | template conv. |
| `@@pin` explicit net | 0 | 0 | 0 | 423 | 78 | 501 | template conv. |
| `@spiceprefix` | 71 | 57 | 41 | 0 | 0 | 169 | template conv. |
| Net-as-param (`@body`, `extra=`) | 13 | 6 | 4 | 437 | 84 | 544 | implicit pins |
| `\\` token terminator | 0 | 0 | 3 | 437 | 84 | 524 | template conv. |
| SPICE-quoted expr passthrough | 11 | 28 (~128 exprs) | 0 | 0 | 0 | 39 | none (verbatim) |
| `expr('…')` (xschem C arith) | 36 (216 exprs) | 0 | 0 | 0 | 0 | 36 | evaluator |
| `expr_eng(…)` | 0 | 0 | 3 | 0 | 0 | 3 | evaluator |
| `tcleval` + `$::VAR`/`$VAR` | 2 | 0 | 0 | 0 | 0 | 2 | PDK vars |
| `tcleval` + `[ev]`/`[ev7]` (22 calls) | 0 | 0 | 8 | 0 | 0 | 8 | evaluator |
| `tcleval` + `[abs_sym_path]`/`pre_osdi` | 1 | 0 | 0 | 0 | 0 | 1 | path resolve |
| `type=tcleval([symtype])` | 0 | 0 | 0 | 0 | 84 | 84 | known default |
| `schematic=tcleval([hierarchy_config …])` | 0 | 0 | 0 | 0 | 84 | 84 | known default |
| `drc=` Tcl proc | 40 | 0 | 24 | 0 | 0 | 64 | checks / skip |
| Label: `@spice_get_*` / `@#n:spice_get_voltage` | 40 | 0 | 2 | 0 | 0 | 42 | display only |
| Label: `[ngspice::get_node]`+`[subst]`+`[to_eng]` | 0 | 19 | 0 | 0 | 0 | 19 | display only |
| Label: `expr()`/`expr_eng()` | 67 | 0 | 0 | 0 | 0 | 67 | evaluator (labels) |
| Label: PDK Tcl proc | 1 | 0 | 1 | 0 | 0 | 2 | display only |

Test benches (`code`/`code_shown`/`corner` instances): sky130 has 23 tcleval blocks (`$::SKYWATER_MODELS` ×40, `$::SKYWATER_STDCELLS` ×12). GF180 has 58 (`$::180MCU_MODELS` ×536). IHP has 48 (`$::MODELS_NGSPICE` ×62, plus bare `.lib` via ngspice `sourcepath`). `xschemrc` files use `$env(PDK_ROOT|PDK|HOME|PWD|XSCHEM_USER_LIBRARY_PATH)`, `[info exists]`, `[file isdir|normalize|dirname]`, `[info script]`, `[pwd]`, `[glob]` and `source`. Pattern-match only the `set VAR ${PDK_ROOT}/…` lines; the rest is environment discovery that Schemify replaces with its own PDK config.

**Bottom line.** Netlist-relevant Tcl appears in **10 of 192** PDK primitive-library symbols: sky130 `corner` (which is not a device) and `reram`, plus 8 IHP `ev`/`ev7` symbols. Netlist-relevant `expr`/`expr_eng` (C arithmetic, not Tcl) appears in another 39. One rule covers all 84 IHP std cells. A 300-line arithmetic evaluator, a variable table from `xschemrc`, and four hard-coded proc rewrites cover 100% of the surveyed libraries. Anything else goes to `needs_review`.

### 5.7 Open questions for Schemify
- Is a `corner`/model-include a property of the test bench (`.shbench`) or of a PDK manifest? The recommendation is the PDK manifest for roots and sections, with the test bench choosing a section.
- Should std cells become `.shprim` (backed by SPICE subckt plus Verilog module), or `.shcomp` when an xschem schematic exists? IHP ships both views.
- Do `drc` checks have a home in `.shprim` (declarative min/max), or are they deferred?
