# Commercial custom-IC schematic environments vs XSchem: feature catalog

Status: research, 2026-10-10. Scope: what Cadence Virtuoso (Schematic Editor, ADE Explorer/Assembler/Verifier, ViVA, AMS Designer), Synopsys Custom Compiler + PrimeWave, Siemens Tanner S-Edit / Pyxis / Solido and Keysight ADS do that XSchem doesn't. For each feature: whether an open-source tool already does it, and whether Schemify should put it in core, behind a core hook, in a plugin, or leave it out.

Companion docs: [xschem-complaints.md](xschem-complaints.md) (pain points, core vs plugin), [pdk-xschem-devices.md](pdk-xschem-devices.md) (PDK symbol import).

## Legend

**Commercial:** **V** Cadence Virtuoso. **CC** Synopsys Custom Compiler / PrimeWave. **SE** Siemens S-Edit / Pyxis / Solido. **ADS** Keysight ADS.
**Open:** **XS** XSchem. **RE** Revolution EDA. **QS** Qucs-S. **CACE**. **KL** KLayout. **PO** PyOPUS. **ALIGN**. **BAG**. **OF** OpenFASoC. **NG** ngspice. **XY** Xyce.

**Verdict:**
- **Core**: in the minimal core.
- **Hook**: the feature is a plugin, but the core data model or API must carry something for it (listed in the "Core DM impact" column). Changing this later breaks file formats.
- **Plugin**: no special core support beyond the generic API.
- **Out**: not a schematic tool's job. Integrate through files or IPC.

Caveat: Cadence datasheets returned HTTP 403 to the fetcher, so Cadence claims rest on search snippets of those pages, Cadence blogs and forums, and older public user guides. Most of these features have existed since IC6.1.x and are stable. Anything marked "IC25.1" is recent.

---

## 1. PDK integration (CDF, callbacks, tech data)

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **CDF parameter schema** | Per-cell typed params: string, int, float, radio, cyclic, boolean, button, netSet. Each has a prompt, default, units, choices, and `display`/`editable` predicates, which are expressions that show or grey out a field depending on other fields. Base and user CDF layers sit at lib and cell level. ([cdf-ug]) | V. CC and SE via iPDK/iCDF ([ipdk], [ipl]) | RE: Python-function params ([re-docs]). XS: free-form `key=value` with no types. KL: PCell declarations (layout only) | **Core** | A param schema attached to each cell (not each instance), with type, unit, default, choices, min/max/step/grid, and visibility/editability predicates. Instances store only overrides. |
| **Parameter callbacks** | Code that runs when a param changes. It recomputes dependents (W = nf × Wf), snaps to the process grid, and enforces min/max ([pdk-cb], [gpdk]). Virtuoso callbacks are SKILL using `cdfgData` ([cdf-cb-forum], [cdf-choices]). A well-known weakness: they fire only from the Add Instance / Edit Properties forms, not from scripted edits | V (SKILL). CC/SE: iPDK **Tcl** callbacks ([ipdk]) | RE: Python. XS: `tcleval()` at netlist time only | **Core** (sandboxed runtime) | Callbacks must run on *every* mutation path: UI, API, import, optimizer write-back. They must be deterministic and run in WASM. Prefer declarative rules (range, grid, derived formula) and use a sandboxed script only as a fallback. |
| **Parameter range / grid validation** | Min L/W, finger-width limits, nf/m rules, snapping to the manufacturing grid. Simulator-side soft limits are a separate layer: Spectre's `+param` range file ([spectre-param]) | V (PDK callbacks + Spectre). ADS | Nothing on the schematic side. NG errors at netlist time on model-bin misses | **Core** (as schema fields) | Same schema as above. Validation results feed the ERC (§3). The process grid comes from the process context (next rows). |
| **simInfo / per-simulator netlist info** | Per cell and simulator: netlist procedure, `instParameters`, `termOrder`, `namePrefix`, `componentName`, `otherParameters` ([cdf-ug]) | V. CC (iPDK) | XS `format=` per symbol plus `spectre_format` etc. RE: per-view netlist lines | **Core** | Netlist templates keyed by (cell, target dialect): SPICE, Spectre, CDL/LVS, Verilog. This is the xschem `format` string done properly (see complaints doc §8). |
| **Annotation label sets** | `paramLabelSet`, `opPointLabelSet` (e.g. `"id gm gds vdsat vds vth"`) and `modelLabelSet` decide what shows next to a device ([oplabel]) | V | XS: symbol text `@tokens`. analog-lens add-on ([analog-lens]) | **Hook** | The PDK cell declares label templates by category (params, op point, model). The annotation layer (§6) renders them. |
| **Process context: techfile, display.drf, layer map** | Techfile: layers, units, DBU, grids, vias, rules. `display.drf`: colours and stipples. Layer map: GDS layer/datatype numbers ([drf], [xic-compat]) | V, CC, SE (OA tech DB) | KL `.lyt`/`.lyp`. Open PDKs ship KLayout tech | **Hook** | A schematic tool needs only a small *process context* record: manufacturing grid, length units, model library files with corner sections, statistical sections. Layer and display data belong to a layout plugin. |
| **PCells** | Parameterised layout generators: SKILL PCells, PyCells, KLayout Python/Ruby PCells | V, CC/SE (PyCell) | KL, gdsfactory, RE (Python pcells) | **Out** of the schematic core | The cell's `layout` view is a reference to an external generator plus the param subset that is "layout-relevant". |
| **Device views per cell** | symbol, schematic, `spectre`, `auCdl`/`hspiceD` (netlist stubs), `veriloga`, `functional`/`verilog`, `layout`, `abstract`, `extracted`, `config`, `maestro`, `constraint` | V. SE: "multiple views per cell" ([pyxis]) | RE: symbol/schematic/veriloga/spice/config ([re]). XS: one .sym + one .sch per name | **Core** | Cell = name + map of typed views. A view can be graphical, a text file (netlist, VA, Verilog) or a reference (layout, extracted). |
| **iPDK / iCDF as an interchange source** | A vendor-neutral PDK built on OpenAccess, iCDF, Tcl callbacks and PyCells ([ipdk], [ipl]) | CC, SE, ADS, LayoutEditor | none (OA is C++) | **Plugin** (importer) | None beyond the param schema. iCDF maps almost 1:1 onto it, so it is a realistic import target for commercial-PDK users. |

## 2. Views, hierarchy and configuration

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Config view / Hierarchy Editor** | A view list (switch list), a stop list and a library list, plus overrides per cell and per instance path. The netlister walks the hierarchy through the config ([config-tut], [stop-pt], [switch-stop]) | V, CC, SE | RE has config views ([re]). XS: only the per-instance `schematic=` / `spice_sym_def=` attributes ([xs-inst-impl]) | **Core** | A first-class **binding object**, separate from the schematic, keyed by cell and by *occurrence path*, that the netlister resolves through. Never store this inside instances: one schematic must netlist differently under two configs. |
| **Switch schematic ↔ behavioural ↔ extracted per instance** | Swap one block's view for `veriloga`, `av_extracted` or a DSPF without touching the schematic | V, CC | RE (config). XS (manual attribute edit) | **Core** (same mechanism) | Covered by the binding object. Port mapping must be validated by pin name between views. |
| **Physical config (physConfig)** | Logical-to-physical hierarchy mapping for Layout XL | V | — | **Out** | — |
| **Cellview from cellview** | Generate a symbol from a schematic, a schematic from a symbol, or a symbol from a Verilog-A/Verilog module or a pin list | V, CC ("symbol generation" [cc-ds]), SE ([sedit]) | XS (make symbol from schematic). RE (symbol from schematic and from VA) | **Core** (sch→sym), **Plugin** (others) | Pin list with direction and signal type must be derivable from every view kind. |
| **Cross-view consistency check** | Symbol pins vs schematic pins vs netlist stub vs VA ports | V (Check), CC | XS: no general checker | **Core** | Same pin model across views. |

## 3. Schematic editing and connectivity

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Check & Save / configurable ERC** | Floating I/O, shorted outputs, unconnected instance terms, illegal names, overlapping instances, cross-view mismatches. Severity is configurable and noERC markers act as waivers ([erc-tut], [vse-ds]) | V, CC, SE | XS: limited netlist-time warnings. KiCad has an ERC matrix (PCB) | **Core** (engine + basic rules), **Plugin** (extra rule packs) | Pin **direction + signal type** (power/ground/analog/clock/…). Waiver objects attached to object IDs. Param-range violations come in as rule results. |
| **Bus / bundle / repeat syntax** | `a<0:7>`, `a<0:7:2>`, `<*4>a`, `<*2>(a,b)`, `x<0:31*4>`, bundles `a,b,c` ([busnames]) | V, CC, SE | XS: `a[3:0]`, `2*a`, `a[0:1:4:3]`, bundles ([xs-bus]) | **Core** | One canonical name grammar in core (parse, expand, width check). Importers map the Virtuoso and xschem dialects onto it. |
| **Iterated instances** | `I0<0:3>`: N instances from one symbol, with buses fanned out per bit | V, CC, SE (S-Edit "advanced array") | XS `x1[15:0]` ([xs-bus]) | **Core** | Instance name follows the same grammar. Occurrence paths must address `I0<2>`. |
| **Global nets** | `vdd!` style, global across the hierarchy | all | XS (global labels) | **Core** | Net scope flag. |
| **Inherited connections / netSet** | Net expressions `[@vdd:%:vdd!]` on a lower-level pin or wire, overridden by a `netSet` property on any ancestor instance ([inherit-blog], [inherit-forum]). CC adds a power-domain analyser for them ([cc-ds]) | V, CC | **none** | **Core** | Net-expression objects plus per-instance override properties, resolved hierarchically by the netlister. They also cover implicit bulk/substrate terminals (PDK 3-terminal symbols). |
| **Pin / terminal types** | input / output / inputOutput / switch / jumper, plus signal type (power, ground, clock, analog…) | V, CC | XS: `dir=` only | **Core** | See ERC row. |
| **Net / instance search, probe, hierarchical net tracer** | Find across the hierarchy and highlight a net through all levels | V, CC ("hierarchical net tracer" [cc-ds]) | XS: search and hierarchical highlight | **Core** | Occurrence-path addressing. |
| **Cross-window select / cross-probe** | Select in one view, highlight in schematic / layout / waveform / LVS report | V, CC, SE ([sedit]) | KL netlist browser (layout side only) | **Hook** | Selection bus keyed by object ID + occurrence path. Netlister **name map** (§6). |
| **On-canvas param edit, auto-wiring, align/distribute, flight-line routing** | Editing niceties | V, CC ("automatic wiring, on-canvas editing" [cc-ds]) | XS: partial (no in-canvas edit, see complaints doc §2) | **Core** | — |
| **Hierarchy navigation** | Descend edit/read, ascend, edit-in-place, Navigator tree | all | XS, RE | **Core** | — |
| **Notes / annotations / design-intent glyphs** | Free text plus typed intent markers on canvas (§7) | V, CC | XS: text only | **Core** (text), **Hook** (intent) | — |
| **Schematic diff** | Visual and semantic compare of nets, instances and properties | CC ("design comparison" [cc-ds]), Cliosoft VDD for V ([vdd]) | **XS `compare_schematics` / `--diff`** (geometric overlay) ([xs-dev]) | **Plugin** | **Stable object IDs** in the file make the diff semantic instead of geometric. |
| **Netlist → schematic import** | CDL/SPICE/Verilog In, auto-placed | V, CC, SE | XS: none built in (scripts exist) | **Plugin** | — |

## 4. Libraries, data management, collaboration

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Library Manager, lib/cell/view, `cds.lib`** | `DEFINE`/`INCLUDE`/`SOFTINCLUDE` library list. Copy/rename with reference fix-up. Categories ([xic-compat]) | V (OA), CC (OA), SE | RE (library browser + GitHub registry [re]). XS: search path only | **Core** | `lib:cell:view` namespace plus a declarative project manifest (already in complaints doc §7). |
| **DM integration: check-in/out, locking, history** | Cliosoft SOS, DesignSync, IC Manage, Perforce through Virtuoso's DM plug-in layer. Auto check-out on edit. Preemptive locks, because binary OA can't merge ([sos]) | V, CC, SE (Pyxis "concurrent design" [pyxis]) | git (text formats). XS and RE files are git-friendly | **Hook** | Lifecycle events (pre-open / pre-edit / pre-save / post-save) plus an advisory-lock API. A text, diffable format makes git the default and locking optional. |
| **Visual design diff in DM** | Graphical diff of two revisions inside the editor ([vdd]) | V + SOS | XS `--diff` | **Plugin** | Stable IDs (above). |
| **OpenAccess interop** | Read/write the OA DB | CC, SE, ADS (OA plug-in), V | none (OA is C++, licence-gated per member) | **Out** (maybe a separate converter later) | — |

## 5. Simulation environment (ADE L / Explorer / Assembler / Verifier)

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Test setup** | Simulator choice, model files plus **sections** (`.lib file tt`), analyses, save options, simulator options, stimuli, nodesets/ICs | V, CC (PrimeWave [pw-ds]), SE, ADS | XS: netlist text blocks on the schematic. QS: sim components on the schematic. RE: revedasim ([re]). CACE YAML ([cace]) | **Hook** | A **testbench-state view** (Virtuoso's `maestro` equivalent) stored as a versioned, diffable artifact next to the design. It references the design cell, config, vars, model sections, analyses, outputs, specs and corners. |
| **Design variables** | Named vars used in instance params, swept and optimised. Global vs per-test scope | V, CC, ADS | XS (`.param` in a code block). RE, QS | **Core** | A scoped variable table plus **one sandboxed expression evaluator** shared by params, callbacks and outputs. Vars carry optional `min/max/step` for sweeps and the optimiser. |
| **Outputs & calculator expressions** | Waveform expressions with hundreds of built-ins: `bandwidth`, `phaseMargin`, `cross`, `settlingTime`, `slewRate`… PrimeWave has over 100 functions plus HSPICE `.measure` ([pw-ds]) | V, CC, ADS | NG `.meas` / control language. XS graph expressions. CACE Python | **Plugin** | Outputs live in the testbench-state view. |
| **Specs & pass/fail** | `<`, `>`, range, min/max goals, colour-coded summary ([ade-asm-ds]) | V, CC | CACE (limits + markdown datasheet) | **Plugin** | — |
| **Corners manager** | PVT corners: model sections × temp × vars, grouped and reused across tests | V, CC, SE | CACE conditions. PO corners ([pyopus]) | **Plugin** | Model-section list in the process context. |
| **Parametric sweeps** | Nested multi-var sweeps, lists/ranges/log | V, CC, ADS, QS | NG (control loops). XY `.STEP`. QS | **Plugin** | Vars (above). |
| **Real-time tuning** | Slider-driven resimulate and overlay ([ade-exp], [ads-opt]) | V Explorer, ADS | **QS tuning mode** ([qucs-s]) | **Plugin** | Undoable var write-back. |
| **Monte Carlo** | Process + mismatch, sample count, seed, auto-stop, yield, histograms | V, CC, SE (Solido [solido]), ADS | NG (`agauss` + loops). CACE MC + mismatch ([cace]). PO | **Plugin** | Statistical section refs in the process context. Results store keyed by sample. |
| **High-yield estimation / worst-case & statistical corners** | 4-6σ estimation with scaled-sigma/importance/ML sampling. Spectre FMC. Statistical corner extraction. Mismatch contribution ([vvo], [weber], [vstudio-pr]) | V (Variation Option), SE (Solido) | PO: worst-case distance, yield ([pyopus]) | **Plugin** | — |
| **Optimisation** (already on the user's list) | Global and local, nominal / over corners / statistical, sizing of design vars against specs ([weber], [opt-forum]). ADS optimisation cockpit ([ads-opt]) | V (ADE GXL / Assembler), CC, ADS | **PO** (derivative-free, MPI, corner-based design). **ASCO** + QS ([asco], [qucs-asco]). BAG (generator scripts [bag]) | **Plugin** | Vars with bounds. **Transactional, undoable write-back** of optimum values into instance params. Results tied to a design snapshot. |
| **Sensitivity / DOE** | Which vars or devices drive an output. Statistical sensitivity ([ade-asm-ds]) | V, ADS (yield sensitivity histograms [ads-opt]) | NG `.sens` (DC/AC). PO screening | **Plugin** | — |
| **Run plans / regressions** | Conditional sequences of runs, batch or GUI ([runplan]). PrimeWave sequential testbenches feed measurements across tests ([pw-ds]) | V, CC | CACE (per-datasheet) | **Plugin** | — |
| **Job distribution** | ICRP worker processes dispatched to LSF/SGE with Max Jobs and linger time ([icrp]). PrimeWave grid jobs ([pw-ds]) | V, CC | CACE (local parallel jobs) | **Hook** | Core async job runner with status, cancel and events (complaints doc §9). Remote back-ends are plugins. |
| **Results DB, history, checkpoints** | Each run is a history entry. Setup can be restored from a history. Results are browsable ([histories]) | V, CC | none integrated (raw files) | **Hook** | Results record = {design snapshot hash, testbench-state hash, corner, sample, sweep point} → waveform files + scalars. The core provides the **snapshot hash** so stale results are detectable. |
| **Datasheets / reporting** | HTML datasheets and reports ([pw-ds]) | V, CC | **CACE markdown datasheets** | **Plugin** | — |
| **Spec traceability (ADE Verifier)** | Requirements → tests → pass/fail across blocks and teams ([verifier]) | V. CC (partially) | CACE (per block) | **Plugin** (late) | — |
| **Ocean / headless batch** | Script the whole ADE flow without a GUI ([ocean]). PrimeWave saves any setup as Tcl ([pw-ds]) | V, CC | CACE CLI. PO. XS Tcl batch | **Core** (API), **Plugin** (language) | Everything in the testbench-state view must be readable and writable through the headless API. |

## 6. Waveforms, measurement, back-annotation

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Waveform viewer** | Graph windows, results browser, digital/bus display ([viva]) | V (ViVA), CC (WaveView in PrimeWave), SE, ADS (Data Display) | XS embedded graphs ([xs-graphs]). RE revedaplot. QS diagrams. gaw, GTKWave | **Plugin** | Results data model (raw/PSF reader) is a plugin API (complaints doc). |
| **Calculator** | Expression builder over waveforms ([viva]) | V, CC, ADS | XS graph expressions. NG control language | **Plugin** | Reuse the core expression evaluator. |
| **Eye diagrams, jitter, BER; spectrum, histogram, scatter** | ([eye], [viva]). PrimeWave: statistical eye, PAM4, DDR ([pw-ds]) | V, CC, ADS | XS: none. Python notebooks | **Plugin** | — |
| **Direct plot / cross-probe schematic ↔ waveform** | Click a net or terminal to plot V or I. Click a trace to highlight the net | V, CC, SE ([sedit]) | XS: add selected nets to a graph | **Hook** | **Netlister name map**: occurrence path ↔ simulator node/device/terminal name, for every target dialect. |
| **DC operating-point back-annotation** | gm, gds, vdsat, vth, region (0-4) printed next to each device. Configurable label sets ([oplabel]) | V, CC, SE ([sedit]) | **XS** (`annotate_op` from a raw file, [xs-annot]). analog-lens adds gm/Id ([analog-lens]). QS: node DC bias | **Hook** | An **annotation overlay layer**: ephemeral, keyed by occurrence path, never written into the design file, rendered through PDK label templates (§1). |
| **Transient value at cursor annotated on schematic** | Moving a cursor in the graph updates the schematic labels | V | **XS** (graph cursor drives labels) | **Hook** | Same overlay. |
| **gm/Id design aids** | Lookup-table sizing | (third-party in V) | pygmid, analog-lens | **Plugin** | — |

## 7. Constraints, design intent, layout link

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Constraint Manager** | Typed constraints (matching, symmetry, alignment, orientation, modgen, net class, shielding, max current, voltage) on sets of instances, nets and terminals. Stored in a constraint view and synced schematic ↔ layout ([vse-course], [pa-flow]) | V (Schematic XL), CC (constraints + templates [cc-ds]) | ALIGN infers constraints from a netlist ([align]). Nothing in schematic editors | **Hook** | **Constraint objects** = {type, params, member set of object IDs / occurrence paths}. Persisted (in-file section or sidecar view), revalidated after edits (dangling member → ERC), exported. |
| **Circuit Prospector** | Finds diff pairs, current mirrors and cascodes and proposes constraint groups ([vse-course], [pa-flow]) | V | ALIGN auto-annotation (ML) | **Plugin** | Needs graph queries over the flattened netlist (core API). |
| **Design Intent (Virtuoso Studio)** | Lighter-weight intent markers on the schematic (matching, shielding, high-current nets, voltage drop, pin info) shown as glyphs and converted to layout constraints ([intent]) | V | none | **Hook** | Same constraint-object model. Rendering is core, semantics are a plugin. |
| **Net electrical properties (current, voltage) for EM / voltage-dependent DRC** | Sim-derived currents and voltages per net pushed to layout. CC annotates them onto the layout for EM and VDRC ([cc-ds]) | V (EAD [ead]), CC | none | **Plugin** | Export of results keyed by net occurrence path (name map + results store). |
| **EAD (electrically aware design)** | Live parasitic R/C/EM checks while drawing the layout ([ead]). CC Extraction Fusion ([cc-ds]) | V, CC | none | **Out** (layout tool) | — |
| **Schematic-driven layout (Layout XL, "XL compliance", ECO)** | Generate layout from source. Connectivity and param mismatch checks. Update components and nets ([xl-blog], [vls-ds]) | V, CC, SE (Pyxis SDL) | none mature (KL has no SDL). OF/BAG/ALIGN generate layout from scripts | **Out** for core. A bridge plugin to KLayout later | Stable instance IDs, plus each instance's layout-relevant params (nf, m, W/L) exported with the netlist. |
| **Modgens, templates, symbolic placement** | Layout-side reuse ([cc-ds]) | V, CC | ALIGN | **Out** | — |

## 8. Post-layout and physical-verification integration

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Extracted view / DSPF selection** | Config binds a block to `av_extracted`, Quantus Smart View or a DSPF include. Parasitics cross-probing ([quantus-sv]) | V, CC (StarRC) | Magic `ext2spice` / klayout-pex netlists included by hand in XS | **Hook** | The binding object (§2) must bind an occurrence to a **netlist-file view** with a port map, per corner if needed. |
| **LVS run + cross-probe** | Run Calibre / PVS / Pegasus from the editor. RVE highlights the error net in schematic and layout ([rve], [pegasus]) | V, CC, SE, ADS | KL LVS + netlist browser ([klayout-lvs]). netgen. XS LVS netlist mode (no cross-probe) | **Plugin** | Name map + selection/highlight API. A CDL/LVS netlist target in the netlister (simInfo `auCdl` equivalent). |
| **Interactive DRC in layout** | iPegasus, Calibre RealTime ([pegasus], [pyxis]) | V, SE | KL DRC | **Out** | — |

## 9. Mixed-signal

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Per-instance view selection: transistor / Verilog-A / Verilog-AMS / RNM / digital** | The config picks the abstraction per block ([ams-course]) | V (AMS Designer), CC, SE | RE (config). XS (`schematic=`, `verilog_sym_def`) | **Core** | Binding object (§2). Text views (VA, Verilog, SV) are first-class cell views. |
| **Connect modules / interface elements** | Auto-inserted A↔D converters at domain boundaries, rules per discipline. CC has parameterisable IEs ([ams-forum], [cc-ds]) | V, CC | NG XSPICE bridges inserted by hand. XS ngspice-verilog cosim examples | **Hook** | Pin and net **discipline/domain** attribute (electrical / logic / real). Boundary detection in the netlister is core; the insertion rules are a plugin. |
| **Real-number models (wreal / SV real)** | Analog behaviour in the digital solver ([rnm-wp]) | V, CC | — (Verilator/Icarus have no wreal) | **Plugin** | Domain attribute above. |
| **Digital + analog co-simulation** | One run, two engines | V, CC, SE | **NG `d_cosim`** with Verilator / Icarus / GHDL ([ng-news], [ng-fsic]). NG OSDI + OpenVAF for VA ([ng-osdi]) | **Plugin** | — |
| **Behavioural model generation & validation** | Schematic Model Generator builds VA/RNM from block schematics. amsDmv compares model vs schematic ([rnm-wp]) | V | none | **Plugin** (late) | — |
| **Mixed text + schematic views, Verilog editor** | Language-aware editor, cross-probe into text views ([cc-ds]) | CC, V | — | **Plugin** | Text views addressable by line for cross-probe. |

## 10. RF and high-frequency

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Periodic / HB analyses** | PSS, QPSS, PAC, PXF, PNoise, PSP, PSTB with shooting or harmonic-balance engines ([spectre-rf]). ADS HB / envelope | V (Spectre RF), CC, ADS | XY multi-tone HB. Qucsator HB ([qucs-rf]). NG experimental `.pss` | **Plugin** (analysis definitions in the sim plugin) | — |
| **S-parameters, Touchstone n-ports, Smith chart** | SP analysis, n-port elements from file, Smith / polar plots, matching synthesis ([ads-smith]) | ADS, V (ViVA), CC | **QS** (SP + Smith chart [qucs-s]). NG `.sp` | **Plugin** | An n-port symbol with a file-reference param (ordinary param schema). |
| **Phase noise, jitter, IMD, compression measurements** | Built-in RF measurement functions ([pw-ds]) | V, CC, ADS | — | **Plugin** | — |
| **EM co-simulation, load-pull, X-params** | Momentum / RFPro in ADS. EMX in V ([ads-py]) | ADS, V | openEMS → Touchstone → QS | **Out** | — |

## 11. Reliability and checks

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Device asserts / SOA checks** | Spectre `assert` + `checklimit` on op params, currents and voltages. ADE "Device Checking" GUI ([asserts]) | V, CC (Circuit Check) | NG: none built in (doable with `.meas`) | **Plugin** | Check rules can ship in the PDK pack next to the param schema. |
| **Aging (HCI/BTI), self-heating** | RelXpert / AgeMOS, Spectre native reliability ([rel-wp], [legato]). PrimeWave Aging ([pw-ds]) | V, CC, SE | none | **Out** (simulator feature). Plugin passes options | — |
| **Transistor-level EM/IR** | Voltus-XFi, PrimeSim EMIR ([spectre-plat], [pw-ds]) | V, CC | none | **Out** | — |
| **Analog fault/defect simulation** | Bridges, opens, parametric faults injected and simulated for test coverage ([fault], [legato]) | V (Legato), CC (Custom Fault), SE | none | **Plugin** (late) | Fault injection = netlist overlay per occurrence. Reuse the binding/overlay mechanism rather than editing the schematic. |
| **FastSPICE / large-design simulators** | Spectre X / FX / APS ([spectre-fx], [spectre-plat]) | V, CC (XA), SE (AFS) | NG/XY (XY parallel) | **Out** (simulator). Sim driver plugin | — |

## 12. Scripting and extensibility

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Full-DB scripting language** | SKILL/SKILL++ in V. Tcl/Python/C++ in CC ([cc-ds]). AEL + Python in ADS ([ads-py]). Tcl in S-Edit ([sedit]) | all | XS Tcl. RE Python REPL ([re]) | **Core** API, **Plugin** language bindings | One command/query API (complaints doc #7). WASM-safe plugin ABI. |
| **Triggers / event hooks** | `deRegUserTriggers` for menus and post-open actions in schematic and layout ([skill-trig]) | V, CC | RE (plugin menus via config) | **Core** | Event bus: open, save, param-change, selection, netlist, sim-done. |
| **Custom menus, forms, bindkeys** | `hiSetBindKey`, `hiCreateAppForm` ([bindkey]). PrimeWave org-wide bindkeys and custom measurements ([pw-ds]) | V, CC | XS Tcl menus | **Core** (remappable keymap), **Plugin** (menus) | — |
| **IPC with external processes** | `ipcBeginProcess` etc. Virtuoso ↔ Python bridges, including LLM agents ([vbridge]) | V | — | **Core** (headless API over stdio/socket) | — |
| **Pre/post-run procedures** | Calibration hooks around simulation ([pw-ds]) | CC, V | CACE | **Plugin** | — |

## 13. AI/ML, generators, migration

| Feature | What it is | Who has it | Open-source today | Verdict | Core DM impact |
|---|---|---|---|---|---|
| **Generative node migration** | Map schematic and layout to a new node, then re-centre with ADE ([vstudio-pr]) | V (Virtuoso Studio), CC | none | **Plugin** | A **replace-master-with-param-map** core command: swap a cell keeping pin-name connectivity and translate params with a mapping table. This is also the basis for manual migration. |
| **ML-based Monte Carlo / high-sigma** | Spectre FMC ([vstudio-pr]). Solido additive learning ([solido]) | V, SE | — | **Plugin** | — |
| **AI design-space optimisation / sizing** | Design Space Optimization ([vstudio-pr]). RL sizing | V, SE | PO, BAG, OF ([openfasoc]), AutoCkt-style research | **Plugin** | Same as optimiser. |
| **NL search, doc assistant, waveform auto-clustering** | IC25.1 ([ic251]) | V | — | **Plugin** | A headless API plus a text, diffable format make LLM agents straightforward. |
| **Generators (BAG/OF)** | Scripts produce sized schematics and layouts from specs ([bag], [openfasoc]) | (BAG on top of V) | BAG, OF, ALIGN | **Plugin** | Programmatic schematic creation API with auto-placement. |

---

## Core data-model requirements (what the plugins force on core)

Ranked by how expensive they are to retrofit:

1. **`lib:cell:view` with typed multi-views per cell.** Kinds: graphical (symbol, schematic), text (netlist stub per dialect, Verilog-A, Verilog/SV), reference (layout, extracted/DSPF), and artifact (testbench-state, config, constraints). Drives §1, §2, §5, §8, §9.
2. **A binding/config object separate from schematics**: switch list, stop list, per-cell and per-occurrence overrides, port-map validation, and binding an occurrence to a netlist-file view. The netlister resolves only through it. Drives §2, §8, §9, §11 (fault overlays).
3. **Per-cell parameter schema (CDF-equivalent):** types, units, defaults, choices, min/max/step/grid, visibility/editability predicates, a "netlisted for target X" flag, a "layout-relevant" flag, and label templates (params / op point / model). Instances store only overrides.
4. **Deterministic, sandboxed parameter callbacks/rules** that run on every mutation path (UI, API, import, optimiser), not only in dialogs. Declarative first, script second, WASM-safe.
5. **Process context from the PDK**: manufacturing grid, units, model files with corner and statistical sections, per-dialect netlist templates. Layers and display stay in a layout plugin.
6. **Scoped design variables + one shared sandboxed expression language** used by params, callbacks, outputs and specs. Vars carry optional bounds for sweep and optimisation.
7. **Stable object identity**: a persisted ID for every instance, net, pin and wire, plus first-class **occurrence paths** (hierarchical, including `I0<2>` iterated bits). Constraints, annotations, diff, results, cross-probe, LVS and ECO all key on these.
8. **Netlister name map**: for every target, occurrence path ↔ simulator/LVS node, device and terminal names. Without it there is no cross-probing, op annotation, LVS debug or EM export.
9. **Connectivity semantics in core**: one bus/bundle/repeat grammar, iterated instances, global nets, inherited connections (net expressions + netSet overrides), pin direction + signal type, and pin/net discipline (electrical/logic/real).
10. **Constraint/intent objects**: {type, params, member set of IDs/paths}, persisted, revalidated after edits, exportable. Rendering is core, semantics are plugins.
11. **Annotation overlay layer**: ephemeral, keyed by occurrence path, rendered through PDK label templates, never saved into the design. Used for op points, cursor values, LVS/ERC/assert violations and constraint status.
12. **Testbench-state view** (maestro-like), a versioned, diffable design artifact holding: design + config ref, vars, model sections, analyses, outputs, specs, corners, MC settings and run plans.
13. **Results-store contract**: every result is tagged with the design snapshot hash, testbench hash, corner, sample and sweep point, so results are reproducible and staleness can be detected. Core provides the hash; plugins own storage.
14. **Transactional, undoable edit API + event bus + lock hooks**: optimiser write-back, migration (replace master with a param map), DM check-out, triggers, and a headless API usable by the CLI, plugins and LLM agents.

## Out of scope for Schemify (integrate, don't build)

Layout editing, PCells, EAD, SDL, modgens, interactive DRC, extraction engines, aging/EMIR solvers, FastSPICE, EM solvers, OpenAccess read/write. Schemify should be a clean **connectivity + intent + testbench source** for these tools (KLayout, Magic, netgen, openEMS, simulators), not a replacement for them.

---

## Sources (link references; open the raw file for the URL list)

[cdf-ug]: https://amarketplaceofideas.com/wp-content/uploads/2015/08/cdfuser.pdf
[cdf-cb-forum]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-skill/37073/pcell-callbacks-and-cdfgdata-variable
[cdf-choices]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-skill/58003/changeing-choices-and-defaultvalue-of-cdfparameter-through-callback
[pdk-cb]: https://skycadeda.com/pcell-development-services/
[gpdk]: https://research.ece.cmu.edu/sandbox/cadfaq/docs/gpdk_referenceManual.pdf
[ipdk]: http://www.layouteditor.net/wiki/iPDK
[ipl]: https://www.edn.com/ipl-group-releases-pdk-standard/
[spectre-param]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/spectre-parameters-check
[oplabel]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-design/47275/adding-parameters-for-annotating-additional-dc-operating-points
[drf]: http://wrcad.com/manual/xicmanual/node140.html
[xic-compat]: http://www.srware.com/manual/xicmanual/node138.html
[config-tut]: https://www.sonnetsoftware.com/support/help-18/cadence/ConfigView_cad_tut.html
[stop-pt]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-design/28398/hierarchy-editor---implication-of-stop-point-on-netlist
[switch-stop]: https://www.thecadforums.com/threads/netlisting-question-why-are-stopviews-also-in-the-switchviewlist.33649/
[xs-inst-impl]: https://xschem.sourceforge.io/stefan/xschem_man/tutorial_instance_based_implementation.html
[xs-bus]: http://repo.hu/projects/xschem/xschem_man/tutorial_busses.html
[xs-dev]: https://xschem.sourceforge.io/stefan/xschem_man/developer_info.html
[xs-annot]: http://repo.hu/projects/xschem/xschem_man/tutorial_ngspice_backannotation.html
[xs-graphs]: http://repo.hu/projects/xschem/xschem_man/graphs.html
[analog-lens]: https://github.com/jonahsaunders/analog-lens
[re]: https://github.com/eskiyerli/revolution-eda
[re-docs]: https://reveda.eu/documentation/
[inherit-blog]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/manage-inherited-connections-with-cpf-in-virtuoso
[inherit-forum]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-design/38850/inherited-connections---connecting-to-nets
[busnames]: https://www.eecs.umich.edu/courses/eecs427/f10/Common/busnames.pdf
[erc-tut]: https://web.itu.edu.tr/~ateserd/CADENCE%20Manual.pdf
[vse-ds]: https://www.cadence.com/content/dam/cadence-www/global/en_US/documents/tools/custom-ic-analog-rf-design/virtuoso-vse-fam-ds.pdf
[vse-course]: https://www.cadence.com/en_US/home/training/all-courses/86420.html
[cc-ds]: https://www.synopsys.com/content/dam/synopsys/implementation&signoff/datasheets/custom-compiler-ds.pdf
[pw-ds]: https://www.synopsys.com/content/dam/synopsys/implementation&signoff/datasheets/primewave-ds.pdf
[sedit]: https://www.eda-solutions.com/app/uploads/2021/02/Siemens-SW-Tanner-S-Edit-Schematic-Capture-FS-81570-C1-fs.pdf
[pyxis]: https://eda.sw.siemens.com/en-US/ic/ic-custom/ams/
[solido]: https://www.siemens.com/en-us/products/ic/solido/design-environment/
[vdd]: https://www.cliosoft.com/2020/05/19/visual-design-diff/
[sos]: https://www.keysight.com/us/en/products/design-engineering-software/engineering-data-management/sos-for-cadence-virtuoso-studio.html
[ade-asm-ds]: https://www.cadence.com/en_US/home/resources/datasheets/virtuoso-ade-assembler-ds.html
[ade-exp]: https://www.cadence.com/en_US/home/tools/custom-ic-analog-rf-design/circuit-design/virtuoso-ade-explorer.html
[vvo]: https://www.cadence.com/en_US/home/resources/datasheets/virtuoso-variation-option-ds.html
[weber]: https://www.researchgate.net/publication/320559729_Variation-Aware_Design_with_Virtuoso_ADE
[opt-forum]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-design/64993/error-doing-in-local-optimization-in-ade-assembler
[histories]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/virtuosity-exploring-histories
[runplan]: https://www.cadence.com/en_US/home/training/all-courses/86256.html
[icrp]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/things-you-didn-t-know-about-virtuoso-ade-xl-take-this-job-and-run-it
[verifier]: https://www.cadence.com/en_US/home/resources/datasheets/virtuoso-ade-verifier-ds.html
[ocean]: https://wikis.ece.iastate.edu/vlsi/index.php?title=Simulations_using_Cadence_OCEAN_Scripts
[viva]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/knowledge-booster-training-bytes---virtuoso-visualization-and-analysis-xl
[eye]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/virtuosity-eye-diagram-measurements
[ead]: https://community.cadence.com/cadence_blogs_8/b/ii/posts/virtuoso-electrically-aware-design-ead-a-new-approach-to-custom-analog-layout
[intent]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/virtuoso-studio-collaborate-with-the-right-intentions
[pa-flow]: https://semiwiki.com/eda/cadence/1519-parasitic-aware-design-flow-with-virtuoso/
[vls-ds]: https://www.cadence.com/en_US/home/resources/datasheets/virtuoso-layout-suite-ds.html
[xl-blog]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/virtuoso-studio-excellent-xl---next-gen-layouts-with-virtuoso-layout-xl
[align]: https://github.com/ALIGN-analoglayout/ALIGN-public
[quantus-sv]: https://www.cadence.com/en_US/home/resources/technical-briefs/quantus-smart-view-tb.html
[rve]: https://www.siemens.com/en-us/products/ic/calibre-design/interfaces/rve/
[pegasus]: https://www.cadence.com/en_US/home/tools/digital-design-and-signoff/silicon-signoff/virtuoso-indesign-drc.html
[klayout-lvs]: https://www.klayout.de/forum/discussion/1873/how-to-display-the-results-of-comparator-in-the-netlist-browser-and-generate-a-customized-netlist
[ams-course]: https://www.cadence.com/en_US/home/training/all-courses/85059.html
[ams-forum]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-design/36627/ams-simulator-in-cadence-virtuoso
[rnm-wp]: https://www.cadence.com/content/dam/cadence-www/global/en_US/documents/solutions/mixed-signal-verification-wp.pdf
[ng-osdi]: https://ngspice.sourceforge.io/osdi.html
[ng-news]: https://ngspice.sourceforge.io/news.html
[ng-fsic]: https://wiki.f-si.org/images/1/1e/FSiC2025_ngspice.pdf
[qucs-s]: https://ra3xdh.github.io/
[qucs-rf]: https://qucs-s-help.readthedocs.io/en/legacy/RF.html
[qucs-asco]: https://github.com/ra3xdh/qucs_s/blob/current/NEWS.md
[asco]: https://asco.sourceforge.net/
[ads-opt]: https://edadownload.software.keysight.com/eedl/ads/2011_01/pdf/optstat.pdf
[ads-smith]: https://edadownload.software.keysight.com/eedl/ads/2011_01/pdf/smithchartutil.pdf
[ads-py]: https://www.keysight.com/us/en/lib/resources/miscellaneous/eda-software/python-eda.html
[cace]: https://github.com/fossi-foundation/cace
[pyopus]: http://spiceopus.si/pyopus/
[bag]: https://www.researchgate.net/publication/325107048_BAG2_A_Process-Portable_Framework_for_Generator-Based_AMS_Circuit_Design
[openfasoc]: https://github.com/idea-fasoc/OpenFASOC
[asserts]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/spectre-tech-tips-spectre-assert-and-design-check-overview
[legato]: https://www.cadence.com/en_US/home/resources/datasheets/legato-reliability-solution-ds.html
[fault]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/spectre-tech-tips-introducing-spectre-analog-fault-analysis
[rel-wp]: https://www.cadence.com/en_US/home/resources/white-papers/analog-reliability-analysis-for-mission-critical-applications-wp.html
[spectre-plat]: https://www.cadence.com/en_US/home/resources/datasheets/spectre-simulation-platform-ds.html
[spectre-fx]: https://www.cadence.com/en_US/home/resources/datasheets/spectre-fx-ds.html
[spectre-rf]: https://www.cadence.com/en_US/home/resources/datasheets/spectre-rf-option-ds.html
[skill-trig]: https://community.cadence.com/cadence_technology_forums/f/custom-ic-skill/45528/user-triggers-for-layout-l-and-layout-xl-using-deregusertriggers-command
[bindkey]: https://analoghub.ie/category/skill/article/skillBindkeysSetup
[vbridge]: https://github.com/Arcadia-1/virtuoso-bridge-lite
[vstudio-pr]: https://www.cadence.com/en_US/home/company/newsroom/press-releases/pr/2023/cadence-unleashes-the-future-of-analog-custom-and-rfic-design.html
[ic251]: https://community.cadence.com/cadence_blogs_8/b/cic/posts/virtuoso-studio-ic25-1-explore-the-new-features-one-byte-at-a-time

<!-- Every URL above was returned by a web search or fetched during this research (2026-10-10). -->
