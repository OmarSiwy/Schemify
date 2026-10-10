# XSchem pain points (and what users like)

Researched 2026-10-10 for Schemify. Scope: XSchem as used with sky130 / gf180 / IHP SG13G2 and ngspice / Xyce.

## Method and caveats

- **Primary corpus:** all 446 GitHub issues and 23 discussions on [StefanSchippers/xschem](https://github.com/StefanSchippers/xschem), sorted by comment count; issues mentioning xschem in IHP-Open-PDK, gf180mcu-pdk and IIC-OSIC-TOOLS (the JKU Docker image most students use); the SourceForge forum and reviews; blogs; HN.
- **Reddit and HN had almost nothing.** Searches returned no xschem threads on Reddit. HN has a few passing mentions. Most community Q&A lived on the open-source-silicon.dev Slack, but its web mirror domain has lapsed (it now shows a for-sale page). Slack items below come from search-engine snippets only and are marked as such.
- **Bias:** a handful of power users file a large share of the issues (olisnr, slagernate, itsayellow, georgtree, kwmartin). The maintainer is very responsive: **427 of 446 issues are closed**, many of them fixed within days. A complaint here means "this hurt someone at some point", not necessarily "this is still broken". Recurrence matters more than open/closed status.
- **Issue volume is growing:** 37 (2021), 52 (2022), 52 (2023), 106 (2024), 138 (2025), 58 (2026 so far). The user base is growing with Tiny Tapeout, IHP and Chipathon.
- Legend: **Freq** R = recurring (3 or more independent reporters or sources), S = a few (2), O = one-off. **Src** V = verified (quote or close paraphrase at the link), I = my inference.

---

## 1. Platform, install, display stack (X11 + Tk)

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Building from source fails (configure, Tcl/Tk/xpm/cairo detection) | R | The most-commented issue ever is a plain install failure ([#16], 81 comments). Similar: [#81], [#28], [#8], [#48], [#83], [#187], [#212], [#237], [#248], [#153] (a umask breaks the install). About 44 issue titles are install or build related. | V |
| macOS needs XQuartz plus a self-built Tk, and breaks on each new OS release | R | Tahoe: "hangs indefinitely with no window ever appearing… One thread pins at 100% CPU" ([#496], [#449]). Earlier: Big Sur ([#24], [#33]), [#213], [#15]. On macOS `tk_scaling` has no effect on menu text ([#365]). | V |
| Windows is second-class: ActiveTcl must be installed separately, plus Cygwin/X or WSL | R | The MSI can't find ActiveTcl ([#430]); a forum thread asks "ActiveTCL no more download links?" ([SF forum]). Cygwin problems ([#324], [#452]). On Windows the sim button "goes red and… never gets released" ([SF win]). Someone asks for "possibility to compile this under mingw! so that no msvc is required" ([D#147]). | V |
| X11 redraw artifacts, flicker, and crashes in containers or over X forwarding | R | "while moving it produces a mess under the object" ([#119], 56 comments). Menus "blink on and off" over XQuartz ([#321]). Cursor redraw ([#306]). GPU driver slowdown ([#202]). In the IIC-OSIC Docker image: crash on zoom ([IIC#116]), whole-machine freeze on start ([IIC#163]), and the Xming server crashes every 10 minutes ([IIC#92], 57 comments). | V |
| HiDPI/4K: tiny UI, and scaling is partial | R | Reports in [#115], [#399], [#337], [#365] and [IIC#97] (Wayland laptop). `tk_scaling` misses the toolbar, menus and dialogs. Stefan: "Toolbar and context menu buttons are based on bitmap images. Old technology"; icon zoom is integer-only ([#337]). One user gave up and "just configure[d] fonts in xschem.tcl" ([#115]). | V |
| Wayland, theming, fonts | S | Tk dialog font rendering is broken ([#249]). In dark theme the simulator-edit dialogs are "barely readable" ([#178]). There is no native Wayland support; the Xlib drawing engine is documented in the [README]. That Wayland users go through XWayland scaling problems is my inference. | V/I |

## 2. UI/UX and discoverability

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Hard for newcomers and "arcane" (opinions are split) | S | "xschem is fairly arcane to most, it has some graphical problems, and is just in general really hard to use if you're just starting out which makes rapid iteration difficult" (YosysHQ blog; the author built KiCad sky130 libraries instead) ([YosysHQ]). On the other side, SourceForge reviewers call the learning curve "about as light as it gets" for Cadence users ([SF reviews]). | V |
| Modal-dialog friction: one save/netlist/simulate loop takes about 9 steps | R | The loop is: "`s` to save, move mouse over to click 'yes', alt-s… enter… n… esc… s… enter… move mouse over to dismiss 'View data'" ([#390]). Many buttons have no hotkey ([#389]). | V |
| Dialogs size and place badly | R | "I've had to resize… the 'Load File' popup… countless times" ([#459]). Dialog sizes also come up in [D#253] and [D#343]. Pop-ups open behind other windows ([#363]) or on the wrong monitor ([#419]). Tk menus don't dismiss or follow hover like native ones ([#266]). | V |
| Placing a symbol means browsing directories | R | "I don't want to… navigate up and down directories. I have to use my mouse, and it slows me down" ([#329]). Related: [#175], [D#429] (Enter should pick the search hit), [#467] (the new browser errors on a `*` pattern), [#165]. | V |
| No in-canvas parameter editing | S | Users want to click a parameter's value and type over it ([#167], [#487]). | V |
| Behaviour is surprising and undocumented | R | "Clarity for mysterious saving behavior" ([#479]); annotate_op needs a hidden `b` keypress ([#454]); spice_ignore needs a hotkey and a visual mark ([#103], [#433]). | V |

## 3. Keybindings, focus, selection, editing mechanics

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Keyboard focus gets stuck or is lost | R | After ctrl-f "`m` or `q`… wont work" ([#134]). With several windows open, a window "ignores all keystrokes… I must close xschem and restart" ([#348]). Keybind customisation "only has effect for first window" ([#300]). `replace_key` is broken or limited ([#5], [#371], [#457]). Other key problems: [#51], [#346]. | V |
| Selection semantics differ from other CAD tools | R | Users want KiCad-style crossing vs window selection ([#132]) and want plain drag and ctrl-drag swapped ([#122]). Right-to-left selection is inconsistent ([#228], [#274], [#338], [#349]). Hit areas are wrong ([#421], [#411]). Grid snapping makes picking hard ([D#147]). | V |
| Move and stretch are fiddly | R | Labels misbehave on move/stretch ([#124], 45 comments). Users want stretch decoupled from selection ([#347]). Other reports: [#289], [#287], [#400] "make moving… stickier", copy drifts ([#110]), unintended copy ([#261]), stretched buses revert to wires ([#481]). | V |
| Cadence muscle memory is only partly served | S | Missing "connect-by-kissing" and "snap-wire-to-pin"; Stefan: "XSCHEM does 5% of what Cadence is able to do" ([SF lib thread]). Requests for Cadence zoom ([#264]) and the yellow-dot cursor ([#279]); `cadence_compat` has bugs ([#351]). | V |
| Embedded graphs capture the scroll wheel | S | "scrolling is captured and held hostage by the graph" ([#495]). Wheel behaviour changes after annotate ([#240]). | V |

## 4. Tabs, windows, hierarchy navigation, undo

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Tabs arrived late and still have rough edges | R | Requested in 2022 ([#42]). "How do you close a tab?" ([#60]). Open replaces the current tab ([#405]). Closing a tab jumps to the first one ([#456]). Ctrl-Tab history is only one deep ([#472]). A reopened tab lands on top of the current one ([#448]). Users want to move a tab to a new window ([D#415], 12 comments) and get session restore ([D#147]). | V |
| **Undo history is lost when you descend or ascend the hierarchy** | S | "very easy to forget and nuke all changes… VIM/neovim is really great with persisting changes" ([#453], still open). The same user asks for tree-style tabs ([D#478]). | V |
| Hierarchy navigation and scripting | S | Early complaint about having to use File→Open to enter a sub-schematic ([#22]). Users get "locked in" a read-only child ([#271]). There is no simple way to list instances with hierarchical paths from Tcl ([#79]). | V |

## 5. File format and attribute / format-string syntax

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| **"Quoting hell"**: format strings mix Tcl quoting with `@param` substitution | R | Stefan's own phrase. To get a quoted value you write `ctlinst=\"@vnam\\\"` ([D#370]). Single quotes or braces swallow parameters: `WR='F1*@WN1'` becomes `WR='F1*` ([#385]). Other examples: [#221], [#312], [#218], [#482], [#502], and the IHP `~@@pin` question ([D#501]). | V |
| `@symname`, `@spiceprefix` and `@model` interact in surprising ways | R | "My guess is that @symname is required in format lines and I couldn't see this mentioned in the documentation" ([#375]). spiceprefix has no effect ([#378]). Getting KLayout-LVS netlists requires turning *off* spiceprefix ([IIC#197]). | V |
| The attribute grammar is underspecified | S | People ask what `+` and `\|` mean inside attribute lists; the answer was a "spice netlist cognitive bias" plus a typo ([#186]). The xschem-viewer author had to ask which characters are legal in keys before writing a parser. | V |
| Each tool variant needs its own format attribute | R | `format`, `lvs_format`, `spectre_format`, and conditional formats per simulator ([#114], [#76]). LVS mode used the wrong attribute ([#352]). LVS export changes 4-terminal MOS to 3 terminals ([#231]). spice_ignore=short trick for ammeters ([#233]). | V |
| PDK symbol format strings are a bug surface | R | gf180 diode symbols drop W/L ([gf#124]). IHP pnpMPA pin order is wrong ([IHP#37]). m-factor confusion ([gf#88], [IHP#25], [IHP#276]). Inductor not netlisted for LVS ([IHP#872]). NMOS symbol default has source up ([IHP#390], 32 comments). | V |
| Only `[]` bus syntax; `<>` from other tools is not expanded | O | [#112] | V |
| `gnd.sym` uses `lab=GND`, not `0`, and breaks Xyce | O | "fail with cryptic convergence errors" ([#444]) | V |
| Generic `.sch`/`.sym` extensions collide with KiCad and others in file managers | O | [D#147] (olisnr) | V |
| Text format exists but diffs are noisy | S | "`git diff` on an Xschem schematic shows moved coordinates" (Riku README; Riku is a semantic-diff tool built because of this) ([Riku]). The rendering library exists because "rendering… has always required the xschem application running in a GUI or headless X11 session" ([xv-rust]). | V |
| No schematic interchange format | S | EDIF was rejected; Stefan agrees "there is a gap for schematics" ([D#358]). OpenAccess PDKs make it "practically impossible to use OS tools" ([D#350]). Users want custom metadata in symbols ([#308]). | V |
| Parsing Tcl generator symbols spams the console | O | Thousands of `SKIP RECORD` lines ([#304], 40 comments) | V |

## 6. Executable Tcl inside design data (`tcleval`, xschemrc, launchers)

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| `tcleval()` in symbols deadlocks or breaks in headless/CI runs | R | Deadlock at load ([#505]). A Tk focus call breaks `--no_x` ([#494]). A regression made PDK OP annotators show NaN ([#498], 16 comments; [IIC#342]). fet_drc tcleval fails ([#198]). Windows passes two args ([#216]). Brackets need double escaping ([#353]). | V |
| **Opening a schematic and pressing Simulate can run arbitrary shell commands** | O (but critical) | Reported as a "critical security vulnerability". Stefan: files "may contain tcl commands, just like an ordinary tcl script"; he is now moving embedded scripts to a safe slave interpreter ([#492]). | V |
| `xschemrc` is a Turing-complete Tcl script with poor error messages | S | "Better error message when error encountered in xschemrc" ([#286]). Using a never-set variable in xschemrc is a common Slack error (search snippet; I couldn't confirm which thread, likely [OSS-slack]). | V |

## 7. Library management, symbol/schematic split, PDK paths (top theme)

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| **A single flat `XSCHEM_LIBRARY_PATH` with no namespaces** | R | "doesn't scale well because I might have different projects all with `opamp.sch`… which is quite annoying" ([#504]). Making each library its own path entry "seems like a recipe for disaster (getting the component from the wrong library)" ([#329]). | V |
| Absolute paths get saved, which breaks sharing and git | R | "Sharing schematics by moving them to a different folder or uploading them to GitHub becomes difficult" ([#54]). The fix is to add `.` to the path, where `.` means "the directory of the parent schematic". Users can't find project symbols without absolute paths ([#234]). | V |
| **MISSING SYMBOL after a PDK or container update** | R | IHP changed its xschemrc and every old schematic lost its symbols. The workaround was to edit the prefix on every symbol or chain env vars ([IIC#257], [IHP commit]). Other cases: [IIC#219] (examples), [IIC#9] (gf180 setup), [gf#90] (wrong test path), [#153] "It used to work". Slack calls it "a pandemonium of xschem path definition inconsistencies" that "take forever to sort out" (search snippet, [OSS-slack-pand]). | V |
| Library management relies on the OS | S | For library organisation (common vs project libraries), Stefan recommends symlinks as the analogue of `cds.lib`; the user found them "a bit awkward" ([SF lib thread]). | V |
| No config view: binding symbols to alternate schematics is hard to manage | S | Users want "a 'view manager' where you can see the list of all symbols, and edit the schematic that they are supposed to use… Some commercial tools use a 'config' view" ([#242], open). Testbench reuse means maintaining copies ([#12]). Xschem assumes the schematic sits next to the symbol ([#58]). Revolution EDA ships a config view ([RevEDA]). | V |
| Model and corner wiring is manual | R | Models not found or `.include` failures ([#7], [#50], [#224]). How to sweep `.lib … mos_tt/ss/ff` corners ([#144], 100 comments, the most-discussed thread). | V |
| Creating a symbol from a schematic silently overwrites the existing symbol | O | [#113] | V |

## 8. Netlisting

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Schematics can't be parameterised (net/bus/array counts from a param) | S | "I've tried every method… but nothing works" ([#353], 45 comments). The answer is to write a Tcl/Python generator script, i.e. a pcell. Parameterised FET sizing breaks LVS ([#450]). | V |
| No import (SPICE/Verilog to schematic) | R | Requested in [#35], [#38], [#407], [#441], [#66]; Viewdraw import in [#39]. | V |
| Batch netlisting used to fork and return meaningless exit codes | S | [#25] (fixed via flags). A `--script` error still hangs headless instead of exiting non-zero ([#510], open). | V |
| Silent or odd netlist behaviour | S | Wire names change ([#323]). Duplicated expanded names give no warning ([#111]). A floating pin is only a warning ([#391]). False warnings ([#32]). | V |
| New Spectre/VACASK backend is immature | O | Hierarchical spectre "not working" ([#461]) | V |

## 9. Simulation and waveform viewing

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Viewer embedded in the schematic has limited features | R | Users want a standalone multi-raw viewer with sheets and panels ([#362], [D#506]). Requested features: two y-axes ([#438]), Smith/polar ([#376]), a 3rd cursor ([#497]), more colours ([#416]), horizontal cursors ([#275]), per-graph cursors ([#217], [D#147]). Signal names overlap ([#477]). | V |
| Crashes when loading .raw files | R | Crash while Xyce is still writing ([#239]), `free(): invalid next size` ([#439]), [#413], [#327], [#244]. | V |
| Weak simulator process control | R | No way to know when a sim finished ([#144], [#200]). The sim button stays red ([#141], [SF win]). Users want early termination ([#21]). Process-list Tcl errors ([#412]). On HN: "Ngspice is okay but it's a bit clumsy switching between it and xschem" ([HN]). | V |
| No ADE-style corner, Monte Carlo or sweep management | R | [#144] (100 comments) and the 2D-sweep API request ([#436]). Efabless/Tim Edwards built CACE to add "a structure around analog design" with corners and Monte Carlo on top of xschem testbenches ([CACE]). Calling this the biggest gap versus Virtuoso ADE is my inference. | V/I |
| PDF/SVG export | R | Waveforms missing from PDF ([#62], 56 comments). [#164] (22). Inconsistent SVG ([#475]). Hierarchical PDF ([#397]). Non-searchable text ([#4]). | V |

## 10. Mixed-signal, Verilog, Verilog-A, RF

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| ngspice + Verilog cosimulation is hard to set up | R | The example "could be simpler" ([#409]). ~2 ns timing offset ([#422]). Verilator flow not working ([#383]). The built-in digital sim has "No comprehensive docs" ([#393]). | V |
| Verilog netlister: parameters, arrays, `include | R | [#11], [#18], [#56], [#87], [#145], [#84] (mostly 2021–22) | V |
| Verilog-A: no symbol-from-`.va` workflow | I | No xschem issue covers it. Revolution EDA advertises "Automatic Symbol Generation from… Verilog-A modules" and a "clear separation between model and instance parameters" ([RevEDA]). | I |
| RF: no S-parameter/Touchstone blocks, no RF simulators | S | [#377], [#361], [#171] | V |

## 11. Scripting and automation

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| The Tcl command API is underdocumented | R | "there doesn't seem to be a comprehensive doc for the commands" ([#52]). The maintainer's own TODO: document the Tcl extensions ([#59]). Also [#345]. | V |
| Headless and remote modes are fragile | R | Script errors hang the process ([#510]). TCP channel problems ([#116], [#91], [#310], [#508]). Scripted saves silently fail ([#479]). | V |
| Demand for programmatic construction and agent editing | S | Request for an "API to programmatically draw wires and place components… LLM chat based… design assistant" ([D#147]). In [#483], agents (Claude Code, Cursor) editing .sch files make the overwrite-on-save footgun worse. | V |
| Tcl only, while ecosystem tooling is Python | I | PDK/CACE/gdsfactory are Python; Stefan suggests "tcl generator (or python if you like)" via external scripts ([#353]); Revolution EDA's pitch is Python-native. | I |

## 12. Data safety, collaboration, version control

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Silent data loss | R | Saving overwrites on-disk changes without warning when `no_ask_save` is set; git checkout/pull/stash and agents trigger it ([#483], partly fixed; the user still wants detection on focus-in "like vim/nvim/VS Code"). The same file open in two tabs isn't marked dirty ([#423]). Undo is lost on hierarchy moves ([#453]). Symbol overwrite ([#113]). A container crash loses unsaved work ([IIC#116]). Open tabs are lost when the window manager dies ([D#147]). | V |
| Hard to collaborate through git | R | Absolute paths ([#54]), path breakage across machines and containers (§7), coordinate-noise diffs ([Riku]). Executable files are a supply-chain risk for shared course material ([#492]). | V |

## 13. Documentation

| Pain point | Freq | Evidence | Src |
|---|---|---|---|
| Key behaviour lives in videos, Slack and the maintainer's head | R | Gaps reported in [#52], [#59], [#375], [#393], [#479], [D#506] ("would you be able to share a minimum reproducible example… or updated docs"). Stefan: "Some just don't read the documentation" ([interview]). | V |

## 14. Performance (few complaints)

There are almost no complaints about speed. The exceptions are a GPU driver regression ([#202]) and slow X forwarding or Docker setups ([IIC#163]). Performance is a strength to keep (see below).

---

## What users like (don't regress these)

| Strength | Evidence |
|---|---|
| **Speed**: instant netlisting, big designs | The drawing engine is C on Xlib, with "very good speed performance, even on very big circuits" ([README]). "In cadence even a small amplifier with… 20 transistors, took 30 seconds to get a netlist" ([interview]). "netlist generation is very fast" ([SF reviews]). The manual targets "5-8 levels of hierarchy and 1000K+ transistors" ([what-is]). |
| **Keyboard-driven, Cadence-like** | "If you're a Cadence user this schematic editor will feel pretty natural"; "learning curve is about as light as it gets" ([SF reviews]). "UI is very intuitive when it comes to drawing / netlisting and simulating" ([#362]). |
| **Plain-text, small, documented format** | "all the files saved for schematic symbols are text files" ([interview]). The format spec is "less than 8 pdf pages" ([D#350]). Third parties (xschem-viewer, Riku, a Rust parser, Revolution EDA's importer) parse it. |
| **Scriptable and hackable** | Tcl customisation "to his own taste" ([Hackaday]). "not as powerful and scriptable as something like xschem" is the KiCad author's concession ([YosysHQ]). Per-project `xschemrc`. Generators/pcells via scripts ([#353]). |
| **Embedded graphs with live back-annotation** | Move a cursor in a graph and see voltages and currents annotated on the schematic ([D#506]). Reaction: "This is awesome!… never thought of this!" |
| **Many netlist targets** | SPICE (ngspice/Xyce dialects), Verilog, VHDL, tEDAx, LVS mode, and now Spectre/VACASK via per-format attributes ([D#370]). Bus and vector notation and instance arrays. |
| **PDK ecosystem** | Ships symbol libraries for sky130/gf180/IHP. "The recommended schematic entry tool… well integrated with the SKY130 PDK" ([SSCS]). "Most popular… look and feel similar to commercial tools" ([OCD]). |
| **Low dependencies, runs anywhere Tk runs** | "a very low dependency list" ([interview]). Bitmap icons were chosen so it "run[s] unchanged on all Tk… released in the last 20 years" ([#337]). |
| **Maintainer responsiveness** | 427 of 446 issues closed. "excellent support from the lead developer" ([SSCS]). Multiple SourceForge reviews say the same. |
| **Batch CLI** | `xschem -n -x -s -q -o …` netlisting is used in flows ([OCD]). |

## Revolution EDA: what its author actually says

I found **no public statement by Murat Eskiyerli blaming xschem**. Searches covered the README, reveda.eu, the release discussions and SemiWiki. His stated motivation is commercial-EDA "high cost, inflexibility, and vendor lock-in" ([RevEDA-about]). His feature list does read like a list of xschem gaps, though (my inference):

- a Cadence-style lib/cell/view browser;
- config views;
- JSON files;
- Python instance-parameter callbacks;
- Verilog-A symbol generation;
- a PDK registry for "switching PDKs without editing configuration files manually";
- a plugin registry;
- per-project config.

It also ships an **xschem `.sym` importer** ([RevEDA]). A Qucs-S PDK discussion cites Revolution EDA's Python callbacks as the reference design for PDK parameter callbacks ([qucs#849]).

---

## Implications for Schemify

**The minimal core MUST fix these.** Each is either the top recurring complaint or something plugins can't fix after the fact.

1. **Library resolution.** Use namespaced references (`lib:cell[:view]`) resolved through a declarative project manifest, never a Turing-complete rc file. References must be project-relative by construction, with a remap/alias table so a PDK update doesn't orphan old designs. Missing symbols need loud, actionable diagnostics. (§7)
2. **Data safety.**
   - Per-document undo that survives hierarchy descend/ascend.
   - External-change detection: file watcher plus a hash check on focus.
   - Atomic saves, autosave/crash recovery, session restore.
   - Dirty state shared across views of the same file. (§4, §12)
3. **A specified, deterministic, diff-friendly file format.**
   - Stable ordering and a canonical serializer.
   - Versioned schema.
   - **No executable code in design files.** Any expression language is sandboxed and pure. This matters doubly for WASM and shared links (§6, [#492]).
   - Ship an xschem `.sch`/`.sym` importer early; without it nobody migrates their PDK libraries.
4. **A netlist templating language with a real grammar.** It needs typed parameters, an explicit escape syntax (no "quoting hell"), and errors that point at the symbol and attribute. "Which view/format is used for which target" (sim vs LVS vs simulator dialect, config views) must be one core mechanism, not N ad-hoc attributes. The SPICE backend belongs in core. (§5, §7, §8)
5. **Rendering and platform.**
   - GPU/vector UI with fractional HiDPI and themes.
   - Native Wayland/Windows/macOS with no X server, no separate Tcl install, and single-binary install.
   - Same core in WASM. (§1)
6. **Input model.** A modal, fully remappable keymap with deterministic focus (no modal dialogs that steal keys), a command palette, and non-blocking inline edits. Selection semantics (window vs crossing, snap-while-selecting) should be configurable. (§2, §3)
7. **Headless-first API.** One documented command API shared by the CLI, scripting and plugins. Batch mode never blocks on UI and returns real exit codes. That API is also the agent/LLM surface. (§8, §11)
8. **Performance parity** with xschem: instant netlist, huge hierarchies. Set budgets and benchmark against xschem examples. (Likes)

**These can live in plugins; core only exposes the hooks.**

| Plugin | Core hook it needs |
|---|---|
| Waveform viewer | Core provides a raw/PSF data model plus a cursor → back-annotation event bus, so the beloved embedded graphs and annotation remain possible. |
| Simulator drivers (ngspice shared-lib, Xyce, VACASK/Spectre) | Core provides an async job runner with status, cancel and completion events (fixes the "is my sim done?" class, §9). |
| Corner, Monte Carlo and sweep manager (ADE-like), CACE integration | — |
| Extra netlisters (Verilog, VHDL, Spectre, tEDAx, CDL) | — |
| Netlist→schematic import, OpenAccess/Qucs/KiCad interchange | — |
| Verilog-A symbol generation, RF/Touchstone blocks | — |
| PDK packs (symbols + parameter callbacks) | Core sandbox for callbacks |
| PDF and other exports | SVG export probably belongs in core |
| Semantic schematic diff (Riku-style) | Enabled by the deterministic format |
| Cadence-compat keymap and other keymap presets | — |
| Cosimulation (Verilator/Icarus + ngspice) | — |

---

[#4]: https://github.com/StefanSchippers/xschem/issues/4
[#5]: https://github.com/StefanSchippers/xschem/issues/5
[#7]: https://github.com/StefanSchippers/xschem/issues/7
[#8]: https://github.com/StefanSchippers/xschem/issues/8
[#11]: https://github.com/StefanSchippers/xschem/issues/11
[#12]: https://github.com/StefanSchippers/xschem/issues/12
[#15]: https://github.com/StefanSchippers/xschem/issues/15
[#16]: https://github.com/StefanSchippers/xschem/issues/16
[#18]: https://github.com/StefanSchippers/xschem/issues/18
[#21]: https://github.com/StefanSchippers/xschem/issues/21
[#22]: https://github.com/StefanSchippers/xschem/issues/22
[#24]: https://github.com/StefanSchippers/xschem/issues/24
[#25]: https://github.com/StefanSchippers/xschem/issues/25
[#28]: https://github.com/StefanSchippers/xschem/issues/28
[#32]: https://github.com/StefanSchippers/xschem/issues/32
[#33]: https://github.com/StefanSchippers/xschem/issues/33
[#35]: https://github.com/StefanSchippers/xschem/issues/35
[#38]: https://github.com/StefanSchippers/xschem/issues/38
[#39]: https://github.com/StefanSchippers/xschem/issues/39
[#42]: https://github.com/StefanSchippers/xschem/issues/42
[#48]: https://github.com/StefanSchippers/xschem/issues/48
[#50]: https://github.com/StefanSchippers/xschem/issues/50
[#51]: https://github.com/StefanSchippers/xschem/issues/51
[#52]: https://github.com/StefanSchippers/xschem/issues/52
[#54]: https://github.com/StefanSchippers/xschem/issues/54
[#56]: https://github.com/StefanSchippers/xschem/issues/56
[#58]: https://github.com/StefanSchippers/xschem/issues/58
[#59]: https://github.com/StefanSchippers/xschem/issues/59
[#60]: https://github.com/StefanSchippers/xschem/issues/60
[#62]: https://github.com/StefanSchippers/xschem/issues/62
[#66]: https://github.com/StefanSchippers/xschem/issues/66
[#76]: https://github.com/StefanSchippers/xschem/issues/76
[#79]: https://github.com/StefanSchippers/xschem/issues/79
[#81]: https://github.com/StefanSchippers/xschem/issues/81
[#83]: https://github.com/StefanSchippers/xschem/issues/83
[#84]: https://github.com/StefanSchippers/xschem/issues/84
[#87]: https://github.com/StefanSchippers/xschem/issues/87
[#91]: https://github.com/StefanSchippers/xschem/issues/91
[#103]: https://github.com/StefanSchippers/xschem/issues/103
[#110]: https://github.com/StefanSchippers/xschem/issues/110
[#111]: https://github.com/StefanSchippers/xschem/issues/111
[#112]: https://github.com/StefanSchippers/xschem/issues/112
[#113]: https://github.com/StefanSchippers/xschem/issues/113
[#114]: https://github.com/StefanSchippers/xschem/issues/114
[#115]: https://github.com/StefanSchippers/xschem/issues/115
[#116]: https://github.com/StefanSchippers/xschem/issues/116
[#119]: https://github.com/StefanSchippers/xschem/issues/119
[#122]: https://github.com/StefanSchippers/xschem/issues/122
[#124]: https://github.com/StefanSchippers/xschem/issues/124
[#132]: https://github.com/StefanSchippers/xschem/issues/132
[#134]: https://github.com/StefanSchippers/xschem/issues/134
[#141]: https://github.com/StefanSchippers/xschem/issues/141
[#144]: https://github.com/StefanSchippers/xschem/issues/144
[#145]: https://github.com/StefanSchippers/xschem/issues/145
[#153]: https://github.com/StefanSchippers/xschem/issues/153
[#164]: https://github.com/StefanSchippers/xschem/issues/164
[#165]: https://github.com/StefanSchippers/xschem/issues/165
[#167]: https://github.com/StefanSchippers/xschem/issues/167
[#171]: https://github.com/StefanSchippers/xschem/issues/171
[#175]: https://github.com/StefanSchippers/xschem/issues/175
[#178]: https://github.com/StefanSchippers/xschem/issues/178
[#186]: https://github.com/StefanSchippers/xschem/issues/186
[#187]: https://github.com/StefanSchippers/xschem/issues/187
[#198]: https://github.com/StefanSchippers/xschem/issues/198
[#200]: https://github.com/StefanSchippers/xschem/issues/200
[#202]: https://github.com/StefanSchippers/xschem/issues/202
[#212]: https://github.com/StefanSchippers/xschem/issues/212
[#213]: https://github.com/StefanSchippers/xschem/issues/213
[#216]: https://github.com/StefanSchippers/xschem/issues/216
[#217]: https://github.com/StefanSchippers/xschem/issues/217
[#218]: https://github.com/StefanSchippers/xschem/issues/218
[#221]: https://github.com/StefanSchippers/xschem/issues/221
[#224]: https://github.com/StefanSchippers/xschem/issues/224
[#228]: https://github.com/StefanSchippers/xschem/issues/228
[#231]: https://github.com/StefanSchippers/xschem/issues/231
[#233]: https://github.com/StefanSchippers/xschem/issues/233
[#234]: https://github.com/StefanSchippers/xschem/issues/234
[#237]: https://github.com/StefanSchippers/xschem/issues/237
[#239]: https://github.com/StefanSchippers/xschem/issues/239
[#240]: https://github.com/StefanSchippers/xschem/issues/240
[#242]: https://github.com/StefanSchippers/xschem/issues/242
[#244]: https://github.com/StefanSchippers/xschem/issues/244
[#248]: https://github.com/StefanSchippers/xschem/issues/248
[#249]: https://github.com/StefanSchippers/xschem/issues/249
[#261]: https://github.com/StefanSchippers/xschem/issues/261
[#264]: https://github.com/StefanSchippers/xschem/issues/264
[#266]: https://github.com/StefanSchippers/xschem/issues/266
[#271]: https://github.com/StefanSchippers/xschem/issues/271
[#274]: https://github.com/StefanSchippers/xschem/issues/274
[#275]: https://github.com/StefanSchippers/xschem/issues/275
[#279]: https://github.com/StefanSchippers/xschem/issues/279
[#286]: https://github.com/StefanSchippers/xschem/issues/286
[#287]: https://github.com/StefanSchippers/xschem/issues/287
[#289]: https://github.com/StefanSchippers/xschem/issues/289
[#300]: https://github.com/StefanSchippers/xschem/issues/300
[#304]: https://github.com/StefanSchippers/xschem/issues/304
[#306]: https://github.com/StefanSchippers/xschem/issues/306
[#308]: https://github.com/StefanSchippers/xschem/issues/308
[#310]: https://github.com/StefanSchippers/xschem/issues/310
[#312]: https://github.com/StefanSchippers/xschem/issues/312
[#321]: https://github.com/StefanSchippers/xschem/issues/321
[#323]: https://github.com/StefanSchippers/xschem/issues/323
[#324]: https://github.com/StefanSchippers/xschem/issues/324
[#327]: https://github.com/StefanSchippers/xschem/issues/327
[#329]: https://github.com/StefanSchippers/xschem/issues/329
[#337]: https://github.com/StefanSchippers/xschem/issues/337
[#338]: https://github.com/StefanSchippers/xschem/issues/338
[#345]: https://github.com/StefanSchippers/xschem/issues/345
[#346]: https://github.com/StefanSchippers/xschem/issues/346
[#347]: https://github.com/StefanSchippers/xschem/issues/347
[#348]: https://github.com/StefanSchippers/xschem/issues/348
[#349]: https://github.com/StefanSchippers/xschem/issues/349
[#351]: https://github.com/StefanSchippers/xschem/issues/351
[#352]: https://github.com/StefanSchippers/xschem/issues/352
[#353]: https://github.com/StefanSchippers/xschem/issues/353
[#361]: https://github.com/StefanSchippers/xschem/issues/361
[#362]: https://github.com/StefanSchippers/xschem/issues/362
[#363]: https://github.com/StefanSchippers/xschem/issues/363
[#365]: https://github.com/StefanSchippers/xschem/issues/365
[#371]: https://github.com/StefanSchippers/xschem/issues/371
[#375]: https://github.com/StefanSchippers/xschem/issues/375
[#376]: https://github.com/StefanSchippers/xschem/issues/376
[#377]: https://github.com/StefanSchippers/xschem/issues/377
[#378]: https://github.com/StefanSchippers/xschem/issues/378
[#383]: https://github.com/StefanSchippers/xschem/issues/383
[#385]: https://github.com/StefanSchippers/xschem/issues/385
[#389]: https://github.com/StefanSchippers/xschem/issues/389
[#390]: https://github.com/StefanSchippers/xschem/issues/390
[#391]: https://github.com/StefanSchippers/xschem/issues/391
[#393]: https://github.com/StefanSchippers/xschem/issues/393
[#397]: https://github.com/StefanSchippers/xschem/issues/397
[#399]: https://github.com/StefanSchippers/xschem/issues/399
[#400]: https://github.com/StefanSchippers/xschem/issues/400
[#405]: https://github.com/StefanSchippers/xschem/issues/405
[#407]: https://github.com/StefanSchippers/xschem/issues/407
[#409]: https://github.com/StefanSchippers/xschem/issues/409
[#411]: https://github.com/StefanSchippers/xschem/issues/411
[#412]: https://github.com/StefanSchippers/xschem/issues/412
[#413]: https://github.com/StefanSchippers/xschem/issues/413
[#416]: https://github.com/StefanSchippers/xschem/issues/416
[#419]: https://github.com/StefanSchippers/xschem/issues/419
[#421]: https://github.com/StefanSchippers/xschem/issues/421
[#422]: https://github.com/StefanSchippers/xschem/issues/422
[#423]: https://github.com/StefanSchippers/xschem/issues/423
[#430]: https://github.com/StefanSchippers/xschem/issues/430
[#433]: https://github.com/StefanSchippers/xschem/issues/433
[#436]: https://github.com/StefanSchippers/xschem/issues/436
[#438]: https://github.com/StefanSchippers/xschem/issues/438
[#439]: https://github.com/StefanSchippers/xschem/issues/439
[#441]: https://github.com/StefanSchippers/xschem/issues/441
[#444]: https://github.com/StefanSchippers/xschem/issues/444
[#448]: https://github.com/StefanSchippers/xschem/issues/448
[#449]: https://github.com/StefanSchippers/xschem/issues/449
[#450]: https://github.com/StefanSchippers/xschem/issues/450
[#452]: https://github.com/StefanSchippers/xschem/issues/452
[#453]: https://github.com/StefanSchippers/xschem/issues/453
[#454]: https://github.com/StefanSchippers/xschem/issues/454
[#456]: https://github.com/StefanSchippers/xschem/issues/456
[#457]: https://github.com/StefanSchippers/xschem/issues/457
[#459]: https://github.com/StefanSchippers/xschem/issues/459
[#461]: https://github.com/StefanSchippers/xschem/issues/461
[#467]: https://github.com/StefanSchippers/xschem/issues/467
[#472]: https://github.com/StefanSchippers/xschem/issues/472
[#475]: https://github.com/StefanSchippers/xschem/issues/475
[#477]: https://github.com/StefanSchippers/xschem/issues/477
[#479]: https://github.com/StefanSchippers/xschem/issues/479
[#481]: https://github.com/StefanSchippers/xschem/issues/481
[#482]: https://github.com/StefanSchippers/xschem/issues/482
[#483]: https://github.com/StefanSchippers/xschem/issues/483
[#487]: https://github.com/StefanSchippers/xschem/issues/487
[#492]: https://github.com/StefanSchippers/xschem/issues/492
[#494]: https://github.com/StefanSchippers/xschem/issues/494
[#495]: https://github.com/StefanSchippers/xschem/issues/495
[#496]: https://github.com/StefanSchippers/xschem/issues/496
[#497]: https://github.com/StefanSchippers/xschem/issues/497
[#498]: https://github.com/StefanSchippers/xschem/issues/498
[#502]: https://github.com/StefanSchippers/xschem/issues/502
[#504]: https://github.com/StefanSchippers/xschem/issues/504
[#505]: https://github.com/StefanSchippers/xschem/issues/505
[#508]: https://github.com/StefanSchippers/xschem/issues/508
[#510]: https://github.com/StefanSchippers/xschem/issues/510
[D#147]: https://github.com/StefanSchippers/xschem/discussions/147
[D#253]: https://github.com/StefanSchippers/xschem/discussions/253
[D#343]: https://github.com/StefanSchippers/xschem/discussions/343
[D#350]: https://github.com/StefanSchippers/xschem/discussions/350
[D#358]: https://github.com/StefanSchippers/xschem/discussions/358
[D#370]: https://github.com/StefanSchippers/xschem/discussions/370
[D#415]: https://github.com/StefanSchippers/xschem/discussions/415
[D#429]: https://github.com/StefanSchippers/xschem/discussions/429
[D#478]: https://github.com/StefanSchippers/xschem/discussions/478
[D#501]: https://github.com/StefanSchippers/xschem/discussions/501
[D#506]: https://github.com/StefanSchippers/xschem/discussions/506
[IIC#9]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/9
[IIC#92]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/92
[IIC#97]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/97
[IIC#116]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/116
[IIC#163]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/163
[IIC#197]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/197
[IIC#219]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/219
[IIC#257]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/257
[IIC#342]: https://github.com/iic-jku/IIC-OSIC-TOOLS/issues/342
[IHP#25]: https://github.com/IHP-GmbH/IHP-Open-PDK/issues/25
[IHP#37]: https://github.com/IHP-GmbH/IHP-Open-PDK/issues/37
[IHP#276]: https://github.com/IHP-GmbH/IHP-Open-PDK/issues/276
[IHP#390]: https://github.com/IHP-GmbH/IHP-Open-PDK/issues/390
[IHP#872]: https://github.com/IHP-GmbH/IHP-Open-PDK/issues/872
[IHP commit]: https://github.com/IHP-GmbH/IHP-Open-PDK/commit/2bda257623753d0571bc40c5f50481e8389309e0
[gf#88]: https://github.com/google/gf180mcu-pdk/issues/88
[gf#124]: https://github.com/google/gf180mcu-pdk/issues/124
[gf#90]: https://github.com/google/globalfoundries-pdk-libs-gf180mcu_fd_pr/issues/90
[qucs#849]: https://github.com/ra3xdh/qucs_s/discussions/849
[README]: https://github.com/StefanSchippers/xschem/blob/master/README
[what-is]: https://xschem.sourceforge.io/stefan/xschem_man/what_is_xschem.html
[interview]: https://www.openedhardware.com/p/xschem-and-the-chip-revolution
[SF reviews]: https://sourceforge.net/projects/xschem/reviews/
[SF forum]: https://sourceforge.net/p/xschem/discussion/general/
[SF win]: https://sourceforge.net/p/xschem/discussion/general/thread/f841fe5958/
[SF lib thread]: https://sourceforge.net/p/xschem/discussion/general/thread/fbc3ef93/
[YosysHQ]: https://blog.yosyshq.com/p/sky130-kicad-spice/
[HN]: https://news.ycombinator.com/item?id=49672641
[Hackaday]: https://hackaday.com/2025/10/08/the-entire-process-of-building-an-open-source-analog-asic/
[SSCS]: https://sscs-ose.github.io/analog/
[OCD]: https://opencircuitdesign.com/analog_flow/index.html
[CACE]: https://wiki.f-si.org/images/7/7d/Cace_fsic_2024.pdf
[Riku]: https://github.com/riku-chip/riku_chip
[xv-rust]: https://github.com/carloscl03/xschem-viewer-rust
[RevEDA]: https://github.com/eskiyerli/revolution-eda
[RevEDA-about]: https://reveda.eu/about/
[OSS-slack]: https://web.open-source-silicon.dev/t/425094/hi-i-have-installed-open-pdks-but-when-i-launch-xschem-i-get
[OSS-slack-pand]: https://web.open-source-silicon.dev/t/16764669/u0172qz342d-with-the-web-browser-xschem-viewer-is-the-xschem
