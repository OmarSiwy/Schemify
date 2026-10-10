# XSchem 3.4.7 functional inventory

Reference for Schemify, the Rust reimplementation of XSchem. The goal is feature-for-feature parity.

- **Source tree:** `/nix/store/rhvgzl9mjl58nr0qm6l98fxbiadg3q98-source`. In this document `$S` stands for this path, and citations such as `xschem.tcl:1234` refer to `$S/src/...` unless a different path is given.
- **Installed tree:** `/nix/store/pxbims2rh0lgm0vamwxskixn0cxvklys-xschem-3.4.7` (`share/xschem`, `share/doc/xschem`).
- **Version:** `XSCHEM_VERSION "3.4.7"` (`xschem.h:26`).
- **Method:** the inventory was extracted mechanically from the C and Tcl sources and then checked against `doc/xschem_man/*.html`. Where `keys.help` or the HTML docs disagree with the code, the code wins.

## Contents
1. Menus, toolbar, context menu, tabs
2. Keybindings and mouse interactions
3. Dialogs
4. Core features
   - 4A: objects, hierarchy, netlisting, net naming, ERC, symbol generation, LVS
   - 4B: simulation, waveforms, back-annotation, export, undo, clipboard, windows, CLI
   - 4C: scripting API (`xschem` subcommands) and xschemrc configuration
5. File format (.sch / .sym)
6. Counts

## 1. Menus, toolbar, context menu, tabs

All menu code lives in `proc build_widgets` (src/xschem.tcl:8772-9457). Every top-level window (main `.` and, in non-tabbed mode, each extra toplevel `.xN`) gets its own full menubar (`$topwin.menubar`), status bar and toolbar (xinit.c:1646-1649). All menus are `-tearoff 0`. Accelerator strings are display-only labels; the real key handling is in callback.c (see section 2). Checkbuttons/radiobuttons are bound to Tcl global variables (column "Var"); most are read by C via `tclgetboolvar()`.

Legend — Type: `cmd` command, `chk` checkbutton, `radio` radiobutton, `casc` cascade, `sep` separator.

### 1.0 Menubar layout (xschem.tcl:8787-8811, 9044-9075)

Order, left to right:

| # | Entry | Type | Notes |
|---|---|---|---|
| 1 | File | casc | `.menubar.file` |
| 2 | Edit | casc | `.menubar.edit` |
| 3 | Options | casc | `.menubar.option` |
| 4 | View | casc | `.menubar.view` |
| 5 | Properties | casc | `.menubar.prop` |
| 6 | Layers | casc | `.menubar.layers`; menubar label background = current layer color (reconfigure_layers_button, xschem.tcl:7413-7422) |
| 7 | Tools | casc | `.menubar.tools` |
| 8 | Symbol | casc | `.menubar.sym` |
| 9 | Highlight | casc | `.menubar.hilight` |
| 10 | Simulation | casc | `.menubar.simulation` |
| 11 | Help | casc | `.menubar.help` |
| 12 | ` - ` | cmd (disabled) | visual spacer (9044) |
| 13 | Netlist | cmd button | `xschem netlist -erc`; bg #888888, active orange (9045) |
| 14 | Simulate | cmd button | `simulate_from_button` (9048) |
| 15 | Waves | casc | `.menubar.waves` (9053) |

Netlist / Simulate / Waves menubar entries act as status lamps (state kept per window in `tctx::<win_path>_{netlist,simulate,waves}`, restored on tab switch by `set_sim_netlist_waves_buttons`, xschem.tcl:8476-8503):

| Button | Color | Meaning | Source |
|---|---|---|---|
| Netlist | Green / red | last netlist OK / had errors | scheduler.c:3488-3494, callback.c:3164-3170 |
| Netlist | default | schematic modified since netlist | actions.c:167 |
| Simulate | orange | simulation running; clicking again kills it (`kill_running_cmds $id -15`) | xschem.tcl:2596-2614, 2681-2685 |
| Simulate | Green / red | sim process exit code 0 / non-zero | `set_simulate_button`, xschem.tcl:8441-8474 |
| Waves | Green | raw file loaded | save.c:1063, actions.c:181 |
| Waves | orange | re-netlisted while waves loaded (waves stale) | xschem.tcl:978-984 |

### 1.1 File (xschem.tcl:8829-8891)

| Item | Type | Accel | Action | Description |
|---|---|---|---|---|
| Clear Schematic | cmd | Ctrl+N | `xschem clear schematic` | New empty schematic in current window (asks to save if modified) |
| Clear Symbol | cmd | Ctrl+Shift+N | `xschem clear symbol` | New empty symbol in current window |
| Component browser | cmd | Shift-Ins, Ctrl-I | `insert_symbol ...` if `$new_symbol_browser` else `load_file_dialog {Insert symbol} *.sym INITIALINSTDIR 2` | Open symbol placement browser (see section 3) |
| Open | cmd | Ctrl+O | `xschem load` | File dialog, load .sch/.sym |
| Open Most Recent | cmd | Ctrl+Shift+O | `xschem load -gui [lindex $recentfile 0]` | Reopen last file |
| Open recent | casc | | `.file.recent` | Dynamic list, see 1.1a |
| Create new window/tab | cmd | | `xschem new_schematic create` | Empty new tab (or toplevel window if non-tabbed) |
| Open schematic in new window/tab | cmd | Alt+E | `open_sub_schematic` | Selected instance's schematic (or current sch) in new tab; carries loaded raw file and descends (xschem.tcl:3959-3995) |
| Open symbol in new window/tab | cmd | Alt+I | `xschem symbol_in_new_window` | Selected instance's .sym in new tab |
| Delete files | cmd | Shift-D | `xschem delete_files` | File selector to delete files from disk |
| Save | cmd | Ctrl+S | `xschem save` | Save current file |
| Merge | cmd | B | `xschem merge` | Import another .sch/.sym into current, placed as a move |
| Reload | cmd | Alt+S | confirm, then `xschem reload` | Discard edits, reload from disk |
| Save as | cmd | Ctrl+Shift+S | `xschem saveas` | Save under new name (.sch) |
| Save as symbol | cmd | Ctrl+Alt+S | `xschem saveas {} symbol` | Save under new name with .sym extension |
| Image export | casc | | `.file.im_exp` | See 1.1b |
| — | sep | | | |
| Start new Xschem process | cmd | X | `xschem new_process` | Spawn separate xschem process |
| Close schematic | cmd | Ctrl+W | `xschem exit` | Close current tab/window (last one: clears) |
| Quit Xschem | cmd | Ctrl+Q | `quit_xschem` | Close all tabs/windows (`new_schematic destroy_all`) then exit (xschem.tcl:8225-) |

1.1a Open recent (dynamic, xschem.tcl:1231-1291): up to 10 entries, label = `file tail`, action `xschem load -gui {path}`. List persisted to `$USER_CONF_DIR/recent_files` as `set recentfile {...}` (also stores the recent-components list of the symbol browser, `c_toolbar`, xschem.tcl:1256-1276).

1.1b Image export (xschem.tcl:8872-8879)

| Item | Type | Accel | Action | Description |
|---|---|---|---|---|
| EPS Selection Export | cmd | | `xschem print eps` | Encapsulated PS of selection |
| PDF/PS Export | cmd | * | `xschem print pdf` | PDF/PS of current view/selection |
| PDF/PS Export Full | cmd | | `xschem print pdf_full` | PDF/PS of whole drawing |
| Hierarchical PDF/PS Export | cmd | | `xschem hier_psprint` | Multipage PDF of whole hierarchy |
| PNG Export | cmd | Ctrl+* | `xschem print png` | Bitmap export |
| SVG Export | cmd | Alt+* | `xschem print svg` | SVG export |

### 1.2 Edit (xschem.tcl:9009-9042)

| Item | Type | Accel | Action | Description |
|---|---|---|---|---|
| Undo | cmd | U | `xschem undo; xschem redraw` | Undo |
| Redo | cmd | Shift+U | `xschem redo; xschem redraw` | Redo |
| Copy | cmd | Ctrl+C | `xschem copy` | Copy selection to clipboard file (`$USER_CONF_DIR/.clipboard.sch`) |
| Cut | cmd | Ctrl+X | `xschem cut` | Copy to clipboard + delete |
| Paste | cmd | Ctrl+V | `xschem paste` | Paste clipboard, starts move |
| Delete | cmd | Del | `xschem delete` | Delete selection |
| Select all | cmd | Ctrl+A | `xschem select_all` | Select everything |
| Duplicate objects | cmd | C | `xschem copy_objects` | Interactive copy (in-schematic duplicate) |
| Move objects | cmd | M | `xschem move_objects` | Interactive move |
| Move objects stretching attached wires | cmd | Control+M | `xschem move_objects stretch` | Move, wire endpoints on pins follow |
| Move objects adding wires to connected pins | cmd | Shift+M | `xschem move_objects kissing` | Move, insert wires at touching pins |
| Horizontal Flip in place selected objects | cmd | Alt-F | `xschem flip_in_place` | Mirror each object about its own anchor |
| Vertical Flip in place selected objects | cmd | Alt-V | `xschem flipv_in_place` | Vertical mirror, in place |
| Rotate in place selected objects | cmd | Alt-R | `xschem rotate_in_place` | Rotate 90° each object around own anchor |
| Vertical Flip selected objects | cmd | Shift-V | `xschem flipv` | Mirror selection as a group |
| Horizontal Flip selected objects | cmd | Shift-F | `xschem flip` | Mirror selection as a group |
| Rotate selected objects | cmd | Shift-R | `xschem rotate` | Rotate selection as group |
| Unconstrained move | radio (`constr_mv`=0) | | `xschem set constr_mv 0` | Free move |
| Constrained Horizontal move | radio (`constr_mv`=1) | H | `xschem set constr_mv 1` | Lock moves/wires to horizontal |
| Constrained Vertical move | radio (`constr_mv`=2) | V | `xschem set constr_mv 2` | Lock to vertical |
| Push schematic | cmd | E | `xschem descend` | Descend into selected instance's schematic |
| Push symbol | cmd | I | `xschem descend_symbol` | Descend into selected instance's symbol |
| Pop | cmd | Ctrl+E | `xschem go_back` | Return to parent level |

### 1.3 Options (xschem.tcl:8892-9006)

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Color Postscript/SVG | chk `color_ps` | | `xschem set color_ps` | Color vs B/W PS/PDF/SVG export |
| Transparent SVG background | chk `transparent_svg` | | | No background rect in SVG |
| Debug mode | chk `menu_debug_var` | | `xschem debug 0/1` | Debug prints |
| Undo buffer on Disk | chk `undo_type` (disk/memory) | | `switch_undo` | Toggle disk vs memory undo; confirms, resets undo (xschem.tcl:8698-8720) |
| Enable stretch | chk `enable_stretch` | Y | | Wires attached to moved objects stretch by default |
| Enable infix-interface | chk `infix_interface` | | | Select-then-command vs command-then-click workflow for wire/line etc. |
| Enable orthogonal wiring | chk `orthogonal_wiring` | Shift-L | | Manhattan routing with auto L-bend |
| Unsel. partial sel. wires after stretch move | chk `unselect_partial_sel_wires` | | | Deselect stretched wires after move |
| Auto Join/Trim Wires | chk `autotrim_wires` | | runs `xschem trim_wires` on enable | Auto-merge/trim wires after edits |
| Persistent wire/line place command | chk `persistent_command` | | | Stay in wire/line mode after each segment |
| Intuitive Click & Drag interface | chk `intuitive_interface` | | `xschem set intuitive_interface` | Click-drag moves, click selects (vs legacy) |
| Crosshair | casc | | `.option.crosshair` | See below |
| Replace [ and ] for buses in SPICE netlist | cmd | | `input_line ... "set bus_replacement_char"` | Two chars replacing `[]` in SPICE net names |
| Group bus slices in Verilog instances | chk `verilog_bitblast` | | | Verilog bus port grouping |
| Draw grid | chk `draw_grid` | % | redraw | Show grid |
| Half Snap Threshold | cmd | G | `xschem set cadsnap [expr $cadsnap/2]` | Halve snap |
| Double Snap Threshold | cmd | Shift-G | `xschem set cadsnap [expr $cadsnap*2]` | Double snap |
| Variable grid point size | chk `big_grid_points` | | redraw | Grid dot size scales with zoom |
| — | sep | | | |
| No XCopyArea drawing model | chk `draw_window` | Ctrl+$ | `xschem set draw_window` | Draw directly to window, no backing pixmap |
| Fix for GPUs with broken tiled fill | chk `fix_broken_tiled_fill` | | `xschem set ...; resetwin; redraw` | Workaround for stipple fill bugs |
| Fix broken RDP mouse coordinates | chk `fix_mouse_coord` | | `xschem set fix_mouse_coord` | Use pointer query instead of event coords (getmousex, xschem.tcl:8507-) |
| — | sep | | | |
| Netlist format / Symbol mode | casc | | `.option.netlist` | See below |

Crosshair submenu (8927-8938):

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Draw snap cursor | chk `snap_cursor` | Alt-Z | | Show snapped-point cursor |
| Draw crosshair | chk `draw_crosshair` | Alt-X | | Full crosshair at mouse |
| Crosshair size | cmd | | `input_line ... "set crosshair_size"` | Int, 0 = full width |

Netlist format / Symbol mode submenu (8983-9008) — radio group `netlist_type`:

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Flat netlist | chk `flat_netlist` | : | `xschem set flat_netlist` | Flatten hierarchy in netlist |
| Split netlist | chk `split_files` | | | One file per subcircuit/module |
| Spice netlist | radio `netlist_type=spice` | Ctrl+Shift+V | `xschem set netlist_type spice; redraw` | SPICE mode |
| VHDL netlist | radio `=vhdl` | Ctrl+Shift+V | same | VHDL mode |
| Verilog netlist | radio `=verilog` | Ctrl+Shift+V | same | Verilog mode |
| tEDAx netlist | radio `=tedax` | Ctrl+Shift+V | same | tEDAx mode |
| Symbol global attrs | radio `=symbol` | Ctrl+Shift+V | same | Edit file as symbol: global attrs are symbol attrs (K block) |

Ctrl+Shift+V cycles through the 5 values.

### 1.4 View (xschem.tcl:9081-9153)

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Redraw | cmd | Esc | `xschem redraw` | Redraw (Esc also aborts) |
| Fullscreen | cmd | `\` | `xschem fullscreen $topwin.drw` | Cycle: fullscreen w/ menus → drawing-only → normal (xinit.c:1125-1165) |
| Zoom Full | cmd | F | `xschem zoom_full` | Fit all |
| Zoom In | cmd | Shift+Z | `xschem zoom_in` | |
| Zoom Out | cmd | Ctrl+Z | `xschem zoom_out` | |
| Zoom box | cmd | Z | `xschem zoom_box` | Drag rectangle to zoom |
| Set snap value | cmd | | `input_line ... "xschem set cadsnap"` | Float snap |
| Set grid spacing | cmd | | `input_line ... "xschem set cadgrid"` | Float grid |
| Toggle colorscheme | cmd | Shift+O | `xschem toggle_colorscheme` | Dark/light |
| Dim colors | cmd | | `color_dim` | Dialog: slider to dim all layers, optional dim background (xschem.tcl:5822) |
| Change current layer color | cmd | | `change_color` | Color picker for current layer, saved to `$USER_CONF_DIR/colors` (6238) |
| Reset all colors to default | cmd | | `reset_colors 1` | Delete colors config, restore defaults (6211) |
| Toggle variable line width | chk `change_lw` | _ | | Line width scales with zoom |
| Set line width | cmd | Alt+- | `input_line ... "xschem line_width"` | Fixed line width |
| Set grid point size | cmd | | `input_line ... "set grid_point_size"` | Int, -1 = auto |
| Tabbed interface | chk `tabbed_interface` | | `setup_tabbed_interface` | Tabs vs separate windows; disabled while >1 tab/window open (xinit.c:1631,1734,1848) |
| Show / Hide | casc | | `.view.show` | See below |

Show / Hide submenu (9126-9154):

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Show ERC Info window | chk `show_infowindow` | | `wm deiconify/withdraw .infotext` | Netlist/ERC message window |
| Visible layers | cmd | | `select_layers; xschem redraw` | Dialog of per-layer visibility checkboxes (5753) |
| Symbol text | chk `sym_txt` | Ctrl+B | `xschem set sym_txt` | Show/hide texts inside symbol instances |
| Show Toolbar | chk `toolbar_visible` | | `toolbar_show/hide` | |
| Horizontal Toolbar | chk `toolbar_horiz` | | hide+show | Toolbar top (horizontal) vs left (vertical) |
| Show hidden texts | chk `show_hidden_texts` | | `xschem update_all_sym_bboxes; redraw` | Show texts with `hide=true` |
| Draw grid axes | chk `draw_grid_axes` | | redraw | Draw x=0/y=0 axes |

### 1.5 Properties (xschem.tcl:9155-9165)

| Item | Type | Accel | Action | Description |
|---|---|---|---|---|
| Edit | cmd | Q | `xschem edit_prop` | Attribute editor dialog for selection, or global attrs if none selected |
| Edit with editor | cmd | Shift+Q | `xschem edit_vi_prop` | Edit attributes in external `$editor` |
| View | cmd | Ctrl+Shift+Q | `xschem view_prop` | Read-only attribute view |
| Toggle *_ignore attribute on selected instances | cmd | Shift+T | `xschem toggle_ignore` | Toggle `spice_ignore`/`verilog_ignore`/`vhdl_ignore`/`tedax_ignore` per current netlist mode |
| Change selected object insertion order | cmd | Shift+S | `xschem change_elem_order -1` | Prompt for new index in object array (affects netlist order) |
| Edit Header/License text | cmd | Shift+B | `update_schematic_header` | Edit file header text (`xschem get/set header_text`) (1404) |
| Edit file (danger!) | cmd (red bg) | Alt+Q | `xschem edit_file` | Raw-edit current .sch (or selected inst's .sym) text |

### 1.6 Layers (dynamic, create_layers_menu, xschem.tcl:9554-9590)

One `cmd` entry per layer, `cadlayers` entries (default 22, xschemrc:400). Label `NN` plus tag for special layers: 0 `-BG`, 1 `-WIRE`, 2 `-GRID` (also selection layer), 3 `-TEXT`, 5 `-PIN` (xschem.h:148-156). Entry background = layer color; column break every 10 entries. Action: `xschem set rectcolor N; reconfigure_layers_button` = set current drawing layer for lines/rects/polygons/arcs; recolors "Layers" menubar label. Recolored on colorscheme change (`reconfigure_layers_menu`, 7424-7446).

### 1.7 Tools (xschem.tcl:9229-9258)

| Item | Type | Accel | Action | Description |
|---|---|---|---|---|
| Insert symbol | cmd | Ins, Shift-I | `xschem place_symbol` | Symbol file dialog then place |
| Insert text | cmd | T | `xschem place_text` | Text dialog then place |
| Insert wire | cmd | W | `xschem wire` | Wire draw mode |
| Insert snap wire | cmd | Shift+W | `xschem snap_wire` | Wire start snapped to nearest pin/net endpoint |
| Insert line | cmd | L | `xschem line` | Graphic line on current layer |
| Insert rect | cmd | R | `xschem rect` | Rectangle |
| Insert polygon | cmd | P | `xschem polygon` | Polygon |
| Insert arc | cmd | Shift+C | `xschem arc` | Arc (3-point) |
| Insert circle | cmd | Ctrl+Shift+C | `xschem circle` | Circle |
| Insert JPG/PNG/SVG image | cmd | | `xschem add_image` | File dialog then place embedded image |
| Grab screen area | cmd | Print Scrn | `xschem grabscreen` | Capture root-window area into an image object |
| Search | cmd | Ctrl+F | `property_search` | Search/select/highlight by attribute (5896) |
| Align to Grid | cmd | Alt+U | `xschem align` | Snap selected objects to current snap |
| Execute TCL command | cmd | | `tclcmd` | Tcl console dialog (5716) |
| Join/Trim wires | cmd | & | `xschem trim_wires` | Merge collinear, remove overlaps, split at T-junctions |
| Break wires at selected instance pins | cmd | ! | `xschem break_wires` | Split wires at pins of selected instances |
| Remove wires running through selected inst. pins | cmd | Ctrl-! | `xschem break_wires 1` | Split and delete wire segments inside selected instances |
| Break wires at mouse position | cmd | Alt-Shift-Right Butt. | `xschem wire_cut noalign` | Cut wire at mouse point |
| Break wires at mouse position, align cut point | cmd | Alt-Right Butt. | `xschem wire_cut` | Cut wire at snapped point |
| Select all connected wires/labels/pins | cmd | Shift-Right Butt. | `xschem connected_nets` | Select net connected to selection |
| Select conn. wires, stop at junctions | cmd | Ctrl-Right Butt. | `xschem connected_nets 1` | Same, stop at 3+-way junctions |

### 1.8 Symbol (xschem.tcl:9167-9228)

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Show symbols | casc | | `.sym.sym` | Radio group `hide_symbols`, see below |
| Set symbol width | cmd | | `input_line ... "set symbol_width"` | Width used by make_symbol |
| Make symbol from schematic | cmd | A | `xschem make_symbol` | Generate .sym from schematic ipin/opin/iopin |
| Make schematic from symbol | cmd | Ctrl+L | `xschem make_sch` | Generate .sch with ports from selected symbol |
| Make schematic and symbol from selected components | cmd | Ctrl+Shift+H | `xschem make_sch_from_sel` | Create subcircuit (LCC) from selection, replace selection by instance; optional .sym |
| Attach net labels to component instance | cmd | Shift+H | `xschem attach_labels` | Add lab_pin labels to all pins of selected instances (dialog) |
| Create symbol pins from selected schematic pins | cmd | Alt+H | `schpins_to_sympins` | Convert selected ipin/opin into symbol pin boxes (5279) |
| Place symbol pin | cmd | Alt+P | `xschem add_symbol_pin` | GUI placement of a symbol pin (PIN layer box) |
| Place schematic input port | cmd | Ctrl+P | `xschem net_label 2` | Place `ipin.sym` |
| Place schematic output port | cmd | Ctrl+Shift+P | `xschem net_label 3` | Place `opin.sym` |
| Place net pin label | cmd | Alt+L | `xschem net_label 1` | Place `lab_pin.sym` |
| Place net wire label | cmd | Alt+Shift+L | `xschem net_label 0` | Place `lab_wire.sym` |
| Change selected inst. texts to floaters | cmd | | `xschem floaters_from_selected_inst` | Flatten instance's symbol texts into schematic texts attached by name |
| Unselect attached floaters | cmd | | `xschem unselect_attached_floaters` | Deselect non-instance objects attached to named instances |
| List of nets | casc | | `.sym.list` | See below |
| Search all search-paths for schematic associated to symbol | chk `search_schematic` | | | Look up .sch by name in all lib paths, not only beside the .sym |
| Allow duplicated instance names (refdes) | chk `disable_unique_names` | | | Don't auto-rename duplicate `name=` |

Show symbols (9167-9178), radio `hide_symbols`, all Accel Alt+B (cycles):

| Item | Value | Description |
|---|---|---|
| Show symbol details | 0 | Full symbol graphics |
| Show instance Bounding boxes for subcircuit symbols | 1 | Boxes only for symbols with schematic |
| Show instance Bounding boxes for all symbols | 2 | Boxes only for all |

List of nets (9210-9222):

| Item | Accel | Action | Description |
|---|---|---|---|
| Print list of highlight nets | J | `xschem print_hilight_net 1` | Dialog listing highlighted nets |
| Print list of highlight nets, with buses expanded | Alt-Ctrl-J | `print_hilight_net 3` | Same, with path + bus expansion |
| Create labels from highlight nets | Alt-J | `print_hilight_net 4` | Place labels for highlighted nets |
| Create labels from highlight nets with 'i' prefix | Alt-Shift-J | `print_hilight_net 2` | Labels with `i` prefix |
| Create pins from highlight nets | Ctrl-J | `print_hilight_net 0` | Place ipin/opin for highlighted nets |

### 1.9 Highlight (xschem.tcl:9260-9304)

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Set schematic to compare and compare with | cmd | | `set compare_sch 1; xschem compare_schematics` | Choose 2nd .sch, highlight graphical differences |
| Swap compare schematics | cmd | | `swap_compare_schematics` | Load the compare-to schematic, keep current as reference (7328) |
| Compare schematics | chk `compare_sch` | | unselect_all, redraw | Toggle diff display |
| View only Probes | chk `only_probes` | 5 | `xschem only_probes` | Dim everything except highlights |
| Highlight net-pin mismatches on sel. instances | cmd | Shift-X | `xschem net_pin_mismatch` | Highlight nets whose name differs from attached pin |
| Highlight duplicate instance names | cmd | # | `xschem check_unique_names 0` | |
| Rename duplicate instance names | cmd | Ctrl+# | `xschem check_unique_names 1` | Auto-rename duplicates |
| Select overlapped instances | cmd | | `xschem warning_overlapped_symbols 1` | Select perfectly overlapping instances |
| Propagate Highlight selected net/pins | cmd | Ctrl+Shift+K | `xschem hilight drill` | Highlight through `propag=` pins (drill) |
| Increment Hilight Color | chk `incr_hilight` | | | Each new highlight gets next color |
| Highlight selected net/pins | cmd | K | `xschem hilight` | Highlight nets of selection (across hierarchy) |
| Send selected net/pins to Viewer | cmd | Alt+G | `xschem send_to_viewer` | Send nets/currents to gaw/bespice |
| Select hilight nets / pins | cmd | Alt+K | `xschem select_hilight_net` | Select all highlighted objects |
| Un-highlight all net/pins | cmd | Shift+K | `xschem unhilight_all` | |
| Un-highlight selected net/pins | cmd | Ctrl+K | `xschem unhilight` | |
| Auto-highlight net/pins | chk `auto_hilight` | | | Highlight net on selection automatically |
| Enable highlight connected instances | chk `en_hilight_conn_inst` | | | Also highlight instances connected to highlighted nets |

### 1.10 Simulation (xschem.tcl:9306-9408)

| Item | Type / Var | Accel | Action | Description |
|---|---|---|---|---|
| Set netlist Dir | cmd | | `set local_netlist_dir 0; set_netlist_dir 1` | Dir chooser for `netlist_dir` (5481) |
| Set top level netlist name | cmd | | `input_line ... "xschem set netlist_name"` | Override netlist filename |
| Show netlist after netlist command | chk `netlist_show` | Shift+A | | Open netlist in viewer after netlisting |
| Keep symbols when traversing hierarchy | chk `keep_symbols` | | | Don't purge loaded symbols on descend/netlist |
| Use netlist directory | radio `local_netlist_dir=0` | | `set_netlist_dir 1` | Use `netlist_dir` |
| Use 'simulation' dir in schematic dir | radio `=1` | | same | `<schdir>/simulation` |
| Use 'simulation/[schname]' dir in schematic dir | radio `=2` | | same | `<schdir>/simulation/<cell>` |
| Configure simulators and tools | cmd | | `simconf` | Simulator configurator dialog (1704; section 3) |
| List running sub-processes | cmd | | `list_running_cmds` | Process list dialog with kill (510); disabled on Windows |
| View last job data | cmd | | `viewdata $execute(data,last)` | stdout of last job |
| View last job errors | cmd | | `viewdata $execute(error,last)` | stderr of last job |
| Utile Stimuli Editor (GUI) | cmd | | `inutile <dir>/stimuli.<cell>` | Edit "utile" stimuli file (144) |
| Utile Stimuli Translate | cmd | | `inutile_translate ...` | Run utile awk chain to make SPICE stimuli (134) |
| Shell [simulation path] | cmd | | `get_shell $netlist_dir` | Terminal in netlist dir (3897) |
| Edit Netlist | cmd | | `edit_netlist [xschem get netlist_name fallback]` | Open netlist in `$editor` (3912) |
| Send highlighted nets to viewer | cmd | Ctrl+Shift+X | `xschem create_plot_cmd` | Write xplot file of highlighted nodes for gaw/bespice/ngspice |
| Changelog from current hierarchy | cmd | | `viewdata [list_hierarchy]` | Show file list of hierarchy w/ mtimes (2441) |
| — | sep | | | |
| Graphs | casc | | `.simulation.graph` | See below |
| LVS | casc | | `.simulation.lvs` | See below |

Graphs (9367-9391):

| Item | Type / Var | Action | Description |
|---|---|---|---|
| Auto highlight plotted nets | chk `auto_hilight_graph_nodes` | | Highlight nets shown in graphs |
| Add waveform graph | cmd | `xschem add_graph` | Place a graph rectangle (layer GRID, `flags=graph`) |
| Add waveform reload launcher | cmd | `xschem place_symbol [find_file_first launcher.sym] "name=h5 descr=\"load waves\" tclcommand=\"xschem raw_read $netlist_dir/<cell>.raw tran\""` | Launcher that loads tran raw |
| Annotate Operating Point into schematic | cmd | `select_raw` → `xschem annotate_op [raw]`, `show_hidden_texts=1` | Back-annotate OP values |
| Live annotate probes with 'b' cursor | chk `live_cursor2_backannotate` | | Probes show values at graph cursor b |
| Hide graphs if no spice data loaded | chk `hide_empty_graphs` | redraw | |

LVS (9393-9407):

| Item | Var | Description |
|---|---|---|
| LVS netlist + Top level is a .subckt | `lvs_netlist` | LVS-oriented SPICE netlist, top wrapped as .subckt |
| Upper case .SUBCKT and .ENDS | `uppercase_subckt` | |
| Top level is a .subckt | `top_is_subckt` | Wrap top level in .subckt/.ends |
| Set 'lvs_ignore' variable | `lvs_ignore` | Honour `lvs_ignore` attrs; runs `xschem rebuild_connectivity; unhilight_all` |
| Use 'spiceprefix' attribute | `spiceprefix` | Prepend `spiceprefix` to instance names in netlist |

### 1.11 Help (xschem.tcl:8823-8827)

| Item | Accel | Action | Description |
|---|---|---|---|
| Help | ? | `textwindow $XSCHEM_SHAREDIR/xschem.help ro` | Read-only help text |
| Keys | | `textwindow $XSCHEM_SHAREDIR/keys.help ro` | Key reference text |
| Show Keybindings | | `show_bindkeys` | Overlay window showing key map (8535) |
| About XSCHEM | | `about` | About dialog w/ links (5843) |

### 1.12 Waves (menubar cascade, xschem.tcl:9053-9075)

| Item | Type | Action | Description |
|---|---|---|---|
| External viewer | cmd | `waves external` | Launch configured external viewer `sim(<type>wave,...)` (gaw, bespice, ...) (2779-2834) |
| — | sep | | |
| Clear | cmd | `xschem raw_clear` | Unload all raw files |
| — | sep | | |
| Op Annotate | cmd | `select_raw` → `xschem annotate_op` | Same as Graphs → Annotate OP |
| Op | cmd | `waves op` → `load_raw op` | Unload, file-select .raw, `xschem raw_read f op` (8739-8754) |
| Dc | cmd | `waves dc` | Load DC raw |
| Ac | cmd | `waves ac` | Load AC raw |
| Tran | cmd | `waves tran` | Load transient raw |
| Noise | cmd | `waves noise` | Load noise raw |
| Sp | cmd | `waves ac` | S-param (loads as ac) |
| Spectrum | cmd | `waves ac` | Spectrum (loads as ac) |

### 1.13 Status bar (xschem.tcl:9440-9455, packed 8670-8686)

Left→right: `SNAP:` entry (`cadsnap`, applied on mouse Leave; green/red bg if snap valid/invalid, actions.c:307-309) · `GRID:` entry (`cadgrid`, same) · `MODE:` label = `$netlist_type` · `Stretch:` + 10 (shows `DRAW WIRE!`/`DRAW LINE!` persistent-mode warning, callback.c:4464-4466) · 9 (`$enable_stretch` value) · 8 (`CAPS LOCK SET!`/`NUM LOCK SET!` warnings, callback.c:4446-4459) · 1 message area (`statusmsg(...,1)`: `mouse = X Y - selected: N w=W h=H` during ops, callback.c:2375).

### 1.14 Toolbar (setup_toolbar 7872-7920, toolbar_add 7925-7941, toolbar_show 7948-7994, add_toolbuttons 8015-8058)

Contributed by N. Johnson. Buttons are 24×24 GIF/PNG images `img<Name>` from `resources.tcl` (scaled by integer `toolbar_icon_zoom`), tooltip via `balloon`. `toolbar_list` (overridable in xschemrc) defines order; `---` = separator. Netlist/Simulate/Waves are always packed at the far end (right if horizontal, bottom if vertical) regardless of `toolbar_list`. Defaults `toolbar_visible=1`, `toolbar_horiz=1`. Horizontal toolbar packs above tab bar; vertical packs left of drawing area.

| # | Name | Command | Tooltip |
|---|---|---|---|
| 1 | FileOpen | `xschem load` | Open File |
| 2 | FileSave | `xschem save` | Save File |
| 3 | FileReload | confirm + `xschem reload` | Reload File |
| — | sep | | |
| 4 | EditUndo | `xschem undo; xschem redraw` | Undo |
| 5 | EditRedo | `xschem redo; xschem redraw` | Redo |
| 6 | EditCut | `xschem cut` | Cut |
| 7 | EditCopy | `xschem copy` | Copy |
| 8 | EditPaste | `xschem paste` | Paste |
| 9 | EditDelete | `xschem delete` | Delete |
| — | sep | | |
| 10 | EditDuplicate | `xschem copy_objects` | Duplicate objects |
| 11 | EditMove | `xschem move_objects` | Move objects |
| — | sep | | |
| 12 | EditPushSch | `xschem descend` | Push schematic |
| 13 | EditPushSym | `xschem descend_symbol` | Push symbol |
| 14 | EditPop | `xschem go_back` | Pop |
| — | sep | | |
| 15 | ViewRedraw | `xschem redraw` | Redraw |
| 16 | ViewToggleColors | `xschem toggle_colorscheme` | Toggle Color Scheme |
| — | sep | | |
| 17 | ToolInsertSymbol | `xschem place_symbol` | Insert Symbol |
| 18 | ToolInsertText | `xschem place_text` | Insert Text |
| 19 | ToolInsertWire | `xschem wire` | Insert Wire |
| 20 | ToolInsertLine | `xschem line` | Insert Line |
| 21 | ToolInsertRect | `xschem rect` | Insert Rectangle |
| 22 | ToolInsertPolygon | `xschem polygon` | Insert Polygon |
| 23 | ToolInsertArc | `xschem arc` | Insert Arc |
| 24 | ToolInsertCircle | `xschem circle` | Insert Circle |
| — | sep | | |
| 25 | ToolSearch | `property_search` | Search |
| — | sep | | |
| 26 | ToolJoinTrim | `xschem trim_wires` | Join/Trim Wires |
| 27 | ToolBreak | `xschem break_wires` | Break wires at selected instance pin intersections |
| end | Netlist | `xschem netlist -erc` | Create netlist |
| end | Simulate | `simulate_from_button` | Run simulation |
| end | Waves | `waves` (no type → `load_raw`) | View results |

Defined-but-unused: icons `ViewZoomIn/ViewZoomOut/ViewZoomBox` exist in resources.tcl, their `toolbar_add` calls are commented out (8038-8040); `FileNew/FileNewSym/FileMerge` commented, no icons. User extension: `src/add_custom_button.tcl` example = `lappend toolbar_list MyButton`, `image create photo imgMyButton`, `toolbar_add MyButton {cmd} "tip" $topwin`, `pack $topwin.toolbar.bMyButton -side left`.

### 1.15 Canvas right-click context menu (`context_menu`, xschem.tcl:7501-7643; actions in `context_menu_action`, callback.c:2029-2149)

Shown on Button-3 **release** if no place/move/copy/zoom operation was pending (`end_place_move_copy_zoom()` returned 0) and semaphore < 2 (callback.c:4265-4268). Implemented as an override-redirect toplevel `.ctxmenu` of flat buttons with 16×16 icons (`Ctxmenu*`, scaled by `ctxmenu_icon_zoom`), positioned at pointer −10 px, clamped to screen, destroyed on pointer `<Leave>`. Proc returns a code; C dispatches. Contents depend on whether anything is selected.

| Code | Label | Icon | Shown when | Action (C) |
|---|---|---|---|---|
| 21 | Abort command | CtxmenuAbort | always (first) | `abort_operation()` |
| 9 | Open most recent | CtxmenuRecent | no selection | `xschem load -gui [lindex $recentfile 0]` |
| 10 | Edit attributes | CtxmenuEdit | always | `edit_property(0)` (dialog) |
| 11 | Edit attr in editor | CtxmenuEdit | always | `edit_property(1)` (external editor) |
| 12 | Descend schematic | CtxmenuDown | selection | `descend_schematic(0,1,1,1)` |
| 13 | Descend symbol | CtxmenuDownSym | selection | `descend_symbol()` |
| 18 | Delete selection | CtxmenuDelete | selection | `delete(1)` |
| — | Rotate selection | CtxmenuRotate | selection | Tcl `xschem rotate` directly |
| — | Flip selection | CtxmenuFlip | selection | Tcl `xschem flip` |
| — | Rotate in-place sel. | CtxmenuRotate | selection | Tcl `xschem rotate_in_place` |
| — | Flip in-place sel. | CtxmenuFlip | selection | Tcl `xschem flip_in_place` |
| 7 | Cut selection | CtxmenuCut | selection | `save_selection(2); delete(1)` (clipboard file) |
| 15 | Copy selection | CtxmenuCopy | selection | `save_selection(2)` |
| 16 | Move selection | CtxmenuMove | selection | `move_objects(START,...)` |
| 17 | Duplicate selection | CtxmenuDuplicate | selection | `copy_objects(START)` |
| 14 | Go to upper level | CtxmenuUp | no selection | `go_back(1)` |
| 1 | Insert symbol | CtxmenuSymbol | no selection | `start_place_symbol()` |
| 2 | Insert wire | CtxmenuWire | no selection | `start_wire(mx,my)` at click point |
| 3 | Insert line | CtxmenuLine | no selection | `start_line(mx,my)` |
| 4 | Insert box | CtxmenuBox | no selection | `new_rect(PLACE,mx,my)` |
| 5 | Insert polygon | CtxmenuPoly | no selection | `new_polygon(PLACE,mx,my)` |
| 19 | Insert arc | CtxmenuArc | no selection | `new_arc(PLACE,180,mx,my)` |
| 20 | Insert circle | CtxmenuCircle | no selection | `new_arc(PLACE,360,mx,my)` |
| 6 | Insert text | CtxmenuText | no selection | `place_text(0,mx,my)` then move |
| 8 | Paste clipboard | CtxmenuPaste | no selection | `merge_file(2,".sch")` |

Pack order — no selection: Abort, Open most recent, Edit attributes, Edit attr in editor, Go to upper level, Insert symbol/wire/line/box/polygon/arc/circle/text, Paste. With selection: Abort, Edit attributes, Edit attr in editor, Descend schematic, Descend symbol, Delete, Rotate, Flip, Rotate in-place, Flip in-place, Cut, Copy, Move, Duplicate. (14 entries each; 25 distinct.)

Note: right-button release while in wire/line/etc. mode ends that mode instead (end_place_move_copy_zoom). Modified right-clicks (Shift/Ctrl/Alt) are separate actions (select connected nets, wire cut), see Tools menu accelerators and section 2.

### 1.16 Tabbed interface

- Var `tabbed_interface` (default 1, xschem.tcl:9947; xschemrc:597). Toggle in View menu only when a single window/tab is open (menu entry disabled otherwise). Max 20 tabs/windows total (`MAX_NEW_WINDOWS`, xschem.h:145).
- Tab bar `.tabs` (setup_tabbed_interface, xschem.tcl:8073-8116): button `.tabs.x0` (main tab, win_path `.drw`), one button `.tabs.xN` per extra tab (win_path `.xN.drw`, created in C create_new_tab, xinit.c:1700-1790), and a trailing `.tabs.add` button ` + ` (`xschem new_schematic create`, tooltip "Create a new tab"). Packed below horizontal toolbar / above drawing (pack_tabs, 8060-8071).
- Tab label = file tail of schematic + `*` when modified (`set_tab_names *`, actions.c:216); tooltip = full path; current tab bg Palegreen (DarkGreen in dark GUI), others default (set_tab_names, 8200-8224).
- Click tab → `xschem new_schematic switch .xN.drw` (saves/restores per-tab Tcl context via `save_ctx`/`restore_ctx`, per-tab xctx struct).
- Drag tab onto another (Button-1 press/release) → swap tab positions (swap_tabs, 8124-8152).
- Ctrl+Right / Ctrl+Left → `next_tab` / `prev_tab` by on-screen x order (8154-8198; callback.c:3800-3835). (Ctrl+arrow is passed to graphs instead when mouse is over a graph.)
- New tab starts in spice mode, inherits line width, default snap/grid, initial zoom (xinit.c:1760-1785).
- Close: Ctrl+W / File→Close / tab menu "Close tab" → `xschem exit`; asks if modified ("UNSAVED data: want to exit?"). Closing main tab `.drw` with others open swaps first other tab into `.drw` (C `swap_tabs`, xinit.c:1343-1390). Window manager close → `xschem exit closewindow` (closes tab; exits program on last). Opening an already-open file warns ("already open").
- Tab right-click → `tab_context_menu` (7753-7870), actions in `tab_ctx_cmd` (7645-7751):

| Entry | Shown | Action |
|---|---|---|
| Tab menu | always (disabled header) | — |
| Copy filename | always | Put tab's file path on clipboard |
| Open directory | always | `xdg-open` dir of file (Windows `start`) |
| Open circuit dir. term. | always | `$terminal` in file's dir |
| Open sim. dir. term. | always | `$terminal` in `$netlist_dir` |
| Open symbol / Open schematic | counterpart exists | Open the .sym of a .sch (or .sch of a .sym) — switch to it if already open, else new tab (symbol tab gets `netlist_type symbol`) |
| Edit netlist | tab is not a symbol | `$editor` on that tab's netlist (alert if not generated) |
| Save | always | Save that tab's file without switching view |
| Close tab | always | `xschem exit` on that tab |

- Non-tabbed mode: `new_schematic create` makes a new toplevel `.xN` with its own menubar/toolbar/statusbar (`build_widgets`/`pack_widgets`), per-window bindings, re-runs `user_startup_commands` (xinit.c:1588-1690). Focus/Enter events switch the active context; objects can be copied across windows (callback.c:4503-4548). Main-window close swaps another window into `.drw` (`swap_windows`).

### 1.17 Extension hooks

- `tcl_files` (xschemrc:529-532): list of Tcl files sourced at startup after GUI build (`source_user_tcl_files`, xschem.tcl:9600; xinit.c:3082). Shipped examples:
  - `add_custom_menu.tcl`: `$topwin.menubar insert Netlist cascade -label Test -menu $topwin.menubar.test` + `add command` entries → user menus inserted before Netlist button.
  - `add_custom_button.tcl`: add toolbar button (see 1.14).
- `user_startup_commands` (xschemrc:567): evaluated at startup and for each new window (xinit.c:1678, 3083). `postinit_commands` (xschemrc:538): evaluated after init (xinit.c:3107).
- `toolbar_list`, `toolbar_visible`, `toolbar_horiz`, `toolbar_icon_zoom`, `ctxmenu_icon_zoom`, `tabbed_interface`, `new_symbol_browser*` settable in xschemrc (xschemrc:252-259, 585-605).
- `replace_key(<key>)` array: rebind keys per window (`set_replace_key_binding`, xschem.tcl:9591).

### 1.18 Counts

| Thing | Count |
|---|---|
| Menubar entries | 15 (11 menus + Waves cascade + Netlist + Simulate buttons + 1 disabled spacer) |
| Top-level pull-down menus | 12 (File, Edit, Options, View, Properties, Layers, Tools, Symbol, Highlight, Simulation, Help, Waves) |
| Submenus (cascades inside menus) | 9 (Open recent, Image export, Crosshair, Netlist format/Symbol mode, Show/Hide, Show symbols, List of nets, Graphs, LVS) |
| Static menu entries (excl. separators, menubar level) | 218 (= 209 leaf items + 9 cascades) |
| Dynamic entries | Layers: `cadlayers` (22 default); Open recent: ≤10 |
| Per menu (entries incl. submenus) | File 19+6, Edit 23, Options 22+3+7, View 17+7, Properties 7, Tools 21, Symbol 17+3+5, Highlight 17, Simulation 19+6+5, Help 4, Waves 10 |
| Toolbar buttons | 30 (27 in default `toolbar_list` + Netlist/Simulate/Waves), 7 separators |
| Canvas context-menu entries | 25 distinct (14 without selection, 14 with selection) |
| Tab context-menu entries | 8 actions + 1 header |

## 2. Keybindings and mouse interactions

Paths below are relative to the source tree `/nix/store/rhvgzl9mjl58nr0qm6l98fxbiadg3q98-source`. `cb` = `src/callback.c`, `tcl` = `src/xschem.tcl`.

### 2.0 Event plumbing (needed to reproduce behaviour exactly)

| Item | Behaviour | Source |
|---|---|---|
| Tk to C bridge | Every canvas event is forwarded as `xschem callback %W %T %x %y <keysym> <button> <aux> %s`. Double-click on buttons 1, 2 and 3 is sent as pseudo-event `-3`. | tcl:8560-8650 (`set_bindings`), tcl:8599 |
| Wheel | Buttons 4 and 5 are X11 wheel events. On Tk >= 8.7 and on Windows, `<MouseWheel>` is converted to button 4/5 presses. | tcl:8588-8596, tcl:8640 |
| Modifier normalisation | NumLock (Mod2) and CapsLock are masked out. `rstate` = state minus Shift minus Button1. Upper-case keysyms therefore stand for Shift+letter. | cb:4610-4615 |
| Alt alias | Alt (Mod1) and the Win/Super key (Mod4) are interchangeable: `SET_MODMASK` means Alt or Win is held, and `EQUAL_MODMASK` means Alt or Win is the only modifier. | cb:26-27 |
| Re-entrancy guard | `xctx->semaphore >= 2`, for example while a modal dialog is open, makes most editing keys no-ops. Redraw, pan and zoom still work. | cb:4598-4605 |
| Mouse coords | `mousex_snap = round(mousex / cadsnap) * cadsnap`. All placement uses snapped coordinates. Hit-testing for selection uses unsnapped coordinates, except as noted in 2.3. | cb:4616-4619 |
| `fix_mouse_coord` | Re-queries pointer position on Key events (workaround for remote desktops). | cb:4568-4578 |
| Status bar | Shows `mouse = x y - selected: N path: …`, plus `w= h=` deltas during an operation. It also shows a mode banner (`DRAW WIRE!` / `DRAW LINE!` / `DRAW POLYGON!` / `DRAW ARC!` / `DRAW RECTANGLE!`), CapsLock/NumLock warnings, netlist type, and editable snap and grid fields. | cb:4422-4482, tcl:8680-8684 |
| User remap | `set replace_key(<from>) <to>` in xschemrc rebinds keys via `key_binding` (Tk event-spec syntax, e.g. `Control-q` -> `Escape`). | src/xschemrc:461-467, tcl:1101-1145, tcl:9591 |
| Extra mouse buttons | An optional `~/.xschem/mouse_bindings.tcl` (added with `lappend tcl_files`) maps buttons 8 and 9 with any modifier combination. Default: 8 = `zoom_full center`, 9 = `unhilight_all`, Shift+8 = force save. | src/mouse_bindings.tcl:38-116 |
| Escape in Tk | Escape also destroys an open context menu (`.ctxmenu`). | tcl:8612 |
| Focus | `autofocus_mainwindow=1` makes the canvas grab focus on motion and button events (focus-follows-mouse). | tcl:8603-8628 |

### 2.1 Keyboard bindings (schematic and symbol canvas, `handle_key_press` cb:2479-3999)

Column notes:
- **Ctx**:
  - `any` = schematic or symbol editing.
  - `G>` = when the pointer is inside a graph, the key is first routed to the graph handler (see 2.2).
  - `cad` = the binding depends on `cadence_compat`.
- **help?**: `y` if the binding is listed in `src/keys.help`, `NO` if it exists only in callback.c.
- Upper-case letters imply Shift.

| Key | Mod | Ctx | Action | cb line | help? |
|---|---|---|---|---|---|
| 0 | – | any | Set selected net/label logic value 0 (`logic_set`) | 2487 | y |
| 1 | – | any | Set logic 1 | 2487 | y |
| 2 | – | any | Set logic X | 2487 | y |
| 3 | – | any | Set logic Z | 2487 | y |
| 4 | – | any | Toggle logic value 1->0, 0->1, X->X | 2487 | y |
| 0-9 | Ctrl | any | Set current drawing layer (`rectcolor`) to N and update the layer toolbar button | 2497, 2525 | y |
| 5 | – | any | Toggle "view only probes" (`only_probes`) | 2514 | y |
| a | – | any, G> | Make symbol from the current schematic's pins (confirm dialog; saves first) | 2542 | y |
| a | Ctrl | any, G> | Select all | 2557 | y |
| A | – | any, G> | Toggle `netlist_show` (show netlist in a window after netlisting) | 2566 | y |
| A | Ctrl | G only | No schematic action. In a graph, toggles horizontal cursor 1 when `graph_use_ctrl_key` is set | 2584 | NO |
| b | – | any, G> | Merge (import) a file into the current schematic | 2592 | y |
| b | Ctrl | any, G> | Toggle display of text inside symbols (`sym_txt`) | 2601 | y |
| b | Alt | any | Cycle `hide_symbols` 0 -> 1 -> 2 (full / bbox for subckt symbols / bbox for all) | 2618 | y |
| B | – | any, G> | Edit file header/license metadata (`update_schematic_header`, tcl:1404) | 2627 | y |
| B | Ctrl | G only | Graph: toggle hcursor2 (with `graph_use_ctrl_key`) | 2635 | NO |
| c | – | any | Duplicate (copy) selection, attached to the mouse | 2643 | y |
| c | Ctrl | any | Copy selection to the clipboard file (`save_selection(2)`) | 2657 | y |
| c | Alt | any | Duplicate with `connect_by_kissing` (insert wires where moved pins detach from touching pins/wires) | 2665 | y |
| C | – | any | Start arc (3-point) | 2679 | y |
| C | Ctrl | any | Start circle (3-point, sweep 360) | 2692 | y |
| d | – | any | Unselect the object under the mouse | 2706 | y |
| d | Ctrl | any | Delete files dialog (`delete_files`) | 2715 | y |
| D | – | any | Unselect by area (next drag) | 2721 | y |
| e | – | any | Descend into the selected instance's schematic | 2738 | y |
| e | Ctrl | any | Go back to the parent (pop hierarchy) | 2743 | y |
| e | Alt | any | Open the selected instance's schematic in a new window/tab (`open_sub_schematic`, tcl:3959) | 2747 | y |
| E | Alt | any | Open the selected schematic in a new xschem process | 2756 | y |
| f | – | any, G> | Zoom full (`zoom_full_center` option centers the drawing) | 2765 | y |
| f | Ctrl | any, G> | Find/select by substring or regex (`property_search`, tcl:5896) | 2775 | y |
| f | Alt | any | Horizontal flip of each selected object around its own anchor (works during move/copy) | 2783 | y |
| F | – | any | Horizontal flip of the selection as a group (works during move/copy) | 2797 | y |
| F | Ctrl | any | Zoom to fit the selection | 2810 | y |
| g | – | any | Halve snap (`cadsnap / 2`) | 2817 | y |
| g | Ctrl | any | Snap value input dialog | 2823 | y |
| g | Alt | any | Highlight selected nets and send them to the waveform viewer. Targets the xschem graph if the graph dialog is open or a graph is selected, otherwise Gaw or Bespice per `sim(spicewave,default)` | 2829 | y (says "gaw") |
| G | – | any | Double snap | 2864 | y |
| h | – | any | Toggle horizontal constraint (`constr_mv=1`) for move/copy/wire/line | 2879 | y |
| h | Ctrl | any | Launcher: follow `url`, run `program`, or eval `tclcommand` of the selected instance (actions.c:1805) | 2873 | y |
| h | Alt | any | Create symbol pins from schematic pins (`schpins_to_sympins`, tcl:5279) | 2898 | y |
| H | – | any | Attach net labels to the selected instance pins (`attach_labels_to_inst`) | 2903 | y |
| H | Ctrl | any | Make a schematic and a symbol from the selected components | 2907 | y |
| i | – | any | Descend into the selected instance's symbol | 2912 | y |
| i | Ctrl | any | Insert-symbol dialog (persistent file dialog, or the new browser if `new_symbol_browser`) | 2917 | y |
| i | Alt | any | Edit the selected symbol in a new window | 2924 | y |
| I | – | any | Insert symbol (file dialog, then place) | 2932 | y |
| I | Alt | any | Edit the selected symbol in a new xschem process | 2941 | NO |
| j | – | any | Print the list of highlighted nets/pins | 2949 | y |
| j | Ctrl | any | Create ipins/ports from highlighted nets (`print_hilight_net(0)`) | 2954 | y |
| j | Alt | any | Create labels without the `i` prefix from highlighted nets | 2958 | y |
| j | Ctrl+Alt | any | Print highlighted nets with bus/label expansion | 2962 | y |
| J | Alt | any | Create labels with the `i` prefix from highlighted nets | 2968 | y |
| k | – | any | Highlight selected nets | 2974 | y |
| k | Ctrl | any | Unhighlight selected nets | 2985 | y |
| k | Alt | any | Select the whole net attached to the selected wire/label/pin | 2982 | y |
| K | – | any | Clear all highlights | 2991 | y |
| K | Ctrl | any | Highlight with "drill": propagates through instances whose pins have `propag=` | 2999 | y |
| l | – | any | Start line (current layer) | 3008 | y |
| l | Ctrl | any | Create a schematic view from the selected symbol (`create_sch_from_sym`) | 3024 | y |
| l | Alt | any | Place `lab_pin.sym` at the mouse (actions.c:1451) | 3028 | y |
| L | – | any | Toggle `orthogonal_wiring` (auto H-V/V-H routing) | 3033 | y |
| L | Alt | any | Place `lab_wire.sym` | 3043 | y |
| m | – | any, G> | Move selection. With `enable_stretch` set, also stretches attached wires | 3048 | y |
| m | Ctrl | any, G> | Move selection stretching attached wires (`select_attached_nets`). Plain move if `enable_stretch` is set | 3066 | y |
| m | Alt | any | Move with `connect_by_kissing` | 3082 | y |
| M | – | any | Move with `connect_by_kissing` (same as Alt+m) | 3096 | y |
| M | Ctrl | any | Move with stretch plus `connect_by_kissing` | 3111 | y |
| n | – | any | Hierarchical netlist with ERC (`xschem netlist -erc`) | 3125 | y |
| n | Ctrl | any | Clear schematic (new empty) | 3130 | y |
| N | – | any | Netlist the current level only (no hierarchy). Turns the Netlist menu button green or red | 3136 | y |
| N | Ctrl | any | Clear symbol (new empty symbol) | 3175 | y |
| o | Ctrl | any | Open/load file (file dialog or the new file browser) | 3181 | y |
| O | – | any | Toggle light/dark color scheme | 3201 | y |
| O | Ctrl | any | Load the most recent file | 3198 | y |
| p | – | any | Start polygon | 3227 | y |
| p | Ctrl | any | Place `ipin.sym` | 3224 | y |
| p | Alt | any | Add a symbol pin box (PINLAYER, `name=XXX dir=inout`) attached to the mouse | 3214 | y |
| P | – | any | Re-center the view on the mouse position | 3243 | y (says "Pan") |
| P | Ctrl | any | Place `opin.sym` | 3249 | y |
| q | – | any | Edit properties dialog (`edit_property(0)`) | 3262 | y |
| q | Ctrl | any | Quit xschem (`quit_xschem`) | 3255 | y |
| q | Alt | any | Open the .sch file, or the selected symbol's file, in a text editor | 3266 | y |
| Q | – | any | Edit properties in an external editor (`edit_property(1)`) | 3282 | y |
| Q | Ctrl | any | View properties, read-only (`edit_property(2)`) | 3287 | y |
| r | – | any | Start rectangle | 3292 | y |
| r | Ctrl | cad=1, G> | Run simulation (confirm) | 3306 | NO |
| r | Alt | any | Rotate each selected object around its own anchor (works during move/copy) | 3318 | y |
| R | – | any | Rotate the selection 90 degrees as a group (works during move/copy) | 3332 | y |
| s | – | cad=0, G> | Run simulation (confirm, invokes the `Simulate` menu) | 3349 | y |
| s | – | cad=1 | Start/end a wire snapped to the nearest pin or net endpoint | 3362 | NO |
| s | Ctrl | any, G> | Save. Untitled files go to Save As | 3366 | y |
| s | Alt | any | Reload from disk (confirm) | 3380 | y |
| s | Ctrl+Alt | any | Save as symbol | 3394 | y |
| S | – | any | Change element order (renumber the selected object's index) | 3400 | y |
| S | Ctrl | any | Save as (schematic) | 3405 | y |
| t | – | any, G> | Place text (dialog, then text follows the mouse) | 3411 | y |
| t | Ctrl | G only | No schematic action. Graph: dataset tracking | 3428 | NO |
| T | – | any | Toggle `spice_ignore` / `verilog_ignore` / `vhdl_ignore` / `tedax_ignore` (per current netlist type) on selected instances | 3436 | y |
| u | – | any | Undo | 3442 | y |
| u | Ctrl | any | Unselect attached "floater" objects (shapes/texts with a `name=` attribute) | 3461 | y |
| u | Alt | any | Align selection/schematic to the snap grid (`round_schematic_to_grid`, optional autotrim) | 3448 | y |
| U | – | any | Redo | 3467 | y |
| v | – | any | Toggle vertical constraint (`constr_mv=2`) | 3475 | y |
| v | Ctrl | any | Paste from the clipboard (`merge_file(2)`); pasted objects follow the mouse | 3495 | y |
| v | Alt | any | Vertical flip around each object's anchor | 3499 | y |
| V | – | any | Vertical flip of the selection (rotate 180 + flip) | 3523 | y |
| V | Ctrl | any | Cycle netlist type spice -> vhdl -> verilog -> tedax -> symbol | 3546 | y |
| w | – | any | Start wire | 3554 | y |
| w | Ctrl | any | Close the current schematic/tab (`xschem exit`) | 3572 | y |
| W | – | cad=0 | Start/end a wire snapped to the closest pin or net endpoint | 3581 | y |
| x | – | any | Start a new xschem process (new CAD session) | 3588 | y |
| x | Ctrl | any | Cut selection to the clipboard | 3600 | y |
| x | Alt | any | Toggle crosshair (`draw_crosshair`) | 3592 | y |
| X | – | any | Highlight mismatches between instance pin names and attached net names | 3610 | y |
| X | Ctrl | any | Create an ngspice `xplot` command file for highlighted nets | 3614 | y |
| y | – | any | Toggle `enable_stretch` | 3619 | y |
| z | – | any | Zoom box (no-op while drawing a rect/line/wire/poly/arc) | 3626 | y |
| z | Ctrl | any | Zoom out (one step, factor 1.2) | 3632 | NO (help says "Zoom out", but Ctrl+z is not undo) |
| z | Alt | cad=1 | Toggle `snap_cursor` | 3635 | NO |
| Z | – | any | Zoom in | 3649 | y |
| Space | – | any | During wire/line/move: cycle `manhattan_lines` 0 -> 1 -> 2. Otherwise: start pan (view follows the mouse until the next click) | 3655 | y |
| _ | – | any | Toggle `change_lw` (line width follows zoom) | 3680 | y |
| % | – | any | Toggle grid drawing | 3691 | y |
| $ | – | any | Toggle `draw_pixmap` (backing pixmap) | 3706 | y |
| $ | Ctrl | any | Toggle `draw_window` (direct window drawing vs XCopyArea) | 3712 | y |
| = | Ctrl | any | Toggle fill pattern for filled layers (effectively 0 <-> 1) | 3724 | y |
| + | Ctrl | any | Line width +0.5 | 3752 | y |
| - | Ctrl | any | Line width -0.5 (min 0) | 3760 | y |
| - | Alt | any | Line width input dialog | 3767 | y |
| Return | – | drawing polygon | Close the polygon (last point = first) and finish | 3772 | NO |
| Escape | any | any | Abort the current operation, unselect, redraw. Also sets `tclstop=1` (stops a running simulation) and clears persistent-wire state | 3778 | y |
| Delete | – | any | Delete selection | 3793 | y |
| Right | – | any, G> | Pan the view right (content moves left by 200 x zoom) | 3800 | y |
| Right | Ctrl | any, G> | Next tab | 3801 | y |
| Left | – | any, G> | Pan left | 3823 | y |
| Left | Ctrl | any, G> | Previous tab | 3824 | y |
| Down | – | any, G> | Pan down | 3846 | y |
| Up | – | any, G> | Pan up | 3856 | y |
| BackSpace | – | any | Go back to the parent schematic | 3866 | y |
| Print | – | any (unix+cairo) | Screen grab: B1-drag a screen rectangle, which is inserted as an embedded image (`flags=image,unscaled`, base64 `image_data`) on GRIDLAYER (draw.c:178-290) | 3872 | y |
| Insert | – | any | Insert symbol (file dialog then place, or the new browser) | 3879 | y |
| Insert | Shift | any | Persistent insert-symbol dialog | 3880 | y |
| * | (Shift) | any | PostScript/PDF export (`ps_draw(7)`) | 3897 | y |
| * | Ctrl | any | PNG/XPM image export | 3902 | y |
| * | Alt | any | SVG export | 3906 | y |
| & | – | any | Join/break/trim wires (`trim_wires`, check.c:151) | 3912 | y |
| \\ | – | any | Toggle fullscreen | 3919 | y |
| > | (Shift) | any | Draw only the current layer | 3926 | y |
| < | (Shift) | any | Draw all layers | 3933 | y |
| ? | – | any | Open help text (`xschem.help`) | 3939 | y |
| / | – | any | Show the key-binding help window (`show_bindkeys`, tcl:8535) | 3943 | y |
| : | – | any | Toggle flat netlist | 3948 | y |
| # | – | any | Highlight instances with duplicate names (refdes) | 3959 | y |
| # | Ctrl | any | Rename duplicate instance names | 3960 | y |
| ! | – | any | Break wires at the selected instance pins | 3983 | y |
| ! | Ctrl | any | Remove wires running through the selected instance pins (`break_wires_at_pins(1)`, check.c:534) | 3984 | NO |
| ; ~ \| | Ctrl | – | Placeholders, disabled (`if(0 && …)`) | 3968-3982 | – |
| Ctrl+Enter | – | Tk dialogs | Confirm and close dialog boxes (Tk-level, not callback.c) | – | y |

**Mismatches between keys.help and callback.c**
- **In callback.c, missing from keys.help:** Alt+Shift+I, Ctrl+Shift+A, Ctrl+Shift+B, Ctrl+t, Ctrl+r (cadence), s (cadence), Alt+z (cadence), Return (polygon close), Ctrl+!, and all graph keys (2.2).
- **In keys.help, not handled by callback.c:** only Ctrl+Enter (dialog-level).
- **Wording differences:**
  - keys.help "Shift P = Pan view area": the code re-centers on the mouse.
  - keys.help "Left = Move right / Right = Move left": this describes content movement.
  - The menu shows Alt-Z for "Draw snap cursor", but it only works when `cadence_compat=1` (cb:3634).

### 2.2 Graph-context keys (pointer inside a graph rectangle, `waves_callback` cb:410-1380)

**Gating (`waves_selected`, cb:29-104).** A key or mouse event goes to the graph only when all of the following hold:
- The pointer is inside a graph rect (GRIDLAYER with `flags&1`).
- No schematic operation is in progress (zoom/rect/line/wire/pan/select/move/copy).
- Alt is not held.
- The event is not a middle-button event and not Shift+B1.
- If `graph_use_ctrl_key=1`, Ctrl is held.
- If the graph has `lock=true`, Ctrl is held.

"Access" keys (a, b, s, m, t, A, B, f) additionally require Ctrl when `graph_use_ctrl_key` is set.

| Key | Area | Action | cb line |
|---|---|---|---|
| a | plot | Toggle vertical cursor A at the mouse x (per-graph `cursor1_x` when `private_cursor`) | 726 |
| b | plot | Toggle cursor B. With `live_cursor2_backannotate=1`, back-annotates schematic values at B | 743 |
| s | plot | Swap cursors A and B | 773 |
| m | plot | Toggle the measurement tooltip (y/x at the mouse) | 821 |
| t | plot | Toggle dataset tracking: pick the closest wave's dataset (analog only) | 827, 1155 |
| A | plot | Toggle horizontal cursor 1 (`hcursor1_y`), master graph only | 1132 |
| B | plot | Toggle horizontal cursor 2 (`hcursor2_y`) | 1144 |
| f | plot / left axis | Full X zoom (plot area) or full Y zoom (left Y-axis area) | 1264 |
| Left / Right | plot | Pan X by 5% of width (all graphs linked by sim_type unless unlocked) | 1184, 1211 |
| Left / Right | left axis | Y zoom out/in 20% around the mouse y (analog) | 1186 |
| Down / Up | plot | X zoom out/in 20%, anchored at the mouse x fraction | 1238, 1251 |

### 2.3 Mouse interactions (schematic canvas)

| Gesture | Condition | Action | Source |
|---|---|---|---|
| B1 click on object | `intuitive_interface=0` | Unselect all, then select the closest object | cb:4147, 4206 |
| B1 click on object | `intuitive_interface=1` (default) | If not already selected: unselect all, then select it. If already selected: keep the selection (so a drag moves the whole group) | cb:4181-4206 |
| B1 click on empty area | – | Clear selection | cb:4181 |
| Shift+B1 click | – | Add the object to the selection | cb:4206 |
| Ctrl+B1 click (no motion, on release) | – | Launcher: `url`, `program`, `tclcommand` attributes of the clicked instance | cb:4270-4292 |
| Alt+B1 click | – | Unselect the object under the mouse | cb:4065 |
| B1 drag on empty area | – | Area select. Drag right (`nl_dir=0`): objects fully inside. Drag left (`nl_dir=1`) with `select_touch=1`: objects touching the box. `incremental_select=1` previews live. Starts after an 8 px threshold | cb:2422-2438, actions.c:3819-3881 |
| Shift+B1 drag | – | Additive area select (does not clear the previous selection) | cb:2452-2469 |
| Ctrl+B1 drag | – | Area select for stretch: wires crossing the box edge get only their inside endpoint selected, so a later move stretches them. Swapped with plain drag when `enable_stretch=1` | cb:2385, 4354-4360 |
| Ctrl+Shift+B1 drag | – | Additive stretch-select | cb:2452 |
| Alt+B1 drag, or `D` then drag | – | Unselect by area (same direction rules) | cb:2388, 2441-2448 |
| B1 drag on object | intuitive | Move the selection (`move_objects START`, ends on release) | cb:4210-4235 |
| Ctrl+B1 drag on object | intuitive | Move stretching attached wires (XOR `enable_stretch`) | cb:4222-4226 |
| Shift+B1 drag on object | intuitive | Copy/duplicate the selection | cb:4234 |
| Shift+Ctrl+B1 drag on object | intuitive | Stretch-move with `connect_by_kissing` (adds wires where touching pins separate) | cb:4229-4232 |
| B1 press on an unselected instance pin or wire endpoint | intuitive | Start a wire there. Button release ends it unless `persistent_command` | cb:4170-4177, 4318-4322, 1792-1845 |
| B1 drag on an endpoint of a selected single wire/line, or a corner of a selected rect | intuitive | Drag only that vertex/corner (rect corner hit-box = 2 x halfdotsize) | cb:4193-4206, 1847-1949 |
| B1 drag on a vertex of a selected single polygon/bezier | any | Move that vertex | cb:4189, 2018 |
| Shift+B1 on a polygon vertex | any | Insert a new vertex after it and start moving it | cb:1979-2001 |
| Ctrl+B1 on a polygon vertex | any | Delete the vertex (if more than 2 points) | cb:2003-2016 |
| B1 click while placing/moving/copying/pasting | – | Drop at the snapped position (`end_place_move_copy_zoom`) | cb:1381-1459 |
| B1 click while drawing a wire/line | – | Commit the segment. Non-persistent: ends. Persistent: chains from this point | cb:1390-1426, 4103-4118 |
| B1 click while drawing a rect | – | Second corner, done | cb:1429 |
| B1 click while drawing a polygon | – | Add a vertex. Clicking on the first vertex closes and finishes it | actions.c:3423-3459 |
| B1 clicks while drawing an arc/circle | – | Click 2 sets the second point, click 3 sets a point on the arc (3-point arc via `arc_3_points`). Circle forces a 360 degree sweep | actions.c:3227-3258 |
| Shift+B1 while drawing a wire | – | Finish the wire snapped to the closest pin or net endpoint | cb:4061-4063, 1660-1679 |
| B1 double-click | idle | Edit properties of the object under the mouse; selection overrides `lock` | cb:4380-4400 |
| B1 double-click | drawing wire/line | End the wire/line. In cadence mode the wire restarts at the point | cb:4403-4411 |
| B1 double-click | drawing polygon | Finish the polygon (open) | cb:4412 |
| B1 click while the edit-prop dialog is open | – | OK the dialog and switch editing to the newly clicked object. Shift+B1 adds to the selection with `preserve_unchanged_attrs` (multi-edit) | cb:4076-4097 |
| B2 (middle) drag | no modifier | Pan | cb:4070-4074, 2370 |
| Space, then move | – | Pan follows the mouse until any button press | cb:3673, 4013-4016 |
| B3 drag | no modifier, idle | Zoom to box | cb:4053-4056 |
| B3 press+release, no drag | no pending op | Context menu (21 actions, `context_menu` tcl:7501, dispatched in cb:2029-2152) | cb:4264-4269 |
| B3 release with a pending wire/line/rect/polygon/arc | – | Commits the pending point first (`end_place_move_copy_zoom`), then shows the context menu | cb:4264 |
| Shift+B3 | – | Select the clicked instance/wire plus all connected wires/labels/pins (crosses junctions) | cb:4039-4045, select.c:93 |
| Ctrl+B3 | – | The same, but stops at wire junctions | cb:4020 |
| Alt+B3 | – | Cut the wire at the mouse into 2 wires; cut point snapped | cb:4028 |
| Alt+Shift+B3 | – | Cut the wire at the exact (unsnapped) mouse point | cb:4034 |
| Wheel up / down | no modifier | Zoom in / out by 1.2 around the mouse. Zoom-out keeps the mouse point fixed only if `unzoom_nodrift=1`, otherwise it re-centers. Limits 0.005 to 10000 | cb:2154-2172, actions.c:2856-2890, xschem.h:166-169 |
| Shift+wheel | `graph_use_ctrl_key=0` | Pan horizontally (100 x zoom units) | cb:2174-2189 |
| Ctrl+wheel | `graph_use_ctrl_key=0` | Pan vertically | cb:2190-2199 |
| Motion | – | Rubber-band update for wire/line/rect/poly/arc/zoom box; move/copy preview; crosshair redraw; snap-cursor redraw (when Shift held or wire drawing active) | cb:2348-2477 |
| Pointer enters another xschem window during a copy | – | Cross-window copy: the sender writes the selection file, the receiver auto-starts a merge/paste at the pointer | cb:2311-2346, 4503-4549 |
| Buttons 8 / 9 | `mouse_bindings.tcl` loaded | User-defined (default: zoom full, unhighlight all, Shift+8 save) | src/mouse_bindings.tcl |
| `auto_hilight=1` | B1 select | Clicking a net highlights it. Clicking empty space without Shift clears highlights | cb:4238-4248 |

**Graph mouse.** Applies when `waves_selected` holds. Areas: "plot" is the inner area; "left" is the Y-axis strip; "bottom" is the X-axis strip.

| Gesture | Area | Action | cb line |
|---|---|---|---|
| B1 drag | plot | Pan X. Linked graphs (same `sim_type`, not unlocked `flags&2`) and selected graphs follow | 928-976 |
| B1 drag | left | Pan Y (master graph only; digital graphs use `ypos1`/`ypos2`) | 932-958 |
| B1 drag | bottom | Scrollbar: center the current X window on the mouse position mapped over the full raw-data range | 859-891, 1279 |
| B1 press within 10 px of cursor A/B or hcursor 1/2 | plot | Drag that cursor. Moving B live-back-annotates and runs `cursor_2_hook` | 562-620, 489-540 |
| B3 drag | plot | Zoom X to the dragged span on release. Shift reverses the axis direction | 893-903, 1298-1314 |
| B3 drag | left | Zoom Y to the span. Shift inverts | 1315-1349 |
| B3 click within 10 px of a cursor | plot | Dialog to enter the cursor position numerically | 623-718 |
| B3 click on a wave label | top labels / digital names | Toggle `hilight_wave` (bold trace) | 445-448, draw.c:3647-3721 |
| B1 double-click | wave label | Wave edit dialog (`graph_edit_wave`, tcl:2856) | 720, draw.c:3689 |
| B1 double-click | elsewhere in graph | Graph properties dialog (`graph_edit_properties`, tcl:3308) | 721-723 |
| Wheel | plot | Pan X by 5% | 978-1050 |
| Shift+wheel | plot | Zoom X by 20%, anchored at the mouse x fraction | 1052-1130 |
| Wheel | left | Pan Y by one division (digital: 5% of `posh`) | 978-1000 |
| Shift+wheel | left | Zoom Y by 20% around the mouse y | 1052-1068 |
| Motion with measurement on (`m`) | plot | Tooltip with y=, x= at the mouse, in engineering units | 451-484 |

### 2.4 Drawing / editing modes and their keys

| Mode (`ui_state` flag) | Started by | Continue / modify | End | Abort |
|---|---|---|---|---|
| STARTWIRE | `w`, Tools menu, context menu, intuitive pin/endpoint drag, `W` / `s`(cad) snapped | Click adds a segment (persistent) or ends. Space cycles Manhattan. `h`/`v` constrain. `L` toggles orthogonal | Click (non-persistent), double-click, Shift+click snapped end, release (intuitive drag) | Esc |
| STARTLINE | `l` | Same as wire (Manhattan applies to lines too, actions.c:3299) | Click / double-click | Esc |
| STARTRECT | `r` | – | Second click | Esc |
| STARTPOLYGON | `p` | Click adds a vertex. `h`/`v` constrain | Click on the first vertex (closed), Return (closed), double-click (open). Esc also stores the polygon placed so far (`new_polygon(END)`, cb:150) | – |
| STARTARC | `C` (arc), Ctrl+C (circle) | – | Third click | Esc |
| STARTMOVE | `m` / `M` / Ctrl+m / Alt+m, drag (intuitive), placement of symbol/text/pin/label/paste | R / F / Alt+r / Alt+f / V / Alt+v transform in flight. `h`/`v` constrain. Space sets Manhattan for stretched wires | Click / release (drag) | Esc. Placement abort deletes the new object (cb:162-172) |
| STARTCOPY | `c` / Alt+c / Shift-drag | Same transforms | Click / release | Esc |
| STARTZOOM | `z`, B3 drag | – | Click / release | Esc |
| STARTPAN | Space, B2 | – | Any click / B2 release | – |
| STARTSELECT / DESEL_AREA | B1 drag, `D`, Alt-drag | – | Release | – |
| GRABSCREEN | Print | B1 drag | Release | – |
| MENUSTART + ui_state2 | Any command key when `infix_interface=0`, or menu commands: MENUSTARTWIRE/LINE/RECT/POLYGON/ARC/CIRCLE/MOVE/COPY/ZOOM/DESEL/SNAPWIRE/WIRECUT/WIRECUT2 | The first click supplies the first point (`check_menu_start_commands`) | – | Esc |

### 2.5 Snapping, grid and Manhattan routing

| Topic | Rule | Source |
|---|---|---|
| Snap | `cadsnap` (default 10, `CADSNAP`). Every mouse coordinate is rounded to a multiple of it. `g`/`G` halve/double it, Ctrl+g sets it, status-bar field `.statusbar.3` edits it. The field is PaleGreen at the default, OrangeRed otherwise. Snap 0 resets to the startup default | xschem.h:159, cb:4618, actions.c:294-314 |
| Grid | `cadgrid` (default 20). Display only. If grid pitch < 10 px on screen, it is multiplied by 2^n until visible (`CADGRIDTHRESHOLD`, `CADGRIDMULTIPLY`). Toggle with `%`. `big_grid_points` scales dots | xschem.h:160-162, draw.c:1104-1106, actions.c:316 |
| Dot/handle size | `cadhalfdotsize = 3.7 * min(snap,20)/10`. Used for junction dots and vertex hit-boxes | actions.c:312 |
| Align to grid | Alt+u rounds all selected (or all) coordinates to `cadsnap` | save.c:5280, cb:3448 |
| Endpoint snapping | `find_closest_net_or_symbol_pin` returns the nearest instance pin or wire endpoint within the visible area. Used by `W` / Shift+click / `s`(cad) snapped wires and by the snap cursor | findnet.c:190-260 |
| Snap cursor | `snap_cursor=1` draws a diamond (`snap_cursor_size`, default 6) at the nearest endpoint while Shift is held or a wire is being drawn. In persistent mode, clicking when the diamond coincides with the snapped mouse ends the wire | cb:1461-1544, 4106-4114 |
| Crosshair | `draw_crosshair`, `crosshair_layer` (default 8), `crosshair_size` (0 = full-screen lines, hides the pointer). With crosshair and (`use_cursor_for_selection` or size 0), selection hit-test uses snapped coordinates | cb:1546-1648, 4153-4158 |
| Intuitive wire start | Requires an exact match of the snapped mouse to the pin coordinate or wire endpoint | cb:1792-1845 |
| `manhattan_lines` | 0 = straight/oblique, 1 = horizontal then vertical, 2 = vertical then horizontal. A wire placed in mode 1/2 is stored as up to 2 orthogonal wire segments. Initial 0 per context. Space cycles it during wire/line/move | actions.c:3093-3139, draw.c:1520-1550, xinit.c:568, cb:3655-3672 |
| Stretched wires on move | With `manhattan_lines` != 0, the moved endpoint of a stretched wire becomes an L (a new wire segment is added) | move.c:1029-1150 |
| `orthogonal_wiring` | Shift+L toggles it. On every rubber update, mode 1 or 2 is chosen by the dominant axis (`recompute_orthogonal_manhattanline`). Applied per stretched wire during move. Turning it off resets to 0 | actions.c:3885-3897, cb:117-120, cb:3033-3041, move.c:1036 |
| Axis constraint | `constr_mv`: 1 = horizontal-only (y locked, key `h`), 2 = vertical-only (x locked, key `v`). Pressing again toggles off. Applies to move/copy/wire/line/polygon/arc rubbers. Reset at the end of each operation. Menu H/V entries set it | cb:2878-2897, 3475-3491, 1385-1459 |
| `autotrim_wires` | Joins/trims wires automatically after wire placement / align | actions.c:3141, cb:3452 |

### 2.6 Interaction-mode options (Tcl variables, set in xschemrc or the Options menu)

| Variable | Default | Effect | Source |
|---|---|---|---|
| `infix_interface` | 1 | Command key uses the current pointer as the first point. 0 = key arms the command and the first click gives the first point | tcl:9898, cb:1681-1790 |
| `persistent_command` | 0 | Wire/line tool stays active. Each click commits a segment and continues. The first Esc ends the chain but keeps the tool armed (next click starts a new wire); the second Esc exits. With `cadence_compat`, a single Esc also disarms the wire tool (cb:3788) | tcl:9893, cb:144-158, 4100-4118, 3785 |
| `intuitive_interface` | 1 | Click-drag moves objects. Drag from pin or wire end draws a wire. Drag on a selected shape's vertex edits it. Shift-drag copies. Selection is not cleared when clicking an already-selected object | tcl:9894, cb:4147-4235 |
| `enable_stretch` | 0 | Swaps `m` <-> Ctrl+m and drag <-> Ctrl+drag (stretch). Toggled with `y` | tcl:9877, cb:3058, 4222 |
| `cadence_compat` | 0 | `s` = snapped wire, Ctrl+r = simulate, Alt+z = snap cursor, `W` disabled. Double-click restarts a wire. B1 release on an item in a multi-selection selects only that item. `src/cadence_style_rc` sets crosshair 1 (size 2), infix 0, orthogonal 1, snap_cursor 1 (size 4), persistent 1, use_cursor_for_selection 1 | tcl:9897, cb:4294-4316, src/cadence_style_rc:1-10 |
| `select_touch` | 1 | Right-to-left drag selects touching objects | tcl:9887, actions.c:3873 |
| `incremental_select` | 1 | Live selection while dragging | tcl:9886 |
| `unselect_partial_sel_wires` | 0 | Unselect stretched wires after a move | tcl:9879 |
| `use_cursor_for_selection` | 0 | Hit-test at the crosshair (snapped) point | tcl:9895 |
| `graph_use_ctrl_key` | 0 | Graph events need Ctrl. Also disables Shift/Ctrl+wheel panning | tcl:9848, cb:2175 |
| `live_cursor2_backannotate` | 1 | Cursor B drives schematic back-annotation | tcl:9872 |
| `auto_hilight` | 0 | Select = highlight net | tcl:9917 |
| `unzoom_nodrift` | 0 | Zoom-out anchored at the mouse | tcl:9868 |
| `zoom_full_center` | 0 | `f` centers the drawing | tcl:9869 |
| `change_lw` | 1 | Line width scales with zoom | tcl:9870 |
| `draw_grid` / `big_grid_points` | 1 / 0 | Grid display | tcl:9889-9890 |
| `snap_cursor` / `snap_cursor_size` | 0 / 6 | Endpoint diamond cursor | tcl:9899, 9863 |
| `draw_crosshair` / `crosshair_size` / `crosshair_layer` | 0 / 0 / 8 | Crosshair | tcl:9860-9862 |
| `orthogonal_wiring` | 0 | Auto L-routing | tcl:9900 |
| `autotrim_wires` | 0 | Auto join/trim | tcl:9896 |
| `tabbed_interface` | 1 | Tabs vs. separate windows (Ctrl+Left/Right switch tabs) | tcl:9947 |
| `new_symbol_browser` | 0 | Insert/Ctrl+i use the new `insert_symbol` browser | tcl:9952 |
| `autofocus_mainwindow` | 0 | Focus follows mouse | tcl:9796 |
| `fix_mouse_coord` | 0 | Re-query pointer on key events | tcl:10018 |

## 3. Dialogs

All dialogs are Tk toplevels built in `src/xschem.tcl` (paths below are `xschem.tcl:LINE` unless noted). 33 procs build a toplevel (`grep 'toplevel ' src/xschem.tcl`); several more prompts use native Tk dialogs (`tk_getOpenFile`, `tk_getSaveFile`, `tk_chooseDirectory`, `tk_chooseColor`, `tk_messageBox`).

Common conventions (apply to every dialog unless noted):

| Convention | Detail |
|---|---|
| Modal-ish lock | Most dialogs do `xschem set semaphore [+1]` while open; C callbacks ignore most canvas commands when `semaphore >= 2` (e.g. `callback.c:2913`). |
| Result passing | Results go through Tcl globals: `retval` (text), `tctx::rcode` (`ok`/`yes`/`no`/empty = cancel), `props`, `tctx::hsize`, `tctx::vsize`, `symbol`. C reads them back with `tclgetvar`. |
| Shared window name | `.dialog` is reused by edit_prop, text_line, enter_text, input_line, ask_save, property_search, attach_labels_to_inst — only one can be open (`if {[winfo exists .dialog]} return`). |
| Placement | Opened at pointer (`winfo pointerx - 60`, `pointery - 35`); size remembered in globals (`edit_prop_size`, `text_line_default_geometry`, `enter_text_default_geometry`, `file_dialog_default_geometry`, `graph_dialog_default_geometry`, `insert_symbol(geometry)`). |
| Keys | `Escape` = Cancel (edit dialogs only cancel if text unchanged), `Return` = OK on single-line dialogs, `Shift+Return` in multi-line text = OK (`return_release` deletes the typed newline, `xschem.tcl:5694`). |
| Click-through | While an edit dialog is open, left-click on canvas invokes its OK (`callback.c:4076-4090`): text_line/enter_text commit; edit_prop commits and re-opens on the newly clicked instance (`edit_symbol_prop_new_sel=1`, loop in `editprop.c:2046`); Shift+click adds instance to the edit set and forces "preserve unchanged". |
| Edit modes | C `edit_property(x)` (`editprop.c:1922`): `x=0` Tk dialog, `x=1` external `$editor` (edit_vi_prop), `x=2` read-only viewdata. Keys: `q`=0, `Shift+Q`=1, `Ctrl+Shift+Q`=2, double-click=0 (`callback.c:3262-3289, 4401`). |

### 3.1 Property / text editing

#### edit_prop — "Edit Properties" (instance attributes) — `xschem.tcl:6275`
Invoked: `q`, double-click instance, menu Properties→Edit, context menu "Edit attributes", `xschem edit_prop` → `edit_symbol_property()` `editprop.c:1727`. Input: `retval` = instance prop string, `symbol` = instance symbol ref.

| Widget | Type | Effect |
|---|---|---|
| Label `txtlabel` + "Path:" | readonly entry | Absolute directory of the symbol (`editprop_sympath`) |
| Symbol | entry | Symbol reference; changing it replaces the instance's symbol (`update_symbol`, `editprop.c:1560`). New prefix char from new symbol `template name=` is applied to instance name and name re-uniquified (`editprop.c:1676-1697`) |
| Browse | button | `tk_getOpenFile` in `INITIALINSTDIR`, fills Symbol entry |
| OK | button | Commit; if Edit Attr ≠ `<ALL>`, value re-quoted and spliced back via `xschem subst_tok` |
| Cancel | button | Discard (also WM close) |
| Load | button | `tk_getOpenFile`, insert file contents at cursor (`read_data_window`) |
| Del | button | Clear text area |
| No change properties | checkbutton `no_change_attrs` | Only change symbol, keep prop string |
| Preserve unchanged props | checkbutton `preserve_unchanged_attrs` | Multi-selection: apply only tokens that differ from the original (`set_different_token`); auto-set to 1 when >1 object selected (`editprop.c:2030`) |
| Copy cell | checkbutton `copy_cell` | When symbol name changed: copy old `.sym` (and `.sch`) to the new name if not existing (`xschem.tcl:6330-6353`) |
| Edit Attr: | ttk::combobox | `<ALL>` or one token from `list_tokens $retval`; selecting a token shows only its value (unquoted, `xschem get_tok ... 2`) in the text area; switching tokens writes back edits |
| (main) | text, undo, x/y scroll, no wrap | Full attribute string `key=value ...`. On open, value after `value=` (else `lab=`) is pre-selected and cursor placed (`xschem.tcl:6475-6493`) |

Notes: `netlist_commands` symbols in external-editor mode edit only the `value=` token, unescaping `\"`/`\\` (`edit_vi_netlist_prop` `xschem.tcl:6180`, `editprop.c:1740`). `drc_check()` runs after update if a Tcl `fet_drc` proc exists (`editprop.c:1510`).

#### text_line — "Text input" (generic multi-line attribute editor) — `xschem.tcl:6573`
Invoked by: rect/line/wire/arc/polygon property edit (`editprop.c:1025,1112,1168,1233,1308`), global schematic/symbol property edit when nothing is selected (`editprop.c:1965`, label "Global schematic property:", edits the property block for the current netlist mode — `K` symbol attrs in symbol view, `G` vhdl, `V` verilog, `S` spice, `E` tedax), header/license editor `Shift+B` → `update_schematic_header` (`xschem.tcl:1404`, edits the file header text).

| Widget | Type | Effect |
|---|---|---|
| preserve unchanged props | checkbutton (enabled only when caller passes `normal`) | As in edit_prop |
| OK / Cancel / Load / Del | buttons | Commit / cancel / insert file / clear |
| Edit Attr: | ttk::combobox | Per-token editing as in edit_prop |
| (main) | text 90x40, undo | Attribute string |

Post-edit parsing per object (`editprop.c`): rect → `dash`, `fill`, `ellipse`, plus flags/image extra data (`set_rect_flags`, `set_rect_extraptr`); line → `bus`, `dash`; wire → `bus`; arc → `dash`, `fill`; polygon → `bezier`, `bus`, `dash`, `fill`.

#### enter_text — "Enter text" (text object create/edit) — `xschem.tcl:5554`
Invoked: place text (`t` key / menu / context "Insert text", `place_text()` `actions.c:3729`) and editing a selected text (`edit_text_property` `editprop.c:1373`).

| Widget | Type | Effect |
|---|---|---|
| preserve unchanged props | checkbutton | Multi-select merge |
| (txt) | text 120x9, undo | Text string (`T {...}` content); may contain `@attr` / `tcleval()` |
| hsize: | entry | X scale (`tctx::hsize`). **Cairo builds bind this entry to `vsize`** and on OK copy vsize→hsize (single size), `xschem.tcl:5590-5596,5620` |
| vsize: | entry | Y scale (`tctx::vsize`), default 0.4 |
| props: | text 70x3 | Text attributes. Recognised keys (`set_text_flags` `actions.c:895`): `font`, `hcenter`, `vcenter`, `layer`, `slant` (italic/oblique), `weight` (bold), `hide`, `name`, `floater` |
| OK / Cancel / Load / Del | buttons | Commit / cancel (empty `retval`) / insert file / clear |

Side effect: if the text string changes and the text sits next to a symbol pin (within cadgrid tolerance) the pin's `name=` is renamed too (`editprop.c:1452-1478`). Empty text on placement = no object created (`actions.c:3750`).

#### edit_vi_prop / edit_vi_netlist_prop / edit_file — external editor (no toplevel)
| Proc | Line | Behaviour |
|---|---|---|
| edit_vi_prop | `xschem.tcl:6153` | Writes `retval` to `$XSCHEM_TMP_DIR/.xschem_edit_file.<pid>.{v,vhd}`, runs `$editor` with `execute_wait`, reads back; `rcode=ok` if changed. `Shift+Q`, menu "Edit with editor", context "Edit attr in editor". |
| edit_vi_netlist_prop | `xschem.tcl:6180` | Same for `netlist_commands` instance `value`, adds `-c :set filetype=<netlist_type>` for vim. |
| edit_file | `xschem.tcl:1148` | `Alt+Q`: opens the raw `.sch` (nothing selected; the `xschem edit_file` command saves first, the key does not) or selected instance's `.sym` in `$editor` (`scheduler.c:933`, `callback.c:3266`). |

#### input_line — "Input number" (single-line prompt) — `xschem.tcl:7355`
Widgets: label `txt`, entry (preset, pre-selected, width `w`), OK (evaluates optional `cmd <value>` then sets `retval`), Cancel. Return/Escape bound; `grab set`.

Callers:
| Prompt | Source | Effect |
|---|---|---|
| Object Sequence number | `editprop.c:1789` (`Shift+S`, `change_elem_order`) | Swap selected object's index with N (instance/rect/wire/text array order) |
| input instance number (leftmost = 1) to descend into… | `actions.c:2399` | Descend into one element of a vector instance `x[3:0]`; negative counts from right |
| Pos: | `callback.c:642,672,697,712` | Double-click on a graph cursor (a/b vertical, A/B horizontal) to type exact position (eng. notation) |
| Enter snap value | `callback.c:2825` (`Ctrl+G`), menu "Set snap value" `xschem.tcl:9093` | `xschem set cadsnap` |
| Enter grid spacing | menu "Set grid spacing" `xschem.tcl:9097` | `xschem set cadgrid` |
| Enter linewidth | `Alt+-` `callback.c:3768`, menu "Set line width" `xschem.tcl:9117` | `xschem line_width` |
| Enter Grid point size | menu `xschem.tcl:9121` | `grid_point_size` |
| Enter crosshair size | menu Options "Crosshair size" `xschem.tcl:8937` | `crosshair_size` (0 = full screen) |
| Enter two characters to replace [] | menu Options `xschem.tcl:8942` | `bus_replacement_char` for SPICE netlists |
| Enter Symbol width | menu Symbol "Set symbol width" `xschem.tcl:9182` | `symbol_width` used by make_symbol |
| Set netlist file name | menu Simulation `xschem.tcl:9313` | `xschem set netlist_name` |

#### viewdata / editdata / textwindow — text viewers
| Proc | Line | Window | Widgets / behaviour |
|---|---|---|---|
| viewdata data ?ro? ?win? | 6934 | `.viewN` "View data" | Text (no wrap, scroll), Dismiss, Save As (`tk_getSaveFile`, omitted when `ro`). Non-blocking. Used for `Ctrl+Shift+Q` view props, job completion status, last job data/errors, `j` list of highlighted nets, changelog, simconf help. |
| editdata data ?title? | 6848 | `.editdata` | Text, OK (returns edited text), Cancel, Copy (to clipboard). Blocking. Used for editing `spice_sym_def` from cellview/traversal. |
| textwindow file ?ro? | 6887 | `.winN` title=filename | Loads file, Dismiss, Save (writes back, omitted when `ro`). Used for netlist display when "Show netlist after netlist command" is on (`proc netlist` `xschem.tcl:972`), `?` help (`xschem.help`, `callback.c:3941`), generator source. |
| infowindow | 6803 | `.infotext` "Info window" | Always exists (withdrawn); text with ERC/netlist messages from `xschem get infowindow_text`, Dismiss/Escape withdraw. Shown per `show_infowindow_after_netlist` (`always`/`onerror`, `show_infotext` 6786) or menu View→Show ERC Info window. |

### 3.2 File / symbol selection

#### load_file_dialog — custom file browser — `xschem.tcl:4494`
Signature: `load_file_dialog msg ext global_initdir loadfile confirm_overwrt initialf`. `loadfile`: 1 load (blocking), 0 save (via `save_file_dialog` `xschem.tcl:3941`), 2 non-blocking symbol insertion.
Callers: Open (`Ctrl+O`, `actions.c:616`, filter `*.{sch,sym,tcl}`), place symbol "Choose symbol" (`actions.c:1488`), Insert symbol non-blocking (`Ctrl+I`, `Shift+Insert`, menu File→Component browser when `new_symbol_browser=0`), Merge file (`paste.c:345`), "Schematic to compare with" (`xinit.c:754`), Save as (`actions.c:588`), export PNG/SVG/PS/PDF destination (`draw.c:110`, `svgdraw.c:910`, `psprint.c:1611`), make schematic+symbol from selection (`save.c:5004`).

| Widget | Type | Effect |
|---|---|---|
| Recent (only `loadfile=2`) | button column (`c_toolbar`, `xschem.tcl:4075`) | Up to 25 recently placed symbols; click = preview + `xschem place_symbol` |
| Left pane | listbox | Library search paths (`pathlist` = XSCHEM_LIBRARY_PATH) + current dir; colored by `dircolor` regex array; right-justified |
| Right pane top | listbox (browse or extended select) | Dirs (`..`, `.`, subdirs) then files matching glob filter; extended select in load mode opens extra files in new windows (`load_additional_files`) |
| Right pane bottom | canvas | Live preview via `xschem preview_window draw` (renders .sch/.sym) |
| Home | button (`H` key) | Reset left pane to library paths |
| Up | button (`U` key) | Parent directory |
| Current dir | button | Directory of current schematic |
| Library paths | checkbutton `load_file_dialog_fullpath` | Show full paths vs tails in left pane |
| New dir: + Create / Delete | entry + buttons | mkdir / delete entry in current dir |
| Search: | entry | Glob filter `*text*` live on KeyRelease |
| File: | entry | Selected filename (absolute, relative, or `http(s)://` URL accepted) |
| OK / Cancel | buttons | Return / abort (Cancel in mode 2 also `xschem abort_operation`) |

Behaviour: Return/double-click accepts a non-directory; in save mode existing file → `alert_ "Overwrite …?"` yes/no; non-xschem file → "does not seem to be an xschem file… Continue?"; `.sym` filter + schematic chosen → "does not seem to be a SYMBOL file"; generators (executable `.tcl`/script symbols) accepted (`file_dialog_getresult` `xschem.tcl:4310`). Mode 2: leaving the dialog window with a selection starts placement immediately (`<Leave>` → `file_dialog_place_symbol` 4374). Sash positions and scroll position persisted in globals.

#### insert_symbol — alternate component browser — `xschem.tcl:5035`
Enabled when `new_symbol_browser=1` (for `Insert`, `Shift+I`, `Ctrl+I`, `Shift+Insert`, `callback.c:2919-2936,3882-3890`) or `new_file_browser=1` for `Ctrl+O` (action `load`, `callback.c:3186`). Config: `new_symbol_browser_paths` (empty = library paths), `_depth` (default 2), `_ext` (regex, default `\.(sch|sym|tcl)$`) (`xschem.tcl:9952-9961`).

| Widget | Type | Effect |
|---|---|---|
| Pattern: | entry `insert_symbol(regex)` | Regex filter on file names |
| Symbol ref: | readonly entry | Path relative to library (`rel_sym_path`) — what gets stored in `C {...}` |
| Update | button | Rescan dirs (`get_list_of_dirs_with_files`) and file list |
| Ext: | entry | Extension regex |
| Full path: | readonly entry | Absolute path of selection / dir |
| Left | listbox | Directories (tails, sorted) containing matching files |
| Middle | listbox | Matching files (sorted dictionary) |
| Right | frame | Preview (`xschem preview_window`) |
| Dismiss | button | Close (or abort ongoing placement) |
| Load file | button | `xschem load -gui <file>` |
| Place symbol | checkbutton (`action` symbol/load) | When on, selecting a file starts placement immediately (`insert_symbol_draw_preview` 4864) |
| N. of items | label | Count |
Keys: Tab moves between lists, Return closes, Escape dismisses. Non-blocking (persistent).

#### Native Tk dialogs
| Dialog | Where | Purpose |
|---|---|---|
| tk_getOpenFile | edit_prop Browse/Load, text_line/enter_text Load, `select_raw` (`xschem.tcl:8724`, filter `.raw`), `xschem add_image` (`scheduler.c:280`, filter jpg/jpeg/png/svg → rect with `flags=image,unscaled alpha=0.8 image=<file>`), delete_files | File pick |
| tk_getSaveFile | viewdata Save As | |
| tk_chooseDirectory | `set_netlist_dir 1` (`xschem.tcl:5523`, "Select netlist DIR"; menu Simulation→Set netlist Dir) | Netlist/simulation dir |
| tk_chooseColor | `change_color` (`xschem.tcl:6238`; View→Change current layer color) | Recolor current layer; saved to `~/.xschem/colors` |

### 3.3 Simulation / processes

#### simconf — "Simulation Configuration" — `xschem.tcl:1704`
Menu Simulation→Configure simulators and tools. Data in `sim(...)` array, defaults `set_sim_defaults` (`xschem.tcl:1537`), persisted to `~/.xschem/simrc`.

Layout: scrollable frame, one band per tool in `sim(tool_list)` = `spice spicewave verilog verilogwave vhdl vhdlwave`; one row per configured command `i < sim(<tool>,n)`:

| Per-row widget | Variable | Effect |
|---|---|---|
| Name | entry `sim(tool,i,name)` | Display name |
| Default | radiobutton `sim(tool,default)=i` | Which row Simulate/Waves uses |
| Command | text 3 lines `sim(tool,i,cmd)` | Shell command; Tcl `subst` with `$N` netlist path, `$n` netlist w/o ext, `$S` schematic full path, `$s` schematic name (or netlist_name), `$d` sim dir, `$terminal` |
| Fg | checkbutton `sim(tool,i,fg)` | Run in foreground (wait) |
| Status | checkbutton `sim(tool,i,st)` | Show completion report (viewdata with stdout/stderr/exit) |

Bottom buttons: Cancel, Help (viewdata text), Accept Save and Close (writes simrc), Reset to default (deletes/reset after `tk_messageBox` confirm, `simconf_reset` 1675), Accept no Save and Close.

Default rows (`xschem.tcl:1566-1663`):
| Tool | Rows (name: cmd) |
|---|---|
| spice | Ngspice interactive: `$terminal -e {ngspice -i "$N" -a \|\| sh}`; Ngspice Control mode: `ngspice "$N" -a`; Ngspice batch: `ngspice -b -r "$n.raw" "$N"`; Xyce batch: `Xyce "$N"`; Xyce parallel batch: `mpirun /path/to/parallel/Xyce "$N"` |
| spicewave | Gaw viewer: `gaw "$n.raw"`; Ngspice Viewer: `$terminal -e ngspice`; Rawtovcd: `rawtovcd -v 1.5 "$n.raw" > "$n.vcd" && gtkwave …`; Bespice wave: `bspwave --socket localhost $bespice_listen_port "$n.raw"` |
| verilog | Icarus verilog: `iverilog -o .verilog_object -g2012 '$N' && vvp .verilog_object`; Verilator |
| verilogwave | Gtkwave: `gtkwave dumpfile.vcd "$N.sav"` |
| vhdl | Ghdl: `ghdl -c --ieee=synopsys -fexplicit "$N" -r "$s" "--wave=$n.ghw"` |
| vhdlwave | Gtkwave: `gtkwave "$n.ghw" "$N.sav"` |

#### list_running_cmds — "List of running commands" — `xschem.tcl:510`
Menu Simulation→List running sub-processes (disabled on Windows). Listbox of `id pid cmd` for every `execute` pipe (extended select, refreshed after spawn/exit). Buttons: Terminate selected (SIGTERM, `kill_running_cmds` 401), Kill selected (SIGKILL), View status (live-updating `.pstat` viewdata of process output, `view_process_status` 472; double-click same), Dismiss. Title auto-refreshes every 1 s.

Related menu items (viewdata): "View last job data" (`execute(data,last)`), "View last job errors" (`execute(error,last)`) `xschem.tcl:9336-9345`. Job completion report when status flag set: "Completed:/Failed: cmd, stderr, data" (`xschem.tcl:286-293`).

#### inutile — "(IN)UTILE" stimuli editor — `xschem.tcl:144`
Menu Simulation→Utile Stimuli Editor (GUI) on `<dir>/stimuli.<schname>`. Text editor + buttons: Dismiss, Help (`inutile_help_window` 111, shows `utile/utile.txt`, Save), Reload / Save (ask filename via `inutile_line` 24 entry dialog), Translate (runs awk chain preprocess→expand_alias→param→clock→stimuli in netlist dir, `inutile_translate`), Template (load `utile/template.stimuli` if empty), "time:" entry (filled from `inutile.simulationtime`). `include`/`.include` lines open `inutile_alias_window` (86: Dismiss/Save/Reload). Legacy feature; low priority for reimplementation.

### 3.4 Graph (waveform) dialogs

#### graph_edit_properties — graph configuration — `xschem.tcl:3308`
Invoked: double-click inside a graph rect not on a wave label (`callback.c:720`). Edits attributes of rect on layer 2 (GRIDLAYER) with `flags=graph`; most widgets apply live (`xschem setprop rect 2 n <attr>` + `xschem draw_graph`), first change pushes one undo (`graph_push_undo`).

| Row | Widget | Attribute / effect |
|---|---|---|
| top | Legend (check) | `legend` (default 1) |
| top | Incr. sort (check) | `graph_sort`: listbox sort order |
| top | Bus (check) | Add-mode: selected nodes join into `BUS_NAME;a,b,c` line |
| top | Priv. Cursor (check) | `flags` += `,private_cursor` (per-graph cursor positions `cursor1_x`…) |
| top | Digital (check) | `digital` (logic-level display; buses shown) |
| top | Unlock. X axis (check) | `flags` += `,unlocked` (X not synced with other graphs) |
| top | Rainbow col. (check) | `rainbow` (color sweeps/datasets) |
| top | Line width (entry) | `linewidth_mult` (deleted if equals global `graph_linewidth_mult`) |
| top | Dataset (entry) | `dataset` (single sweep/dataset index to plot; empty = all, `draw.c:3261`) |
| top2 | X units / Y units (spinbox f p n u m 1 k M G T) | `unitx`, `unity` |
| top2 | X div. / Y div. (entry) | `divx`, `divy` (default 5, min 1) |
| top2 | X subdiv. / Y subdiv. | `subdivx`, `subdivy` (default 1; Log toggles set 8 / 4) |
| top2 | Mode (combobox Line HistoV HistoH) | `mode` |
| top2 | Sweep (entry) | `sweep` (alternate X variable(s)); applied on `<Leave>` |
| top3 | Log X / Log Y (check) | `logx`/`logy`, also resets subdiv and full-zooms |
| top3 | X min / X max / Y min / Y max (entries) | `x1 x2 y1 y2` |
| top3 | X/Y lab mag (2 entries) | `xlabmag`, `ylabmag` |
| left | Search (entry, regex) + Add (button) | Filter / add selected signals |
| left | listbox (extended) | Signals from the graph's raw file (`v(x)` shown as `x`); double-click adds |
| right | Auto load (check) | `autoload`: read the raw file on draw |
| right | Sim type (combobox dc ac tran op sp spectrum noise constants table) | `sim_type` |
| right | Raw file: (button + entry) | `rawfile` (button opens `select_raw`; `$netlist_dir` substituted back as literal) |
| right | text | `node` attribute: one wave per line; color-tagged per wave; expressions `"alias;operand operand op"` (RPN), buses `NAME;b1,b2`, `node%n` dataset select (see `doc/xschem_man/graphs.html`) |
| bottom | OK / Apply / Cancel | Commit node list + ranges + flags / same without closing / close |
| bottom | color radiobuttons layers 4..cadlayers-1 | Recolor wave(s) under text selection or insertion cursor (`color` attr list) |
| bottom | AUTO SET | `color="4 5 6 … 21"` |
Keys: `Ctrl+Return` OK, Escape Cancel. Dialog stays bound to the schematic it was opened on (`graph_schname`); edits ignored if another schematic is active. While open, `Alt+G`/clicking nets adds nodes (`graph_add_nodes_from_list` 2922).

#### graph_edit_wave — wave color picker — `xschem.tcl:2856`
Invoked: double-click on a wave label in a graph (`draw.c:3693,3720`). Row of color radiobuttons (layers 4..cadlayers-1) changing that wave's entry in `color`, OK, Cancel. Missing colors padded with default 4.

#### graph_show_measure — hover tooltip — `xschem.tcl:3875`
Override-redirect toplevel `.measure` (black bg, yellow text) shown 400 ms after hover with `measure_text` ("y=… x=…"), stopped on `<Leave>`.

### 3.5 Search / labels

#### property_search — "Search" — `xschem.tcl:5896`
Invoked: `Ctrl+F`, menu Tools→Search, toolbar Search. Re-opens until something is found or Cancel.

| Widget | Type | Effect |
|---|---|---|
| Token | entry `custom_token` (default `lab`) | Attribute name to match. Special: `propstring` (whole prop string), `cell::name` (symbol ref), `cell::propstring`, `cell::<attr>` (symbol global attr) (`hilight.c:719-790`) |
| Value | entry `search_value` | Value to match |
| OK | button | `xschem searchmenu exact|regex <select> <token> <value> <case>` |
| Exact search | check `search_exact` | Exact vs regex |
| Match case | check `search_case` | |
| Highlight / Select / Unselect | radiobuttons `search_select` 0/1/-1 | Highlight matching nets/instances, add to selection, or remove from selection |
| Cancel | button | |
Searches instances, wires, lines, rects, arcs, polygons (`hilight.c:719-970`; non-instance objects only in Select/Unselect mode); Highlight on wires also highlights the net. No hits → `tk_messageBox "Not found."` (`scheduler.c:5030`).

#### attach_labels_to_inst — "Add labels to instances" — `xschem.tcl:6032`
Invoked: `Shift+H` (`attach_labels_to_inst(1)` `actions.c:1273`). Places a net label on every unconnected pin of selected instances, named after the pin.

| Widget | Variable | Effect |
|---|---|---|
| Prefix | entry `custom_label_prefix` (preset `<instname>_`, `actions.c:1307`) | Prefix for label names |
| use prefix | check `use_label_prefix` | Apply prefix |
| use wire labels | check `use_lab_wire` | `lab_wire.sym` instead of `lab_pin.sym` |
| Do all | check `do_all_inst` | Apply same settings to all remaining selected instances without re-prompting (`actions.c:1311`) |
| Rotated Text | entry `rotated_text` | Force text orientation |
| OK / Cancel | buttons | `rcode=yes` / cancel |

Related (no dialog): `Alt+J`/`Alt+Shift+J` labels from highlighted nets (`add_lab_no_prefix`/`add_lab_prefix`), `Ctrl+J`-style `create_pins` (`xschem.tcl:5235`, highlighted nets → ipin/opin/iopin instances merged from `.clipboard.sch`), `j` → viewdata list of highlighted nets (`hilight.c:2250-2338`).

### 3.6 View / layers / colors

| Proc | Line | Invoked | Widgets |
|---|---|---|---|
| select_layers ("Visible layers") | 5753 | View→Visible layers | One checkbutton per layer (`enable_layer(i)`), background = layer color, labels `NN-PIN/WIRE/TEXT/BG/GRID`, 10 per column; toggle → `xschem enable_layers; redraw`; OK |
| color_dim ("Dim colors") | 5822 | View→Dim colors | Scale "Dim factor" -5..5 step 0.1 (live `xschem color_dim`), check "Dim background" `enable_dim_bg`, OK |
| change_color | 6238 | View→Change current layer color | tk_chooseColor for current layer (`xschem get rectcolor`); writes `~/.xschem/colors` |
| reset_colors | 6211 | View→Reset all colors to default | tk_messageBox confirm, deletes colors file |
| show_bindkeys (`.bk`) | 8535 | `/` key, Help→Show Keybindings | Full-window preview of `bindkeys_cheatsheet.sym`; any key/click closes |

### 3.7 Confirmations / misc

#### ask_save — "Ask Save" — `xschem.tcl:6100`
Label + Yes / Cancel (optional) / No; returns `yes`/``/`no`; grabs. `ask_save_optional` (6090) short-circuits to `yes` when `no_ask_save=1`. Callers: save on close/go-up with unsaved changes (`actions.c:565,2525`), "Schematic file … Has been changed since opening. Save anyway?" (`save.c:3417`, mtime check), "Create schematic file … already exists, overwritten" (`Ctrl+L` make schematic from symbol, `save.c:5074`), "Schematic … does not exist. Descend into base schematic?" (`actions.c:2216`).

#### alert_ — "Alert" — `xschem.tcl:6730`
`alert_ text ?pos? ?nowait? ?yesno?`: bold label, OK (or Yes/No → 1/0). Return/Escape close; keeps itself above the canvas. Used for overwrite confirm, non-xschem file warnings, "simulator for $tool is not configured", "Are you sure you want to reload?" (toolbar Reload, File→Reload), netlist not generated, overflow warnings from C.

#### tk_messageBox prompts (C side)
| Message | Source |
|---|---|
| Warning: <file> already open. | `actions.c:626`, `xinit.c:1603,1703`, `scheduler.c:2990,3067` |
| do you want to make symbol view ? (`a` key) | `callback.c:2549`, `scheduler.c:3266`; symbol exists → "Overwrite?" `xschem.tcl:5434` |
| do you want to make symbol view for %s ? | `save.c:5020` |
| Please Set netlisting mode (Options menu) | `callback.c:3153`, `scheduler.c:3473` |
| Run circuit simulation? | `callback.c:3312,3355` |
| Are you sure you want to reload from disk? (`Alt+S`) | `callback.c:3382` |
| Cannot overwrite current schematic | `save.c:5009` |
| <cell>: UNSAVED data: want to exit? | `xinit.c:1804,1877,1943,1997`, `scheduler.c:1048-1091` |
| Ok to keep undo in memory / save undo on disk? Undo will be reset. | `switch_undo` `xschem.tcl:8700` (Options undo type) |
| Swapping with same schematic name on disk… | `swap_compare_schematics` `xschem.tcl:7328` |
| Can not execute '<cmd>' | `execute` `xschem.tcl:371` |
| Warning: delete simulation configuration / 'colors' file? | `xschem.tcl:1678, 6216` |
| Tcp socket error | `xschem.tcl:2728,2750` (bespice/gaw sockets) |

#### tclcmd — Tcl console — `xschem.tcl:5716`
Menu Tools→Execute TCL command. Paned: input text (3 lines, `tclcmd_txt` persisted) / output text (scroll). Buttons Evaluate (`Shift+Return`; evaluates at global level with `puts` redirected into the output pane, `redef_puts` 5661), Close, Clear.

#### about — "About XSCHEM" — `xschem.tcl:5843`
Version label, description, 4 link buttons (repo.hu, GitHub, online manual, local manual via `xdg-open`/`start`), copyright, Close.

#### cellview (`.cv`) — `xschem.tcl:1971` and traversal (`.trav`) — `xschem.tcl:2179`
Tcl-only commands (no menu/key binding; reachable via tclcmd/scripts).
| Proc | Columns | Per-row buttons | Effect |
|---|---|---|---|
| cellview ?derived_symbols? | SYMBOL, SCHEMATIC (editable entry) for every `subcircuit` symbol in hierarchy (netlist pass with `-keep_symbols`) | Sch (open schematic in new window / edit `spice_sym_def` / show generator), Sym (open symbol) | Editing the schematic entry rewrites symbol's `schematic=` attr and saves the `.sym`; colors: missing file (red bg), non-default schematic (green), spice_sym_def (green fg), derived symbol (orange fg). Bottom: Update, status line. |
| traversal ?only_subckts? ?all_hierarchy? | INSTANCE (indented by level), SYMBOL, SCHEMATIC | Sym, Sch, Upd | Recursive walk (`hier_traversal` 2239, descends with `xschem descend`); Upd writes/removes instance `schematic=` attr in parent and saves (`traversal_setlabels` 2119). |
Both skip libraries matching `nolist_libs`; hover shows balloon with resolved path / `spice_sym_def` excerpt.

#### context_menu / tab_context_menu — popups (`.ctxmenu`, override-redirect)
| Proc | Line | Items |
|---|---|---|
| context_menu (right-button release, `callback.c:2035`) | 7501 | No selection: Open most recent, Edit attributes, Edit attr in editor, Go to upper level, Insert symbol/wire/line/box/polygon/arc/circle/text, Paste clipboard, Abort command. With selection: Edit attributes, Edit attr in editor, Descend schematic, Descend symbol, Delete/Cut/Copy/Move/Duplicate selection, Rotate/Flip selection, Rotate/Flip in-place sel., Abort command. Returns code → `callback.c:2039-2120`. |
| tab_context_menu (right-click on tab button) | 7753 | Copy filename, Open directory, Open circuit dir. term., Open sim. dir. term., Open symbol/Open schematic (counterpart), Edit netlist (non-symbol), Save, Close tab (`tab_ctx_cmd` 7645) |

#### balloon / balloon_show — tooltips — `xschem.tcl:7470,7479`
1 s hover tooltip toplevel used by toolbar buttons and cellview/traversal entries.

### 3.8 Dialog inventory count

| Category | Toplevel procs |
|---|---|
| Property/text | edit_prop, text_line, enter_text, input_line, viewdata, editdata, textwindow, infowindow |
| File/symbol | load_file_dialog, insert_symbol |
| Simulation | simconf, list_running_cmds, inutile, inutile_line, inutile_alias_window, inutile_help_window |
| Graph | graph_edit_properties, graph_edit_wave, graph_show_measure |
| Search/labels | property_search, attach_labels_to_inst |
| View | select_layers, color_dim, show_bindkeys |
| Misc | ask_save, alert_, tclcmd, about, cellview, traversal, context_menu, tab_context_menu, balloon_show |
| **Total** | **33** toplevel-building procs (+ 4 native Tk chooser types, ~20 tk_messageBox prompts, external-editor flow) |

Not dialogs in XSchem (commands only, no window): join/trim wires, break wires, make symbol (`a` → awk `make_sym.awk`, `xschem.tcl:5429`), replace symbol (done via edit_prop Symbol entry), toolbar visibility (Options checkbutton).

## 4. Core features

Split into 4A (objects, hierarchy, netlisting), 4B (simulation, waveforms, export, editing infrastructure) and 4C (scripting API and configuration).

## 4A. Core features: objects, hierarchy, netlisting

Source root `S=/nix/store/rhvgzl9mjl58nr0qm6l98fxbiadg3q98-source`. Paths below are relative to `S/src` unless noted.

### 4A.1 Drawing primitives

Coordinates are doubles; screen y grows downward. Every object carries a free-form property string (`{...}` in the file) parsed by `get_tok_value()` (token.c:438).

| Object | File rec | Geometry | Property-driven behaviour | Load code |
|---|---|---|---|---|
| Line | `L layer x1 y1 x2 y2 {p}` | endpoints are normalised by `ORDER()` | `dash=n` sets the dash length in pixels (n>=0). `bus=<bool/int>` != 0 draws thick, width `4*lw` (`INT_BUS_WIDTH`, xschem.h:327,429) | save.c:3022-3053 |
| Rect | `B layer x1 y1 x2 y2 {p}` | normalised by `RECTORDER` | `fill`: default **stippled**; `false` gives no fill, `full` gives solid. `dash=n`. `ellipse=a,b` draws an ellipse/arc inscribed in the box, where a = start° and b = sweep°; a bare `ellipse` with no numbers means 0,360. `flags=` see below | save.c:2961-3020 |
| Polygon | `P layer npts x y ... {p}` | point list | `fill` default **none**; `true` gives stipple, `full` gives solid. `dash=n`. `bezier=true` draws a chained quadratic B-spline that uses midpoints of consecutive control segments, 32 steps per segment (draw.c:1981-2040). `bus=` thick | save.c:2860-2915 |
| Arc / circle | `A layer x y r a b {p}` | centre, radius, a = start angle°, b = sweep° (X11 convention, 0° = +x, positive is CCW on screen); b=360 gives a full circle | `fill` default none (`true`/`full`). `dash=n` | save.c:2918-2959 |
| Text | `T {txt} x y rot flip xscale yscale {p}` | rot 0..3, flip 0/1, xscale = h-size, yscale = v-size | see text attrs table | save.c:2761, actions.c:895-927 |
| Wire | `N x1 y1 x2 y2 {p}` | segment on WIRELAYER (1) | `bus=true` draws thick. `lab=` is the net name; the netlister **writes** it back, unnamed nets get `#netN` (netlist.c:1382-1392) | save.c:2787-2810 |
| Instance | `C {sym} x y rot flip {p}` | rot 0..3; flip mirrors about x0 *before* rotation; rot=1 maps (x,y) to (x0-(y-y0), y0+(x-x0)), which is 90° clockwise on screen (xschem.h:339-346) | instance attrs table. `embed=true` saves the symbol inline as `[...]` | save.c:2811 |

**Rect `flags=` bitfield** (actions.c:752-785, `set_rect_flags`):

| token | bit | meaning |
|---|---|---|
| `graph` | 1 | rect is a waveform graph (lives on GRIDLAYER 2) |
| `unlocked` (with graph) | 2 | graph x-axis not locked to the other graphs |
| `private_cursor` (with graph) | 4 | graph has its own cursors |
| `image` | 1024 | rect is an image container |
| `unscaled` (with image) | 2048 | container resized to image pixel size instead of image scaled to rect (draw.c:4898) |

**Image rects** (draw.c:4838-4920, psprint.c:100-120, svgdraw.c:545-549): `image=<file>` (PNG/JPG/SVG; loaded once, then base64-embedded into `image_data`), `image_data=<base64>`, `filter=<shell cmd>` (pipe applied to the data, e.g. gzip -d), `alpha=<0..1>`, `InvertOnExport=true` / `ps_invert=true` (invert colours in PS/PDF export), `jpeg_quality=` / `jpg_quality=` (PS export compression).

**Text attributes** (`set_text_flags`, actions.c:895-927; drawing draw.c:820-890, 5250-5275):

| attr | values | effect |
|---|---|---|
| `font` | cairo font family | per-text font, default var `cairo_font_name` |
| `hcenter` / `vcenter` | true | centre horizontally / vertically on the anchor |
| `layer` | 0..cadlayers-1 | text colour layer, default TEXTLAYER 3 |
| `weight` | bold | bold |
| `slant` | italic \| oblique | slant |
| `hide` | true \| instance | `true` hides always; `instance` hides only when the symbol is instantiated (`HIDE_TEXT_INSTANTIATED`); shown again via "show hidden texts" |
| `name` / `floater` | instance name | makes it a **floater**: `@attr` tokens resolve against the named instance; text that names no instance but contains `@` or `tcleval(` is translated with inst=-1 (actions.c:100-151) |
| `symbol_ignore` | true | object not shown when the symbol is instantiated |

Text substitution: symbol texts go through `translate(inst)` and then `translate3(inst props, sym template)` (draw.c:877-883). Schematic texts are substituted **only** when they are floaters. Instance-level overrides `text_size_<n>` and `text_layer_<n>` apply to the n-th symbol text (draw.c:565-615). With `hide=true` on an instance only `@symname` and `@name` texts are drawn (draw.c:836).

**Generic per-object attrs**: `lock=true` makes the object unselectable unless lock is overridden (select.c:903-1214, findnet.c:46-478). `name=` gives an object a name usable by `attach=`. `attach="n1 n2"` on an instance co-selects the named objects (select.c:1059, move.c:538). `symbol_ignore=true` applies inside .sym files to L/B/P/A/T/N/C (save.c:4414-4853). `hide=true` on an instance draws only its bounding box. `hide_texts=true` hides all symbol texts.

**Instance render states** (draw.c:645-660): disabled (`*_ignore=true/open`) instances draw in GRIDLAYER grey. Shorted (`*_ignore=short`) instances draw in PINLAYER red. Types `launcher logo probe architecture noconn` are never greyed. `only_probes` mode greys everything except probes (actions.c:416, draw.c:844).

### 4A.2 Complete attribute catalogue (every literal key passed to `get_tok_value` in src/*.c)

Scope legend: **I** instance, **S** symbol global (`K{}`), **P** symbol pin rect (layer 5), **G** generic-pin rect (layer 3 in symbol), **O** graphic object, **Gr** graph rect, **NO** netlist_options instance, **F** file header.

| attr | scope | meaning | cite |
|---|---|---|---|
| `name` | I,P,O,T | instance refdes / pin name / object name for attach & floaters | actions.c:839 |
| `type` | S | symbol class (see 4A.6) | actions.c:795 |
| `template` | S | default attr values for instances | actions.c:792 |
| `format` | S,I | SPICE netlist line; an instance-level value overrides the symbol's | token.c:2176-2186 |
| `lvs_format` | S,I | used instead of `format` when `lvs_netlist=1` | spice_netlist.c:179 |
| `verilog_format` | S,I | Verilog primitive line | token.c:2903-2914 |
| `vhdl_format` | S,I | VHDL primitive line | token.c:940-952 |
| `tedax_format` | S,I | tEDAx primitive line | token.c:2573 |
| `spice_ignore` `verilog_ignore` `vhdl_ignore` `tedax_ignore` | S,I,P | `true`/`open` drop it; `short` shorts all pins together; P-level drops the pin from @pinlist | actions.c:803-830, token.c:2280 |
| `lvs_ignore` | S,I,NO | same values; active only if tcl `lvs_ignore=1`; on NO it sets that var | actions.c:827, netlist.c:609 |
| `spice_stop` `verilog_stop` `vhdl_stop` `tedax_stop` | S | emit the subckt header + global props but don't descend into instances | spice_netlist.c:629 |
| `spice_primitive` `verilog_primitive` `vhdl_primitive` | S | never emit .subckt/module/entity; only the format line | spice_netlist.c:498 |
| `spice_sym_def` `verilog_sym_def` `vhdl_sym_def` | S,I | text dumped instead of the subckt body (typ. `.include`); I-level needs `schematic=` | spice_netlist.c:659, actions.c:2053 |
| `default_schematic` | S | `ignore` = no implicit schematic; descend only via instance `schematic=` | actions.c:2080, spice_netlist.c:637 |
| `schematic` | S,I | alternate schematic; may contain `@symname`, `tcleval(...)`, `@attr`, generator `cmd(args)` | actions.c:2058,2193 |
| `extra` | S | format params that are extra pins, not .subckt params; also excluded from the Verilog param list | token.c:1778, spice_netlist.c:677 |
| `extra_pinnumber` | S,I | tEDAx pinnumbers for `extra` pins | token.c:2558 |
| `verilog_extra` | S | extra Verilog port connections | verilog_netlist.c:474 |
| `verilog_extra_dir` | S | `PIN=input\|output\|inout` for verilog_extra ports | verilog_netlist.c:573 |
| `verilogprefix` | S | prefix for module name / subckt expansion in Verilog | verilog_netlist.c:497 |
| `generic_type` | S,I | per-param type list (`m=integer model=string`); `string` gets quoted in VHDL/Verilog, `time` is skipped as Verilog param | token.c:1532-1824 |
| `sig_type` | I,P | VHDL signal type of a label/pin (default std_logic); `sig_type(PIN)=` override | netlist.c:1341 |
| `verilog_type` | I,P | Verilog net type (wire/reg...) | netlist.c:1342 |
| `class` | I | VHDL signal class (signal/constant) | netlist.c:1344 |
| `value` | I | initial value of VHDL/Verilog signal; netlist_commands body | vhdl_netlist.c:372 |
| `device_model` | S,I | .model/.subckt text emitted once at netlist end (instance wins) | spice_netlist.c:228-235 |
| `only_toplevel` | I | netlist_commands emitted only at top level | spice_netlist.c:216 |
| `place` | S,I | netlist_commands: `header` (top of SPICE file) / `end` | spice_netlist.c:325-558 |
| `global` | S,I | label nets become global (`.GLOBAL`); instance beats symbol | netlist.c:1333 |
| `lab` | I,N | net label name of label/pin instances; wire net name | actions.c:847 |
| `highlight` | S,I | highlight instance when an attached net is highlighted | actions.c:797,886 |
| `hide` | S,I,T | bbox-only symbol; text hide | actions.c:800,851 |
| `hide_texts` | I | hide all symbol texts | actions.c:853 |
| `embed` | I | store symbol inline in .sch | actions.c:889 |
| `lock` | all | not selectable/editable | callback.c:61 |
| `attach` | I | co-select named objects | select.c:1059 |
| `floater` | T | floater instance reference (alt. to `name`) | actions.c:921 |
| `url` `program` `tclcommand` | I | launcher: Shift-H / Alt-click opens url (with program, default xdg-open) or evals Tcl | actions.c:1826-1828 |
| `dir` | P | pin direction in/out/inout (ERC + port decl.) | netlist.c:479 |
| `pinnumber` | P,I | package pin; `pinnumber(name)` / `pinnumber(idx)` instance override; slotted `U1:2` picks 2nd of `a:b:c` | token.c:2634, 3700 |
| `sim_pinnumber` | P | port sort key for all netlists (all pins must have it) | save.c:3301-3370 |
| `numslots` | S,I | tEDAx slot count | token.c:2564 |
| `propag` | P | `n[,m]` pin indices for drill-highlight propagation | hilight.c:1068 |
| `goto` `clock` | P | built-in logic sim: outputs to update; clock 0/1 = active low/high clk, 2/3 = active low/high reset | hilight.c:1669-1670 |
| `function<n>` | S | RPN logic function for output pin n (`& \| ^ ~ M m z d x r H L Z U`) | hilight.c:1666 |
| `spiceprefix` | NO | enable/disable `@spiceprefix` substitution | netlist.c:635 |
| `bus_replacement_char` | NO | 2 chars that replace `[` `]` in net names | netlist.c:601 |
| `lvs_netlist` | NO | LVS netlist (implies top subckt + lvs_format) | netlist.c:615 |
| `top_is_subckt` | NO | top level emitted as real `.subckt` | netlist.c:628 |
| `hiersep` | NO | hierarchy separator (default `.`) | netlist.c:642 |
| `spice_netlist` `verilog_netlist` `vhdl_netlist` | S | in split-files mode, netlist this subckt in that format (mixed mode) | spice_netlist.c:493-495 |
| `savecurrent` | I,S | `@savecurrent` emits `.save I( ?1 inst )` | token.c:2257-2265 |
| `spice_data` | I | base64-embedded raw file (`xschem embed_rawfile`) | save.c:954 |
| `drc` | S | Tcl expression run if proc `fet_drc` exists; non-empty result = DRC message | editprop.c:1510-1545 |
| `file_version` | F | in `v{}` header; "1.0" files get `.sym` appended to symbol refs | save.c:3079, 4856 |
| `image` `image_data` `filter` `alpha` `InvertOnExport` `ps_invert` `jpeg_quality` `jpg_quality` | O(B) | image rects (above) | draw.c:4862 |
| `fill` `dash` `bus` `ellipse` `bezier` `flags` | O | see 4A.1 | save.c |
| `font` `hcenter` `vcenter` `layer` `weight` `slant` | T | see 4A.1 | actions.c:903-913 |
| `symbol_ignore` | O,I in .sym | not instantiated | save.c:4414 |
| `node` `color` `sweep` `dataset` `rawfile` `sim_type` `autoload` | Gr | traces, colours (layer idx), sweep var, dataset idx, per-graph raw file + type (tran/ac/op/dc/noise/table/...), auto-load raw | draw.c:2536-2648, 3846-4076 |
| `x1 x2 y1 y2` `divx divy subdivx subdivy` `unitx unity` `logx logy` `xlabmag ylabmag` | Gr | axis ranges, grid divisions, engineering unit prefix, log axes, label magnification | draw.c:3159-3246 |
| `digital` `ypos1 ypos2` `rainbow` `legend` `mode` `hilight_wave` `linewidth_mult` | Gr | digital (logic) display & vertical band, rainbow colouring, legend on/off, `Line`/`HistoV`/`HistoH`, bold trace idx, line width | draw.c:3195-3254 |
| `cursor1_x` `cursor2_x` `hcursor1_y` `hcursor2_y` | Gr | private / stored vertical (a,b) and horizontal cursors | draw.c:3146-4031 |

Pin-attribute override syntax on instances: `<attr>(<pinname>)=v` or `<attr>(<pinindex>)=v`, e.g. `sig_type(OUT)=bit_vector` (token.c:627-660).

### 4A.3 Layers, colours, fill patterns

| idx | define (xschem.h:148-157) | role | light / dark default (xschem.tcl:10024-10035) | fill (pixdata, xschem.tcl:10064-10152) |
|---|---|---|---|---|
| 0 | BACKLAYER | background | #ffffff / #000000 | solid |
| 1 | WIRELAYER, TEXTWIRELAYER, PROPERTYLAYER | wires, wire labels | #0055ee / #00ccee | solid |
| 2 | GRIDLAYER, SELLAYER | grid, selection, graph rects, disabled instances | #aaaaaa / #4f4f4f | empty |
| 3 | TEXTLAYER, GENERICLAYER | default text, VHDL generic-pin rects in symbols | #222222 / #cccccc | stipple |
| 4 | SYMLAYER | symbol body graphics | #229900 / #88dd00 | solid |
| 5 | PINLAYER | pins, ERC/short markers (red) | #bb2200 / #bb2200 | solid |
| 6..21 | user | free layers; highlight colours cycle over enabled layers >= 7 (actions.c:1041-1059, hilight.c:327-344) | 16 more colours | 6-9,11-13,15,16,18 stipple; 10 solid; 14,17,19-21 empty |

- `cadlayers` = 22 by default; it becomes 35 with `rainbow_colors=1` (xschem.tcl:10022,10039), and then layers >5 are solid (xinit.c:367).
- Fill style per layer comes from a 16x16 `pixdata(n)` bitmap: all ones gives solid, all zeros gives no fill, anything else stipple (xinit.c:328-367).
- `dark_colorscheme` (default 1, xschem.tcl:9702) selects which list feeds `tctx::colors`.
- `light_colors` / `dark_colors` can be edited per layer (`change_color`) and are persisted to `$USER_CONF_DIR/colors` (xschem.tcl:6239-6270).
- `svg_colors` is derived from the current scheme. `ps_colors` always comes from `light_colors` (xschem.tcl:6228-6231).
- `dim_value` / `dim_bg` (+ `enable_dim_bg`) brighten or darken colours. For d>=0: c += (51 - c/5)*d; for d<0: c += (c/5)*d (xinit.c:295-325).
- `enable_layer(n)` hides layers; hidden layers are also skipped in PDF/SVG export (Changelog:912-914). `xschem enable_layers` applies the setting.

### 4A.4 Hierarchy

| Feature | Behaviour | cite |
|---|---|---|
| Descend schematic (`e`) | Needs exactly one selected instance whose symbol type is `subcircuit` or `primitive`. An unnamed current schematic gets a save-as prompt first; a modified one prompts save. Builds a port map (child pin to parent net) for resolved-net lookups. Stores parent inst props + sym template in `hier_attr[level]` and pushes zoom. Max depth `CADMAXHIER`=40 | actions.c:2317-2504, xschem.h:197 |
| Vector instances | Descending into `x1[3:0]` asks for the instance number (1 = leftmost, negative counts from the right); path uses that bit's name. `change_sch_path` switches the bit afterwards | actions.c:2390-2410, 2267 |
| Descend symbol (`i`) | Opens the .sym for editing. Embedded symbols are dumped to `$XSCHEM_TMP_DIR/.xschem_embedded_<pid>_<name>` and reloaded into the parent on return (marks parent modified). The `missing` type cannot be descended | save.c:5152-5260, actions.c:2541-2565 |
| Go back (`Ctrl-e`) | Asks to save if modified, pops the level, reloads the parent, restores zoom, re-propagates highlights | actions.c:2506-2582 |
| Path naming | `sch_path[0]="."`; each descend appends `<instname>.` (e.g. `.x1.xdec[3].`). `@path` drops the leading `.` and skips levels above the raw-file level. Tcl `path` in tclpropeval2 gets an `x` prefix per component for SPICE | actions.c:2433-2443, token.c:3993-4007, xschem.tcl:6007-6011 |
| Schematic resolution | `get_sch_from_sym`: instance `schematic=` (translated with instance attrs) else symbol `schematic=`. `@symname` is replaced by the symbol base name, then `tcleval()` runs. Generator syntax `cmd(args)` is returned as-is. Otherwise `<symbol>.sch` from the symbol's dir, or searched across all paths if `search_schematic=1`. A missing instance schematic offers "descend into base schematic?" | actions.c:2162-2265 |
| Instance-based (polymorphic) schematics | During netlisting each instance with `schematic=` clones its symbol under the new name, keeping `parent_prop_ptr` and dropping `default_schematic`. `spice_sym_def` gets translated. The clone becomes its own subckt | actions.c:2025-2160 (`get_additional_symbols`) |
| `default_schematic=ignore` | Symbol never auto-descends or netlists the implied schematic and doesn't complain if missing | actions.c:2080, spice_netlist.c:637-640 |
| `hierarchy_config` idiom | `schematic="tcleval([hierarchy_config @symname])"` lets a user Tcl proc pick the view (doc symbol_property_syntax.html) | |
| Parameter passing (display) | `@x` values are resolved recursively up `hier_attr` levels: instance props, then parent props, then parent template (token.c:4595-4634) | |
| Parameter passing (netlist) | Instance params go on the `x...` line. Symbol `template` (minus `name`, `extra` tokens, spiceprefix) goes onto the `.subckt` line (`get_sym_template`, token.c:536; spice_netlist.c:690) | |
| Symbol generators | A symbol/schematic ref matching `^[^ \t()]+\([^()]*\)\s*$` (e.g. `gen.tcl(4,2)`) runs `/abs/gen.tcl 4 2` via popen; stdout is the symbol text. The name is sanitised to `gen_4_2` for subckt names | token.c:81-180, save.c:4267-4276 |
| Web symbols | http(s) schematics: referenced syms/schs are downloaded to `xschem_web_*` tmp dir (`try_download_url`) | save.c:4285-4292, actions.c:2244-2258 |
| Missing symbol | Falls back to `systemlib/missing.sym` (type `missing`) | save.c:4300-4306 |
| Library search | `XSCHEM_LIBRARY_PATH` is split on `:` (`;` on Windows), `~` expanded, non-existing dirs dropped, giving `pathlist`. Write-traced: changes re-run `set_paths` (xschem.tcl:9481-9520, 10249). `abs_sym_path name ?ext?`: abs path/URL as-is; `./` and `../` relative to pwd; else first `pathlist` dir where `dir/name` exists; else `pwd/name` (xschem.tcl:7241-7320). `rel_sym_path` strips the first matching pathlist prefix (xschem.tcl:7190-7238). Symbol refs in files are stored relative (e.g. `devices/res.sym`) | |
| LCC (Local Custom Cell) | A `.sch` can be instantiated directly like a `.sym`. Its ipin/opin/iopin become symbol pins (`add_pinlayer_boxes`), drawn with `systemlib/{i,o,io}pin_lcc[_top].sym`. Non-electrical types are skipped (logo, netlist_commands, netlist_options, arch_declarations, architecture, attributes, package, port_attributes, use, launcher, verilog_preprocessor, timescale) | save.c:4055, 4094-4113, 4838-4866; Changelog:915 |
| Highlight across hierarchy | `hilight_child_pins` on descend, `hilight_parent_pins` on return, `propagate_hilights` | hilight.c:564, 632, 1385 |

### 4A.5 @-substitution, tcleval, expr

**Tokeniser rules** (token.c:3956-3972, 4636-4643):
- A token starts at an unescaped `@` or `%`. It ends at whitespace, the next `@`/`%`, or a backslash-escaped char (`@var\iable` substitutes `@var`).
- A token needs more than 1 char.
- `\@` is a literal `@`.
- `%attr`: if undefined it yields the literal `attr` instead of empty (token.c:4600-4607).

**Value lookup** for generic `@attr`: instance prop_ptr, then symbol template, then (display) parent levels recursively. Undefined gives empty.

**@-tokens recognised** (translate token.c:3880-4664; print_spice_element token.c:2150-2535; vhdl/verilog/tedax printers token.c:923-1219, 2537-3189):

| token | expands to | where |
|---|---|---|
| `@name` | instance name (vector names expanded in netlists) | all |
| `@symname` | symbol name without extension/path dir | all |
| `@symname_ext` | symbol name with extension | all |
| `@symref` | symbol reference exactly as in instance | all |
| `@path` | hierarchy path of the instance (no leading `.`) | translate |
| `@pinlist` | nets on all pins in pin order as `?mult net` pairs. Duplicate pin names once; spice_ignore'd pins skipped; port order taken from the `.subckt` in `spice_sym_def` if present (token.c:1868-1990, 2272-2292) | netlists |
| `@@pin` | net on the pin named `pin` | netlists + translate |
| `@#n` / `@#pin` | `?mult net` on pin index n / name | netlists |
| `@#n:attr` / `@#pin:attr` | pin attribute (instance `attr(pin)` override wins). Special attrs: `net_name` (attached net), `resolved_net` (full hier net), `spice_get_voltage` (sim value at that net), `pinnumber` (slot-aware) | token.c:3587-3731 |
| `@sch_last_modified` `@sym_last_modified` `@time_last_modified` | mtime `%Y-%m-%d  %H:%M:%S` of the inst's .sch / the .sym / the current .sch | translate |
| `@schname` / `@schname_ext` | current schematic name without / with ext | all |
| `@topschname` | top-level schematic name | all |
| `@prop_ptr` | the instance's full property string | translate |
| `@schprop` `@schvhdlprop` `@schverilogprop` `@schtedaxprop` `@schsymbolprop` | the schematic S/G/V/E/K global property blocks | translate |
| `@spiceprefix` | value of the instance `spiceprefix`, only if tcl `spiceprefix=1`. Emitted as `**** spice_prefix X` and folded by spice.awk | token.c:2382-2465 |
| `@savecurrent` | `.save I( ?1 inst )` if savecurrent=true | spice |
| `@tcleval(<script>)` | `tclpropeval` script with `$instname`/`$symname` in scope | netlists (token.c:2368) |
| `@spice_get_voltage` | op/cursor-b voltage of pin-0 net | translate (needs raw loaded + `live_cursor2_backannotate`) |
| `@spice_get_voltage(net)` | voltage of named net | translate |
| `@spice_get_diff_voltage` | V(pin0) - V(pin1) of a 2-pin instance | translate |
| `@spice_get_current`, `@spice_get_current_<p>`, `@spice_get_current(dev)`, `@spice_get_current_<p>(dev)` | device branch current / named current param (e.g. `_ib`) | translate (token.c:4264-4356, 4420-4540) |
| `@spice_get_modelparam[_<p>][(dev)]` | model param (e.g. `gm`) | translate |
| `@spice_get_modelvoltage[_<p>][(dev)]` | model voltage (e.g. `vth`) | translate |
| `@spice_get_node <spicenode> ` | value of an arbitrary raw-file vector; space-delimited; one per string | token.c:3733-3785 |
| `#pfx#@name` | (format) `pfx` prefixed to every bit of a vector name, done in spice.awk | spice.awk:188-210 |
| `##tok` | `##` prefix stripped by spice.awk (protects from expansion) | spice.awk |

Sim-value tokens render as engineering notation. `-` means not found. Nets `0`/`GND` render `0.0`.

**tcleval rules**:
1. `get_tok_value` auto-evaluates any value that **starts** with `tcleval(`. Exceptions: `with_quotes&2`; `\}` is unescaped to `}` first (token.c:59-79, 519-522).
2. Evaluation is `tclpropeval2`: `uplevel #0 subst {...}`. Globals `path`, `sch_basename`, `raw_level`, `netlist_type` are set (xschem.tcl:5991-6031).
3. After `@` substitution the whole translated string is passed through `tcl_hook2` again (token.c:4655, 2519-2522).
4. Errors yield `?\n`.
5. Because `prepare_netlist_structs` calls `reset_caches`, flags like `spice_ignore="tcleval(...)"` stay live (netlist.c:1517).

**expr()** (eval_expr.y): Any `expr(...)` or `expr('...')` in the result is evaluated after translation (token.c:4657-4662, 2524-2526).
- Operators: `+ - * / %(int) ^`, unary minus, parens.
- Functions: `int sin cos exp asin acos atan log(=log10) ln sqrt round(6 sig)`.
- Constants: `pi e k h echarge abszero c`.
- Numbers accept engineering suffixes (`atof_eng`) (eval_expr.y:46-64, 82-110, 256).

**Attribute string syntax / escaping** (token.c:425-534):
- Whitespace-separated `key=value`. Values containing spaces go in `"..."`.
- `\"` is a literal quote and `\\` a backslash. `get_tok_value(...,0)` strips unescaped `\` and `"`.
- A bare key with no `=` matches with an empty value (`tok_size>0`).
- `subst_token` (token.c:1220) is the canonical edit primitive.
- Boolean parsing: `strboolcmp` / `get_attr_val` accept `true/on/yes/1` and `false/off/no/0`, case-insensitive (editprop.c:87-101).

### 4A.6 Symbol `type` values with special semantics

| type | semantics | cite |
|---|---|---|
| `subcircuit` | has a schematic; descended into; emitted as `.subckt`/module/entity | spice_netlist.c:478 |
| `primitive` | may be descended for viewing, but netlisted only via format | actions.c:2360 |
| `label` | names a net (`lab`); `global=true` makes it global; one pin | netlist.c:1316-1352 |
| `ipin` `opin` `iopin` | schematic ports; drive `.subckt` port list, `*.ipin` comments, `*.PININFO` (dir I/O/B) | spice_netlist.c:186-205 |
| `show_label`, `scope`, `bus_tap` | treated like labels for highlight (IS_LABEL_SH_OR_PIN, xschem.h:379). `bus_tap`: pin1 on bus, pin0 named `<busbase><lab>` where lab is `[n]` / `[a:b]` (netlist.c:1147-1190). `scope`: placement auto-attaches an embedded graph (actions.c:1590) | |
| `netlist_commands` | `value`/`format` text copied verbatim (`**** begin user architecture code`); `place=header/end`, `only_toplevel` | spice_netlist.c:214-240 |
| `netlist_options` | sets bus_replacement_char, lvs_ignore, lvs_netlist, top_is_subckt, spiceprefix, hiersep | netlist.c:598-646 |
| `noconn` | marks intentionally unconnected net | tedax_netlist.c:49 |
| `probe` | back-annotation probe; drawn even in only_probes mode; `ngprobe` doc'd as pull-method probe | select.c:49, draw.c:650 |
| `launcher` | url/tclcommand hyperlink object | draw.c:649 |
| `logo` | title block, non-electrical | draw.c:649 |
| `architecture` `arch_declarations` `attributes` `package` `use` `port_attributes` | VHDL: instance prop string printed verbatim in architecture body / declarations / attributes / before entity / use clauses / port attributes | vhdl_netlist.c:45-84,160-177,292 |
| `generic` | VHDL generic pin | vhdl_netlist.c:216 |
| `timescale` `verilog_preprocessor` | Verilog: printed before module | verilog_netlist.c:54-55,136 |
| `ammeter` `vsource` | current probes: highlight sends I(dev) to viewer | hilight.c:1940 |
| `missing` | placeholder for unresolved symbol | save.c:5186 |

Types needing no `name` (no ERC warning): package, port_attributes, architecture, arch_declarations, attributes, netlist_options, use (netlist.c:1288-1296). The other library types (nmos, pmos, resistor, capacitor, vsource...) are informational only.

### 4A.7 Netlisting

**Pipeline**:
1. C writes a temp file `$netlist_dir/.<cell>_<pid>`.
2. The Tcl `netlist` proc post-processes it with awk into `$netlist_dir/<cell>.<ext>` (xschem.tcl:972-1040).
3. SPICE: `spice.awk [-xyce] | [flatten.awk if flat_netlist] | break.awk`.
4. VHDL: `vhdl.awk`. tEDAx: `tedax.awk | flatten_tedax.awk`.
5. Verilog: `verilog.awk [-bitblast]` then `convert_to_verilog2001.awk` if `verilog_2001`.

| Format | Driver | Top-level structure | Sub-block |
|---|---|---|---|
| SPICE | `global_spice_netlist` spice_netlist.c:252 | `** sch_path:`; `place=header` cmds; netlist_options; `**.subckt top <ports>` (commented unless top_is_subckt/lvs_netlist; ports ordered by sim_pinnumber or by `<top>.sym` pins if that exists); `*.PININFO a:I b:O c:B`; pin comment lines (`*.ipin x`); element lines; netlist_commands (not header/end); `S{}` global prop; `**.ends`; subckts; `.GLOBAL n`; `place=end` cmds; device_model table; `.end` | `spice_block_netlist` spice_netlist.c:610: `* expanding symbol: X # of pins=N`, `** sym_path:`, `** sch_path:`, `.subckt name pins templ-params`, body, `S{}` prop, `.ends` |
| Verilog | `global_verilog_netlist` verilog_netlist.c:78 | timescale/preproc, `module top (ports);` `input/output/inout` + type decls (`verilog_type`, initial `value`), `---- begin signal list`, instances, `V{}` prop, `endmodule` | `verilog_block_netlist` :416 (verilog_sym_def, verilog_stop, verilog_extra pins, `parameter` from template with generic_type quoting) |
| VHDL | `global_vhdl_netlist` vhdl_netlist.c:105 | packages, use, `entity top is port(...)`, port_attributes, `architecture arch_top of top is`, components, signals (sig_type/class/value), attributes, `begin`, instances, `G{}` prop, `end` | `vhdl_block_netlist` :504 |
| tEDAx | `global_tedax_netlist` tedax_netlist.c:124 | `tEDAx v1`, `begin netlist v1 top`, conn/device lines from `tedax_format`/pinnumber/slots, `__GLOBAL__`, `__HIERSEP__`, `end netlist` | `tedax_block_netlist` :72 |

**Element line generation** (`print_spice_element`, token.c:2150-2535):
- Format lookup: instance `<fmt>`, then symbol `<fmt>`, then instance `format`, then symbol `format`. `<fmt>` is `lvs_format` or a custom name set with `xschem set format X`.
- No format means the instance is not netlisted (token.c:2187-2193).
- Values for `name` and `lab` go through `expandlabel` (vector instances, token.c:2476-2480).
- Unresolved `@x` becomes empty; `%x` becomes `x`.
- The result passes `tcl_hook2` then `eval_expr`.

**Symbol iteration**: All symbols in use, plus instance-schematic clones, are walked once by cell name. Skipped:
- spice_ignore/short symbols;
- `lvs_ignore` symbols when the var is set;
- symbols matching an `xschem_libs` regex (check_lib, netlist.c:565-596);
- `default_schematic=ignore`;
- `spice_primitive=true`.

`split_files` gives one file per subckt; `spice_netlist`/`verilog_netlist`/`vhdl_netlist` pick the format per block (spice_netlist.c:490-498).

**Flat vs hierarchical**: The netlist is always hierarchical in C. `flat_netlist=1` pipes through `flatten.awk` (CDL flattener, params and globals) (xschem.tcl:998-1002). `flatten_savenodes.awk` hierarchises `.save` nodes.

**spice.awk post-processing** (spice.awk:26-339):
- Joins `+` continuation lines (outside user code).
- Expands `?n` multiplicity: one line per bit of vector instance names, and nets are split per bit.
- Resolves parametric vector names `R1[m:n]` from `.param`.
- Applies `**** spice_prefix`.
- Prefixes `#p#name` / strips `##`.
- Rewrites ` value=` to `cur=` (G), `vol=` (E), `r=` (R), `c=` (C); D `PERI=` becomes `PJ=`; drops `PARAM:`.
- Explodes `.save v(a,b)` into individual saves.
- Removes `m=` from `.subckt` lines.
- Handles special Y-devices.
- `-xyce`: `.save` becomes `.print`, and `m=1` is dropped.

`break.awk` wraps lines over 130 chars with `+`; `.include/.lib/.save/.title/.write` and user code are not wrapped.

**Netlist options vars** (UI: Options > Netlist, Simulation > LVS; xschem.tcl:8987-9007, 9396-9406): `netlist_type` spice|vhdl|verilog|tedax|symbol, `flat_netlist`, `split_files`, `netlist_show`, `netlist_dir` / `local_netlist_dir` (0, 1=`./simulation`, 2=`./simulation/<cell>`), `spiceprefix`, `lvs_netlist`, `lvs_ignore`, `top_is_subckt`, `uppercase_subckt`, `verilog_2001`, `verilog_bitblast`, `bus_replacement_char`, `keep_symbols`, `search_schematic`, `xschem_libs` / `noprint_libs` / `nolist_libs` (regex exclusion for netlist / hier print / list_hierarchy, xschemrc:122-135).

**Hierarchical print / list** (`hier_psprint`, spice_netlist.c:46-140): walks the hierarchy for multi-page PDF, or for a cell list with mtimes (`list_hierarchy`).

### 4A.8 Net naming, labels, buses, connectivity

**Connectivity build** (`prepare_netlist_structs`, netlist.c:1509-1549):
1. Spatial hash of wires and inst pins.
2. Label/pin instances seed node names from `lab` (else template `lab`). Names propagate through touching wires (recursive `wirecheck`) and instance pins.
3. Pass-through symbols (duplicate pin names) or `*_ignore=short` instances copy the name across pins (netlist.c:1097-1250).
4. Remaining wires get `#netN` and are written back as `lab=` (netlist.c:1382-1392).
5. Unconnected pins get `#netN`. In the netlist `#` is stripped, giving `netN`. Bus-width unnamed nets become `netN[k:0]` (token.c:3437-3452).
6. A truly unconnected pin becomes `__UNCONNECTED_PIN__k` (token.c:3424-3428).
7. VHDL generics are named via generic pins (`name_generics`, netlist.c:797).

| Item | Detail |
|---|---|
| Label symbols | `devices/ipin.sym` (`*.ipin @lab`), `opin.sym`, `iopin.sym`, `lab_pin.sym` / `lab_wire.sym` (type label, `*.alias @lab`, template `sig_type=std_logic lab=xxx`), `gnd.sym` / `vdd.sym` (label, `global=true`, function0 L/H), `noconn.sym`, `bus_tap.sym`, `generic_pin.sym`, `lab_show.sym`, `lab_generic.sym` (xschem_library/devices) |
| Global nets | `0` always global; any label with `global=true` (instance or symbol). SPICE emits `.GLOBAL n`, tEDAx `__GLOBAL__ n` (netlist.c:701-733) |
| Placement helpers | `place_net_label` (Alt-L pin label, Alt-Shift-L wire label, Ctrl-P ipin, Ctrl-Shift-P opin; actions.c:1451); `attach_labels_to_inst` (Shift-H; auto-labels every pin of the selected instance, options use_lab_wire / prefix / rotated_text; actions.c:1273, xschem.tcl:6031) |
| Name uniqueness | Instance names are kept unique **case-insensitively**, including expanded vector bits. On clash: basename + first free index + bracket suffix (`m1` to `m2`, `x1[3:0]` to `x2[3:0]`). Toggle `disable_unique_names`. `check_unique_names(rename)` highlights duplicates in red or renames them (token.c:721-882) |

**Bus syntax** (`expandlabel`, expandlabel.y + parselabel.l; doc tutorial_busses.html):

| form | expands to |
|---|---|
| `A,B,C` | 3 nets |
| `A[3:0]` | A[3],A[2],A[1],A[0] |
| `A[6:0:2]` | start:end:step gives A[6],A[4],A[2],A[0] |
| `A[0:1:4:3]` | start:end:offset:repetitions gives A[0],A[1],A[4],A[5],A[8],A[9] |
| `A[3..0]` / `A[6..0..2]` / `A[a..b..c..d]` | same, **without brackets**: A3,A2,A1,A0 |
| `A[1,3,5:4]` | index lists mixing singles and ranges |
| `2*A[1:0]` / `2*(A,B)` | prefix replication of the whole list: A[1],A[0],A[1],A[0] |
| `A[1:0]*2` / `(A,B)*2` | postfix replication per element: A[1],A[1],A[0],A[0] |
| `A[3:0]suffix` | trailer appended to each bit |
| `*comment` at line start | B_LINE, passed through |

Allowed name chars include ``-a-zA-Z0-9_%$~"+#!/\<>`` (parselabel.l:163-177). Index numbers may be negative. Instance names use the same syntax (vector instances). When an instance vector of width W connects a pin of width P to a net of width N, bits are assigned round-robin: `((inst-1)*P + k - 1) % N` (actions.c:2422-2425).

**Net highlight / probe** (hilight.c):

| Function | Behaviour | cite |
|---|---|---|
| `hilight_net(viewer)` | Highlights selected wires/labels' nets in the current colour. Instances are highlighted individually. viewer = none / XSCHEM_GRAPH (adds to graph via `graph_add_nodes_from_list`) / GAW (TCP `gaw_fd`) / BESPICE (socket `add_voltage_on_spice_node_to_plot`). `ammeter`/`vsource` instances send I(dev) | hilight.c:1886-1960, 1128-1380 |
| Colour cycling | `incr_hilight` increments the colour per net; colours are enabled layers >= 7 | hilight.c:327-344 |
| `propagate_hilights` | Re-applies highlights after a hierarchy move; `en_hilight_conn_inst` / instance `highlight=true` paints attached instances | hilight.c:1385 |
| Drill highlight (`propag`) | Follows pin `propag=n` through instances | hilight.c:1027-1108 |
| `hilight_netname(name)` | Highlight by name | hilight.c:1110 |
| `resolved_net` | Maps a local net to its top-level hierarchical name via portmaps; globals returned as-is | hilight.c:2065 |
| `print_hilight_net(show)` | Lists highlighted nets (optionally bus-expanded) or creates pins/labels from them (`create_pins`, `add_lab_prefix`, `add_lab_no_prefix`) | hilight.c:2247 |
| `select_connected_nets(stop_at_junction)` / `select_dangling_nets` | Connectivity selection | select.c:93, 164 |
| Built-in logic sim | `logic_set` drives a net 0/1/X/Z; evaluates `function<n>` RPN per symbol, `goto`, `clock`; propagates | hilight.c:1457-1885 |
| `search(tok,val,sub,sel,match_case)` | Regex/substring search of attr values, highlight or select | hilight.c:719 |

### 4A.9 ERC / checks

| Check | Message | cite |
|---|---|---|
| Instance without name (types needing one) | `Warning: instance: X (sym): no name attribute set` | netlist.c:1297 |
| Symbol without type | `Warning: Symbol: X: no type attribute set` | netlist.c:1306 |
| Pin missing name/dir | `Error: instance X, symbol Y: missing all or name or dir attributes on pin` | netlist.c:481 |
| Two differently named labels on one net / bus tap clash | `Error: <tag> shorted: n1 - n2` | netlist.c:866-885 |
| Unconnected pin | `Warning: unconnected pin, Inst idx..` (instance painted red) | token.c:3418 |
| Node dir analysis (pin `dir` counts) | `Error: undriven node`, `Warning: open net`, `Warning: shorted output node`, `Warning: node: X goes nowhere` (globals exempt) | node_hash.c:192-240 |
| Perfectly overlapped instances | `Error: overlapped instance found` | netlist.c:1586-1620 |
| Symbol vs schematic pins | pin count mismatch, sch pin not in sym, sym pin not in sch, direction mismatch, no direction | netlist.c:1625-1900 |
| Duplicate instance names | highlight red / rename | token.c:818 |
| `drc` symbol attr + user `fet_drc` proc | arbitrary DRC text | editprop.c:1510 |
| All-pins-disconnected | code present but `#if 0` (disabled) | netlist.c:1435-1460 |

Wire hygiene (check.c):
- `trim_wires` (:151) repeats until nothing changes. Each pass:
  - breaks wires at T-junctions;
  - deletes wires contained in other wires;
  - merges parallel wires that touch end-to-end when nothing else connects at that point.
- `break_wires_at_pins(remove)` splits wires at selected-instance pins and optionally deletes wires fully inside instance bboxes (:534).
- `break_wires_at_point` (:469).
- `update_conn_cues` (:48-118) draws junction dots. Per wire end it adds +1 for each other wire end at that point and +2 for each mid-segment touch. A dot is drawn when the count is >1. Only wires count; pins don't.
- `connect_by_kissing` (actions.c:1077) adds wires from moving instance pins and selected wire ends to the objects they were touching, so connections survive a move.

Errors are shown in the info window (`show_infotext`). A netlist with errors exits with code 10 (spice_netlist.c:603-605).

### 4A.10 Symbol / schematic generation and import

| Feature | Implementation |
|---|---|
| Make symbol from schematic (`A`) | Saves, then `make_symbol` Tcl runs `awk -f make_sym.awk <symbol_width> file.sch` (xschem.tcl:5429, save.c:3244). Layout: ipins and generic pins (B 3 rects) on the left, opins/iopins on the right, sorted by schematic y. Pitch 20, pin box +/-2.5, pin stubs 20 long on layer 4, body polygon on layer 4. Half-width `symbol_width` default 150. Texts `@symname` and `@name`. Pin rect props: `name dir=in/out/inout sig_type verilog_type value` plus all remaining pin-instance props (e.g. `*_ignore`, `sim_pinnumber`). Writes the default `K{type=subcircuit format="@name @pinlist @symname" template="name=x1"}`; the code tries to keep an existing K block but its regex never matches (make_sym.awk:24-77, 230-330) |
| Make schematic from symbol (Ctrl-L) | `create_sch_from_sym`: writes `<sym>.sch` with ipin (x=-120), opin/iopin (x=+120), generic_pin per symbol pin, 20 units apart, `lab=` from pin name (save.c:5033-5150) |
| Make schematic+symbol from selection (Ctrl-Shift-H) | `make_schematic_symbol_from_sel`: writes the selection to a new .sch (K `type=subcircuit`), replaces the selection with an instance of it, optionally `make_sym_lcc.awk` (save.c:4999, 3257) |
| Sym pins from sch pins (Alt-H) | `schpins_to_sympins` (xschem.tcl:5279) |
| Place symbol pin (Alt-P) | `xschem add_symbol_pin` |
| Pin list placement | `place_pins.tcl` (instances from a "name num" file), `place_sym_pins.tcl` (pin rects + labels in a symbol) |
| symgen | `symgen.awk`: djboxsym-like symbol from a symdef text (doc tutorial_symgen.html) |
| From SPICE | `make_sch_from_spice.awk`, `make_sym_from_spice.awk` (use `*.PININFO` for dirs, else inout); `make_vhdl_from_spice.awk` |
| Other importers | `gschemtoxschem.awk` (gEDA .sch/.sym), `import_opus_symbols.awk` (Cadence), `make_sch_from_cadence_pin.awk`, `make_sch_from_vhdl.awk`, `parse_synopsys_vhdl.awk`, `herculestospice.awk`, `make_edif.awk`; libraries `gschem_import/`, `viewdraw_import/` |
| Label utilities | `label_compactor.awk`, `netlist_compactor.awk`, `sort_labels.awk`, `order_labels.awk`, `reduce_even_odd_array_labels.awk`, `change_ref.awk`, `rescale.awk`, `change_index.tcl` |

### 4A.11 Compare / LVS-related

| Feature | Detail | cite |
|---|---|---|
| `xschem compare_schematics [file]` | Hashes instances by `C sym x y rot flip props` and wires by `N x1 y1 x2 y2` in both schematics, checks both directions, and draws mismatches in PINLAYER red over the current view. Returns 1 if they differ. Menu: "Set schematic to compare and compare with", "Swap compare schematics" (`swap_compare_schematics`), toggle `compare_sch` (redraw always overlays the diff) | xinit.c:738-900, scheduler.c:627-643, draw.c:5311, xschem.tcl:7328, 9261-9268 |
| LVS netlist | `lvs_netlist=1` makes top a real `.subckt` with `*.PININFO` and switches to `lvs_format` per symbol | spice_netlist.c:176-205 |
| `lvs_ignore` | Per-instance/symbol open/short applied in all formats when var set; also `xschem rebuild_connectivity` | actions.c:827, xschem.tcl:9403 |
| `uppercase_subckt` | `.SUBCKT`/`.ENDS` | spice_netlist.c:346-352 |
| Custom format | `xschem set format <attr>` picks any attribute name as netlist rule with fallback to `format` | scheduler.c:5349, token.c:2176-2186 |
| sym_vs_sch_pins | Interface consistency check during netlist | netlist.c:1625 |
| Toggle ignore (keys) | `toggle_ignore` cycles the selected instances' `<attr>` false → true → short | actions.c:1974-2020 |

### 4A.12 Embedded symbols

- An instance with `embed=true` causes the symbol definition to be written once after the first such `C` line as `[\n<full .sym text>\n]` (save.c:2492-2600). All instances of that symbol then use the embedded copy (`EMBEDDED` flag).
- On load, a `[` after a `C` line is parsed as an inline symbol (`load_sym_def(name, embed_fd)`, save.c:4209, 4296).
- Editing an embedded symbol (descend) round-trips through a temp file. Changes stay local to the schematic.
- Removing `embed` reverts to the library symbol after save and reload (doc component_property_syntax.html).
- `get_sym_type` peeks at embedded `[...]` blocks while loading LCC schematics (save.c:4820-4835).

## 4B. Core features: simulation, waveforms, export, editing infrastructure

Paths are relative to the XSchem 3.4.7 source tree (`src/`, `doc/xschem_man/`, `xschem_library/`). `xschem.tcl` means `src/xschem.tcl`.

### 4B.1 Simulation launching and configuration

#### `sim` Tcl array (tool configuration)

Defaults come from `set_sim_defaults` (xschem.tcl:1537). If it exists, `~/.xschem/simrc` is sourced in their place. `save_sim_defaults` (xschem.tcl:1173) writes the file. Keys never contain spaces around commas.

| Key | Meaning |
|---|---|
| `sim(tool_list)` | Known tools. Default is `spice spicewave verilog verilogwave vhdl vhdlwave`. |
| `sim(<tool>,n)` | Number of configured commands for the tool |
| `sim(<tool>,default)` | Index of the command that runs (radio button in the dialog) |
| `sim(<tool>,<i>,cmd)` | Command line. `simulate` runs it through Tcl `subst -nobackslashes` (xschem.tcl:2658). |
| `sim(<tool>,<i>,name)` | Display name. `create_plot_cmd` (hilight.c:400) matches `Gaw`, `Bespice` or `Ngspice` in this name to pick the cross-probe protocol. |
| `sim(<tool>,<i>,fg)` | 1 = foreground: `execute_wait`, which blocks the UI with `vwait` |
| `sim(<tool>,<i>,st)` | 1 = show a status dialog (stdout, stderr, exit code) when the process ends |

Default commands (xschem.tcl:1566-1672):

| Tool | # | Name | Command |
|---|---|---|---|
| spice | 0 | Ngspice interactive | `$terminal -e {ngspice -i "$N" -a \|\| sh}` (on Windows: `ngspice -i "$N" -a`) |
| spice | 1 | Ngspice Control mode | `ngspice "$N" -a` (st=1) |
| spice | 2 | Ngspice batch | `ngspice -b -r "$n.raw" "$N"` |
| spice | 3 | Xyce batch | `Xyce "$N"` |
| spice | 4 | Xyce parallel batch | `mpirun /path/to/parallel/Xyce "$N"` |
| spicewave | 0 | Gaw viewer | `gaw "$n.raw"` |
| spicewave | 1 | Ngspice Viewer | `$terminal -e ngspice` |
| spicewave | 2 | Rawtovcd | `rawtovcd -v 1.5 "$n.raw" > "$n.vcd" && gtkwave "$n.vcd" "$n.sav"` |
| spicewave | 3 | Bespice wave | `$env(HOME)/analog_flavor_eval/bin/bspwave --socket localhost $bespice_listen_port "$n.raw"` |
| verilog | 0 | Icarus verilog | `sh -c "iverilog -o .verilog_object -g2012 '$N' && vvp .verilog_object"` |
| verilog | 1 | Verilator | `sh -c "verilator -Wno-fatal --binary --timing --cc --Mdir obj_$s -o $s '$N' --trace && ./obj_$s/$s"` |
| verilogwave | 0 | Gtkwave | `gtkwave dumpfile.vcd "$N.sav"` |
| vhdl | 0 | Ghdl | `ghdl -c --ieee=synopsys -fexplicit "$N" -r "$s" "--wave=$n.ghw"` |
| vhdlwave | 0 | Gtkwave | `gtkwave "$n.ghw" "$N.sav"` |

#### Substitution variables in command strings

These are Tcl variables, not `%` codes (xschem.tcl:2567-2590 and 2615-2640).

| Var | Value |
|---|---|
| `$N` | Full netlist path: `$netlist_dir/<s>.<netlist_type>`. Verilog uses `.v`. |
| `$n` | `$N` without its extension |
| `$s` | Schematic basename, or the root of `xschem get netlist_name` if one is set |
| `$S` | Full path of the schematic |
| `$d` | Netlist directory |
| `$terminal` | Global terminal variable (default `xterm`, xschem.tcl:9979) |
| `$env(...)`, `$bespice_listen_port`, any global | Expanded by `subst` |

#### Simulation flow

| Item | Behaviour | Source |
|---|---|---|
| `simulate ?callback?` | Requires `set_netlist_dir 0` to be non-empty. Picks `sim($netlist_type,default)`, `cd`s to netlist_dir and runs `execute $st $cmd`. Sets the Simulate menubar button to orange. Returns the exec id. | xschem.tcl:2621 |
| `simulate_from_button` | If Simulate is orange (a job is running), sends SIGTERM (`kill -15`) to it instead of launching | xschem.tcl:2596 |
| `set_simulate_button` | Completion callback. No exit code: reset to `$simulate_bg`. Exit 0: **Green**. Non-zero: **red**. | xschem.tcl:8441 |
| Netlist button color | Green or red after `xschem netlist -erc` (errors give red). Reset to `$simulate_bg` on any modify. | scheduler.c:3488-3494, actions.c:167 |
| Waves button color | Green while waves are loaded for the current schematic. Turns orange when a netlist is regenerated (stale waves). | save.c:1061, xschem.tcl:977-983 |
| Per-tab button state | `tctx::<win_path>_{netlist,simulate,waves,simulate_id}`, restored on tab switch by `set_sim_netlist_waves_buttons` | xschem.tcl:8476 |
| `execute status cmd...` | Asynchronous pipe with `fileevent`. Status flags: `rw` (r+ pipe), `line` or `none` buffering, `1` = report on end. Keeps `execute(data,id)`, `(error,last)`, `(exitcode,id)` and `(cmd,last)`. Runs `execute(callback,id)` at the end. | xschem.tcl:352, 242 |
| `execute_wait` | Same, but blocks with `vwait` and bumps the semaphore | xschem.tcl:328 |
| Sub-process dialog | Simulation > List running sub-processes. Listbox with buttons Terminate selected (-15), Kill selected (-9), View status (live stdout), Dismiss. Not available on Windows. | xschem.tcl:510, 401 |
| View last job data / errors | `viewdata $execute(data,last)` and `$execute(error,last)` | xschem.tcl:9336-9345 |
| `xschem simulate ?callback?` | C entry point. Calls `set_netlist_dir(0)` and then the Tcl `simulate`. | scheduler.c:5805 |
| CLI `-S/--simulate`, `-W/--waves` | Run the simulation or the waves viewer after load | xinit.c:3063-3075 |
| `s` key | Run simulation after asking for confirmation | keys.help |
| Simconf dialog | Simulation > Configure simulators and tools (`simconf`). One row per tool command: name entry, default radio, 3-line cmd text, Fg checkbutton, Status checkbutton. Buttons: Cancel, Help, Accept Save and Close (writes simrc), Accept no Save and Close, Reset to default (deletes simrc). | xschem.tcl:1704 |

#### Netlist / simulation directory

| Var / proc | Behaviour |
|---|---|
| `netlist_dir` | Default `$USER_CONF_DIR/simulations` (xschem.tcl:9837). Created by `set_netlist_dir 0` when missing. |
| `local_netlist_dir` | 0 = use `netlist_dir`. 1 = `<sch_dir>/simulation`. 2 = `<sch_dir>/simulation/<cellname>` (`simuldir`, xschem.tcl:5453). Exposed as three radio buttons in the Simulation menu. |
| `set_netlist_dir what ?dir?` | what=0: ensure the directory exists. 1: prompt with `tk_chooseDirectory`, or use `dir`. 2: only apply the `local_netlist_dir` rule (xschem.tcl:5481). |
| Set top level netlist name | `xschem set netlist_name` (overrides `$s`) |
| CLI | `-o/--netlist_path`, `-N/--netlist_filename` |
| Shell [simulation path] | `get_shell $netlist_dir` opens `$terminal` in netlist_dir |
| Edit Netlist | `edit_netlist` runs `$editor` on the netlist (xschem.tcl:3912) |
| Utile Stimuli Editor / Translate | Runs the external `utile` tool on `<sch_dir>/stimuli.<cell>` (src/utile) |

#### External waveform viewers and cross-probing

| Viewer | Mechanism | Source |
|---|---|---|
| Any (`waves external`) | Runs `sim(<netlist_type>wave,default)` with the same `$N $n $s $S $d` substitution | xschem.tcl:2779 |
| gaw | TCP client to `$gaw_tcp_address` (default `{localhost 2020}`). Sends `table_set <s>.raw` and `copyvar v(<path><net>) sel #rrggbb`. Xyce uses `:` as the path separator and uppercase names. | xschem.tcl:2713, 2742, hilight.c:457-465 |
| bespice | xschem acts as a TCP **server** on `$bespice_listen_port` (0 = auto-pick). On connect it sends `set_customer_specialization xschem`, then pushes probe commands. | xschem.tcl:2407, 9658, hilight.c:356 |
| ngspice | `create_plot_cmd` writes `$netlist_dir/xplot` containing `.control`, `set colorN=rgb:..` and `plot "<path><net>" ...`, grouping 8 nets per plot. You then type `xplot` in ngspice. | hilight.c:371 |
| gtkwave | `gtkwave_server.tcl` is sourced inside gtkwave and listens on port 2022. It runs Tcl commands it receives. Not used by the C code. | src/gtkwave_server.tcl |
| Send to viewer | `xschem send_to_viewer` / Alt-G (selected nets → gaw/bespice, or adds to the selected internal graph). `xschem create_plot_cmd` / Ctrl-Shift-X (highlighted nets → xplot). | scheduler.c:5257, 730 |
| `rawtovcd` | Standalone C tool: `rawtovcd [-v voltage] rawfile > vcdfile`. With `-v`, converts analog waves to binary using the voltage threshold. | src/rawtovcd.c:265-292 |

**ngspice shared-library mode: absent in 3.4.7.** The source has no `ngSpice_Init`/`sharedspice` reference. `ngspice::` is only a Tcl namespace of raw-data accessors (xschem.tcl:1294-1430).

### 4B.2 Raw file loading (simulation data)

| Feature | Detail | Source |
|---|---|---|
| Reader | ngspice/Xyce raw format with ASCII `Values:` or `Binary:` blocks. `Flags: complex` marks AC data. Multiple plots per file. Each repeated matching plot becomes a new **dataset** (Montecarlo/sweep runs). | save.c:593 (`read_dataset`) |
| Plotname → sim_type | `transient analysis`→tran. `dc transfer characteristic`→dc. `noise spectral density curves`→noise. `operating point`→op. `integrated noise`→op or noise. `ac analysis` / `spectrum` / `sp analysis`→ac. Requested types `sp` and `spectrum` are aliased to `ac`. | save.c:614-690 |
| Complex data | AC data is stored as magnitude and phase (deg). The expressions `re()` and `im()` convert it. | save.c:2036-2043 |
| Sweep window | `raw_read file type sweep1 sweep2` loads only points with sweep1 ≤ x < sweep2 | scheduler.c:4418 |
| Multiple raw files | Up to `MAX_RAW_N`=50 raw/type pairs (xschem.h:286), managed by `extra_rawfile(what)`: 1=load, 2=switch, 3=remove, 4=info, 5=switch back | save.c:1210 |
| Raw bound to hierarchy | The raw file records `schname` and `level`. `sch_waves_loaded()` returns the level, or -1 when the current schematic is not under it. Paths are trimmed by `raw_level` for lookups. | draw.c:2405 |
| Table data | `xschem raw table_read file`, or graph `sim_type=table`. First line holds headers, first column is the sweep. Two empty lines start a new dataset. `#` starts a comment. | save.c:1490 |
| Embedded raw | `xschem embed_rawfile f` base64-encodes a raw file into the selected instance's `spice_data` attribute. `xschem raw_read_from_attr ?sim?` decodes it to a tmp file and loads it. | draw.c:3621, save.c:932 |
| Synthetic vectors | `xschem raw add name ?expr? ?sweepvar?` (expr is RPN). `xschem raw new name type sweepvar start end step` creates a raw. `raw del`, `raw rename`, `raw set node n value ?dset?` edit data. | scheduler.c:4072-4190 |
| Multi-OP as sweep | A raw file with several 1-point `op` datasets is treated as a dc sweep for plotting | draw.c:2575, 2694 |
| UI | Waves menu: External viewer, Clear (`raw_clear`), Op Annotate, Op, Dc, Ac, Tran, Noise, Sp, Spectrum. Each opens a file dialog (`load_raw`→`raw_read file type`). | xschem.tcl:9053-9073, 8739 |

`xschem raw` sub-subcommands (scheduler.c:4072-4190): `read, clear, del, rename, info, new, list, vars, switch, switch_back, datasets, value, loaded, rawfile, sim_type, index, values, pos_at, points, set, add, table_read`. Related top-level subcommands: `raw_read`, `raw_clear`, `raw_query` (alias of `raw`), `raw_read_from_attr`, `embed_rawfile`, `table_read`, `annotate_op`, `add_graph`, `draw_graph`, `cursor`.

### 4B.3 Built-in waveform graphs

A graph is a rectangle (`B`) on **layer 2 (GRIDLAYER)** whose `flags` contains `graph`. `set_rect_flags` (actions.c:766) caches these bits: graph=1, unlocked=2, private_cursor=4, image=1024, unscaled=2048. Create one with Simulation > Graphs > Add waveform graph (`xschem add_graph`, scheduler.c:238). It is placed as an 800×400 box with these defaults: `flags=graph y1=0 y2=2 ypos1=0 ypos2=2 divy=5 subdivy=1 unity=1 x1=0 x2=10e-6 divx=5 subdivx=1 xlabmag=1.0 ylabmag=1.0 node="" color="" dataset=-1 unitx=1 logx=0 logy=0`.

#### Graph attributes (complete list; parsed in `setup_graph_data`, draw.c:3129, and `draw_graph`, draw.c:3993)

| Attr | Type / default | Meaning |
|---|---|---|
| `flags` | `graph[,unlocked][,private_cursor]` | Without `unlocked`, all graphs of the same sim_type share X pan/zoom ("locked"). `private_cursor` stores cursor x positions in this graph's attributes. |
| `node` | quoted, newline-separated list | One wave per line. See the node syntax below. |
| `color` | space-separated layer indices | Color for each node line. Default is layer 4. Clamped to `< cadlayers`. |
| `sweep` | space/newline list | X-axis variable for each node line. The default is raw variable 0 (time, freq, swept source). |
| `x1`,`x2` | eng number, default 0 / 1e-6 | X viewport. With logx these are log10 values. |
| `y1`,`y2` | default 0 / 5 | Y viewport for analog graphs |
| `divx`,`divy` | int, default 5, min 1 | Major divisions |
| `subdivx`,`subdivy` | int, default 0 | Minor ticks |
| `unitx`,`unity` | one of `f p n u m 1 k M G T` | Display scale suffix (`get_unit`, draw.c:2388). unity is forced to 1 when logx is set. |
| `xlabmag`,`ylabmag` | float, default 1.0 | Label size multipliers |
| `logx`,`logy` | `0`/`1` | Log axes |
| `digital` | `0`/`1` | Digital stacked view. In this mode `ypos1`/`ypos2` set the vertical window (default 0/2) and 10 traces fit (`DIG_NWAVES`=0.1, xschem.h:434). Logic 1 is above 80% of y1..y2, logic 0 below 20%. |
| `ypos1`,`ypos2` | float | Digital-mode vertical scroll and zoom |
| `dataset` | int, default -1 = all | Plot only one dataset (run) |
| `mode` | `Line` \| `HistoV` \| `HistoH` | Draw style |
| `legend` | int, default 1 | Show wave labels |
| `hilight_wave` | int, -1 | Index of the wave drawn bold (right-click on a label) |
| `linewidth_mult` | float | Overrides the global `graph_linewidth_mult` (default 1.4) |
| `rainbow` | `0`/`1` | Rainbow coloring of datasets |
| `rawfile` | path (Tcl-`subst`ed, `$netlist_dir` allowed) | Per-graph raw file that overrides the base raw |
| `sim_type` | `dc ac tran op sp spectrum noise constants table` | Analysis to load from `rawfile` |
| `autoload` | `0`/`1` | Reload `rawfile` automatically on redraw |
| `cursor1_x`,`cursor2_x` | number | Cursor a/b positions. Used only with `private_cursor`. |
| `hcursor1_y`,`hcursor2_y` | number | Horizontal cursors A/B. Their presence enables them. |
| `lock` | `true` | Generic lock. A locked graph ignores mouse unless Ctrl is held (callback.c:61). |

`fullxzoom` and `fullyzoom` are `xschem setprop rect 2 n fullxzoom|fullyzoom` actions, not attributes (scheduler.c:5678-5690).

#### Node line syntax (draw.c:4084-4200, doc graphs.html)

`"alias; expr_or_node % dataset rawfile sim_type"`. Every field except the node is optional.

| Form | Meaning |
|---|---|
| `v(out)` / `out` | Plain saved vector. Lookup is case-insensitive via the raw hash. |
| `alias;expr` | Shows `alias` as the label |
| `BUSNAME,b3,b2,b1,b0` (or `;` separator) | Bus. Digital mode shows a hex value. A schematic bus `LDA[12:0]` sent with Alt-G expands to bits. |
| `node%3` | Dataset 3 only |
| `node % 2 $netlist_dir/x.raw tran` | Per-wave raw file and type |
| Any text containing spaces after the alias | **RPN expression**, evaluated by `plot_raw_custom_data` (save.c:1792) |

RPN operators (save.c:1825-1894):

| Arity | Operators |
|---|---|
| 3-arg | `?` (X cond Y ?) |
| 2-arg | `+ - * / ** == != < > <= >= max() min() exch() ravg() del() re() im()` |
| 1-arg | `sin() cos() tan() asin() acos() atan() sinh() cosh() tanh() asinh() acosh() atanh() cph() sqrt() sgn() abs() exp() ln() log10() idx() db20() avg() prev() deriv() deriv0() integ() dup()` |

Operands are vector names or numbers. The stack holds `STACKMAX` entries, and overflow aborts evaluation.

#### Graph mouse and key interactions (callback.c:414-1360)

These are active while the pointer is inside the graph area. If `graph_use_ctrl_key=1`, every one of them needs Ctrl (callback.c:423).

| Input | Action |
|---|---|
| Click near the graph border | Select the graph rectangle (move, resize or delete it like a rect) |
| Double-click inside | Open the graph dialog (`graph_edit_properties`, xschem.tcl:3308). Double-click on a wave label changes its color instead. |
| Right-click on a wave label | Toggle bold (`hilight_wave`) |
| LMB drag in plot area | Pan X (all locked graphs follow) |
| LMB drag left of the Y axis | Pan Y |
| LMB drag in the bottom (X-axis) strip | Absolute X positioning |
| RMB drag in plot area / left of Y axis | Zoom X / Y to the dragged range |
| Wheel / Shift-wheel | Pan or zoom. Left of the Y axis this works on Y; in digital mode it moves ypos1/ypos2. |
| Left/Right arrows | Pan X. Left of the Y axis: zoom Y. |
| Up/Down arrows | Zoom X in or out around the mouse |
| `f` | Full X zoom. Left of the Y axis: full Y zoom. |
| `a` / `b` | Toggle vertical cursor a / b at the mouse X. Drag a cursor to move it. RMB near a cursor sets it numerically. |
| `A` / `B` | Toggle horizontal cursors (hcursor1/2). Drag to move, RMB to enter a value. |
| `s` | Swap cursors a and b |
| `m` | Toggle the measurement tooltip, which follows the mouse with `y=`/`x=` (`graph_show_measure`, xschem.tcl:3875) |
| `t` | Track the dataset closest to the mouse (`find_closest_wave`) |
| Alt-G on a net | Add the net to the selected or master graph |

Readouts: with both a and b shown, the x difference (and 1/Δx) is drawn. With both A and B shown, the y difference is drawn. Wave labels show the value at cursor a (draw.c:3324-3450).

#### Graph dialog (summary; full field list belongs in section 3)

| Area | Contents |
|---|---|
| Left | Signal search (regex), Add button, signal listbox |
| Right | Auto load, Sim type combobox, Raw file entry with browse button, editable colored node text |
| Top row | Legend, Bus, Incr. sort, Rainbow col., line width, Dataset, Priv. Cursor, Unlock. X axis, Digital |
| Second row | X units, Y units, X div., Y div., X subdiv., Y subdiv., Sweep, Mode (Line/HistoV/HistoH) |
| Third row | X min/max, Y min/max, X/Y label magnitude, Log X, Log Y |
| Bottom | OK, Apply, Cancel, color radio buttons (layers 4..cadlayers-1), AUTO SET (default colors) |

All edits push one undo step per dialog session (`graph_push_undo`, xschem.tcl:2846).

#### Related Tcl vars

`graph_use_ctrl_key`, `hide_empty_graphs`, `auto_hilight_graph_nodes` (highlight plotted nets in the schematic with the wave color), `live_cursor2_backannotate`, `graph_linewidth_mult`, `cursor_2_hook` (Tcl evaluated whenever cursor b moves, callback.c:276), `graph_dialog_default_geometry`. GUI state variables: `graph_bus graph_sort graph_logx graph_logy graph_rainbow graph_digital graph_legend graph_autoload graph_unlocked graph_private_cursor graph_sel_color graph_sel_wave graph_selected graph_schname graph_change_done`.

`xschem set cursor1_x|cursor2_x <v>` and `xschem cursor n 0|1` drive cursors from scripts (scheduler.c:742).

### 4B.4 Back-annotation (operating point / cursor b)

| Feature | Detail | Source |
|---|---|---|
| `xschem annotate_op ?raw? ?level?` | Loads `$netlist_dir/<cell>.raw`, or the given file, as type `op`. For Xyce it falls back to a 1-point `dc`. Sets `live_cursor2_backannotate 1` and fills the Tcl array `ngspice::ngspice_data(<var>)` with `%.4g` values (`update_op`), then redraws. `level` binds the raw file to an upper hierarchy level. | scheduler.c:329, save.c:1446 |
| Live annotation at cursor b | When `live_cursor2_backannotate` is set, moving cursor b sets `raw->annot_p` to the matching point. Every `@spice_get_*` text then shows the value at that time or frequency. | callback.c:274 |
| Menu | Simulation > Graphs: Auto highlight plotted nets, Add waveform graph, Add waveform reload launcher (places `launcher.sym` with `tclcommand="xschem raw_read $netlist_dir/<cell>.raw tran"`), Annotate Operating Point into schematic, Live annotate probes with 'b' cursor, Hide graphs if no spice data loaded. | xschem.tcl:9367-9391 |

Annotation tokens, usable in symbol texts and `format` (token.c):

| Token | Meaning | Source |
|---|---|---|
| `@spice_get_voltage` | Voltage of the net on pin 0 | token.c:4095 |
| `@spice_get_voltage(...)` | Voltage of the given net | token.c:4183 |
| `@#pin:spice_get_voltage` / `@#n:spice_get_voltage` | Voltage on the named or numbered pin | token.c:3617 |
| `@spice_get_diff_voltage` | Difference of pins 0 and 1 (`spice_probe_vdiff.sym`) | token.c:4357 |
| `@spice_get_current`, `@spice_get_current_<param>` (e.g. `_ibs`, `_ie`) | Device current | token.c:4257 |
| `@spice_get_modelparam_<p>`, `@spice_get_modelvoltage_<p>` | Parsed. Only the current forms are processed. | token.c:3925 |
| `@spice_get_node <spice_node>` | Arbitrary vector, space-delimited. One per string. Result is in engineering notation. | token.c:3733 |
| Tcl `ngspice::get_voltage n`, `get_current n`, `get_diff_voltage n m`, `get_node n` | Accessors used inside `tcleval(...)` texts | xschem.tcl:1300-1430 |
| `@savecurrent` | Expands to `\n.save I( ?1 <inst> )` when `savecurrent=true` (ammeter, vsource) | token.c:2257 |

Probe symbols in xschem_library/devices:

| Symbol | Shows |
|---|---|
| `ngspice_probe.sym` | Net voltage, via `@spice_get_voltage` |
| `ngspice_get_value.sym` | `tcleval([ngspice::get_node @node])` |
| `ngspice_get_expr.sym` | Arbitrary Tcl expression |
| `device_param_probe.sym` | Device parameter |
| `spice_probe.sym`, `spice_probe_vdiff.sym` | Voltage / differential voltage. vdiff also emits `.save v(p) v(m)`. |
| `ammeter.sym` | Current. Emits a 0 V source plus `.save I()`. |
| `scope_ammeter.sym` | `tcleval(@spice_get_node [xschem get_fqdevice @device])` |
| `lab_show.sym` | `@#0:net_name` |

The data is not persistent; you re-run annotate after reload (tutorial_ngspice_backannotation.html). Legacy Tcl raw reader: `ngspice::read_raw`. HSPICE log annotation lives in `hspice_backannotate.tcl` (`read_hspice_log`, `annotate`).

#### Built-in logic propagation (pseudo digital simulation)

`xschem logic_set n ?num?` and `logic_set_net net n` work on keys `0`/`1`/`2`/`3`/`4`, which set the selected net to 0, 1, X or Z, or toggle it. Values propagate through symbols with `function<N>` RPN attributes and `clock`/`goto` pin attributes. Operators: `d r x ~ z R M m | & ^`, with `L H Z U` literals (hilight.c:1477-1600, 1645-1817).

### 4B.5 Export and printing

| Format | Command / menu | Implementation | Source |
|---|---|---|---|
| PNG | `xschem print png file [w h] [x1 y1 x2 y2]`. File > Image export > PNG Export. Ctrl-Shift-`*`. CLI `--png`. | Renders the drawing into an offscreen pixmap (grid off), then `cairo_surface_write_to_png`. Without cairo it writes XPM and runs `$to_png` (default `gm convert`). With no explicit area and a selection present, the selection bbox is used; otherwise full zoom. | draw.c:97, scheduler.c:3887 |
| SVG | `xschem print svg ...`. Alt-Shift-`*`. CLI `--svg`. | `svg_draw`: native vector output with CSS classes per layer. Text is real `<text>` when `text_svg=1` (globals.c:182). Embedded images are base64. Graphs are rasterized to embedded PNG (`svg_embedded_graph`). | svgdraw.c:895, draw.c:4980 |
| PS/EPS/PDF | `xschem print pdf\|ps\|eps file [w h] [x1 y1 x2 y2]`. `*` key. CLI `-p/--postscript/--pdf`. | `create_ps` writes PostScript to a tmp file, then `convert_to_pdf` runs `$to_pdf` (default `ps2pdf`) when the destination ends in `.pdf`. Paper from `ps_paper_size` (default `{a4 842 595}`). Orientation follows the bbox aspect. Optional page title (`ps_page_title`). Graphs and images are embedded as bitmaps (`ps_embedded_graph`). `w h` = 0 sets the paper to the bbox. | psprint.c:1201, 1599, xschem.tcl:1045 |
| PS/PDF full | `xschem print pdf_full\|ps_full` | Full-zoom page | scheduler.c:3883 |
| Hierarchical PDF | `xschem hier_psprint ?file?`. File > Image export > Hierarchical PDF/PS Export. | Multi-page document: top level plus each distinct subcircuit schematic once, via hierarchy traversal | spice_netlist.c:46, scheduler.c:2255 |
| EPS selection | File > Image export > EPS Selection Export | `print eps` | xschem.tcl:8874 |
| Netlists | SPICE/Verilog/VHDL/tEDAx | See section 4A | |

| Option var | Default | Effect |
|---|---|---|
| `color_ps` (`-c`) | 1 | Color output. 0 gives B/W: black on white, or inverted when `dark_colorscheme` is set. |
| `transparent_svg` | 0 | No background in SVG |
| `ps_paper_size` | `{a4 842 595}` | Paper name, width and height in pt |
| `ps_page_title` | 1 | Page title in the top-left corner |
| `to_pdf` / `to_png` | `ps2pdf` / `gm convert` | External converters |
| `svg_font_name` / `cairo_font_name` | `Sans-Serif` | Fonts |
| `svg_colors` / `ps_colors` | Derived from the layer colors | Per-layer color lists |
| `--plotfile f` | n/a | Output file for CLI print |
| `dark_colorscheme` | 1 | Affects the background of exported graphs |
| `xschem grabscreen` / Print Screen | n/a | Grab a screen area into the schematic as an embedded image (`flags=image,unscaled alpha=0.8 image_data=<b64>`) | 

Grab screen sources: draw.c:178, 286.

### 4B.6 Undo / redo

| Aspect | Detail | Source |
|---|---|---|
| Modes | `undo_type` = `disk` (default, xschem.tcl:9944) or `memory`. `xschem undo_type disk\|memory` switches by rebinding the `push_undo`/`pop_undo`/`delete_undo`/`clear_undo` function pointers and discarding the other stack. | scheduler.c:6289 |
| Disk | Each step is a full `write_xschem_file` piped through `gzip --fast` into `$XSCHEM_TMP_DIR/xschem_undo_XXXX/undo<N>`. Falls back to fork+pipe or uncompressed when popen is unavailable. | save.c:3713 |
| Memory | Each slot deep-copies every line, rect, poly, arc, wire, text, instance and symbol, plus the G/V/S/K/E global property strings | in_memory_undo.c:263 |
| Depth | `MAX_UNDO`=80, ring buffer (xschem.h:272) | |
| Redo | Supported (`pop_undo(1)`). The first undo pushes the current state so redo can restore it. Mode 2 peeks without moving. Mode 4 undoes without pushing. | save.c:3787-3830 |
| Scope | Per context (tab/window). **Cleared on every schematic load and on descend/ascend** (`load_schematic(..., reset_undo)`), so history does not survive hierarchy navigation. | save.c:3513, actions.c:2480 |
| Script | `xschem undo ?redo? ?set_modify?`, `xschem redo`, `xschem push_undo`, `xschem set no_undo 1`. Keys: `u` undo, `U` redo. | scheduler.c:4062, 6272 |

### 4B.7 Clipboard, merge, cross-window copy

| Feature | Detail | Source |
|---|---|---|
| Clipboard file | `~/.xschem/.clipboard.sch`, a plain .sch fragment. The `G {mx my}` record holds the anchor mouse position. **Shared across all xschem processes**; there is no OS clipboard for objects. | save.c:5358, xinit.c:2631 |
| Copy / cut / paste | Ctrl-C (`xschem copy`) and Ctrl-X (`xschem cut`) save the selection. Ctrl-V (`xschem paste ?x y?`) merges it and enters move mode, or places at the given offset. | scheduler.c:664, 770, 3627 |
| Merge | `b` / File > Merge (`xschem merge ?file?`) inserts another .sch/.sym as movable objects | scheduler.c:3277, paste.c:326 |
| Cross-window drag | Starting a copy writes `~/.xschem/.selection.sch` (`save_selection(1)`). When the pointer enters another xschem window with nothing selected, that window auto-merges the file. The source aborts its copy once the file is consumed. | move.c:630, callback.c:2311 |
| Text clipboard | Tk `clipboard` is used only for text widgets and the tab context menu "copy filename" | xschem.tcl:7669 |
| Generated fragments | Tcl helpers write `.clipboard.sch` and merge it (e.g. labels or pins from a pin list) | xschem.tcl:5249-5386 |

### 4B.8 Launchers and hyperlinks

| Feature | Detail | Source |
|---|---|---|
| Trigger | Ctrl-LMB click on the object (with no drag), or Ctrl-H with exactly one object selected. Works on any object type, not only instances. | callback.c:4271-4290, 2870 |
| `launcher()` | Reads `url`, `program` and `tclcommand` from the selected object. If `url` or `program` is set it calls Tcl `launcher url program`, which runs `exec [subst program] [subst url] &`. The default program is `launcher_default_program` (`xdg-open`; Windows `start`). Otherwise it evaluates `tclcommand` at global scope, and errors produce an alert. | actions.c:1805, xschem.tcl:7400 |
| `launcher.sym` | `type=launcher`, `format="** @descr : @url"`, ignored by all netlisters. Template `name=h1 descr=google url=www.google.com`. Draws an arrow polygon plus `@descr`. | xschem_library/devices/launcher.sym |
| Uses | Reload raw (`xschem raw_read ...`), `xschem annotate_op`, open docs, open datasheets. Help menu entries use `launcher file://...`. | scheduler.c:130 |

### 4B.9 `tcleval(...)` attributes

| Context | Rule | Source |
|---|---|---|
| Whole-string wrap | If an attribute value or translated text *starts* with `tcleval(`, the content (with `\}` unescaped) goes to `tclpropeval2`. That proc sets `path` (hierarchy path trimmed by raw_level; for SPICE each component gets an `x` prefix), `sch_basename` and `netlist_type`, then runs `uplevel #0 subst {...}`. On error it returns `?\n`, and `debug_tcleval` prints the error. | token.c:59 (`tcl_hook2`), xschem.tcl:5991 |
| Where applied | End of `translate()` (instance texts, format output), `translate3()` (symbol texts and floaters), symbol `format` in `print_spice_subckt`, symbol and instance names (`schematic=`/`sym` paths may be `tcleval`), `spice_sym_def`, `template` and parent props in the netlisters | token.c:4653, 4912, 2017, spice_netlist.c:86-641, save.c:3477 |
| `@tcleval(<script>)` token inside `format` | `tclpropeval` (`subst` in proc scope) with `symname` and `instname` available. Spaces and quotes must be escaped. | token.c:1147, 2368, 2843, 3105; xschem.tcl:5977 |
| `expr(...)` | After translation, `expr(` is evaluated by the built-in `eval_expr` (yacc, eval_expr.y) | token.c:4657 |
| Engineering notation | Numbers inside spice_get results are rendered as `2m`, `3u`, … | token.c:3939 |

### 4B.10 Search

| Feature | Detail | Source |
|---|---|---|
| Dialog | Ctrl-F (`property_search`): Token entry, Value entry, Exact search checkbox (otherwise regex), Match case, radio Highlight / Select / Unselect, OK, Cancel. Loops until something is found or Cancel. | xschem.tcl:5896 |
| `xschem search regex\|exact sel tok val ?match_case?` | `sel`: 0 = highlight, 1 = select, -1 = unselect. `tok` is an attribute name, or `propstring` (the whole instance prop string), `cell::name` (symbol name), `cell::propstring`, or `cell::<attr>` (a symbol attribute). Searches instances and wires (`lab`); lines and rects only when selecting. Regex is POSIX extended. `searchmenu` is the same but pops "Not found." | scheduler.c:4990-5045, hilight.c:719 |
| Replace | **No find/replace UI.** Scripted via `xschem setprop`, `xschem replace_symbol inst newsym`, `xschem set_different_tok`. | scheduler.c:4644 |
| Duplicate refdes | `#` highlights duplicates. Ctrl-`#` renames them. | keys.help |

### 4B.11 Grid, snap, zoom

| Item | Value / behaviour | Source |
|---|---|---|
| Snap | `cadsnap`, default 10 (`CADSNAP`). `g` halves, `G` doubles, Ctrl-G sets. Status bar field 3 turns OrangeRed when it differs from the default. `xschem set cadsnap v`. | xschem.h:159, actions.c:294 |
| Grid | `cadgrid`, default 20. Drawn points are multiplied by 2 until the on-screen spacing is ≥ 10 px (`CADGRIDTHRESHOLD`/`CADGRIDMULTIPLY`). Status bar field 5 changes color like snap. Options: `draw_grid` (`%` toggles), `big_grid_points`, `grid_point_size`, `draw_grid_axes`. | draw.c:1097-1106, actions.c:316 |
| Align | Alt-U / `xschem align`: snap the selection to the grid. `xschem round_schematic_to_grid`. | scheduler.c:307, save.c:5280 |
| Zoom step | `CADZOOMSTEP` 1.2. Limits `CADMINZOOM` 0.005 and `CADMAXZOOM` 10000. Wheel zooms around the mouse. `unzoom_nodrift` keeps the mouse fixed when zooming out. | xschem.h:166-169, actions.c:2856-2890 |
| Zoom commands | `f` full (`zoom_full`, lower-left aligned unless `zoom_full_center`). Ctrl-Shift-F zoom to selection. `z` zoom box. RMB drag zoom box. Shift-Z in. Ctrl-Z out. `xschem zoom_hilighted`. `zoom_box x1 y1 x2 y2 factor`. | actions.c:2809, scheduler.c:6536-6620 |
| Pan | MMB drag, Space, Shift-P, arrow keys (`CADMOVESTEP` 200) | |
| Line width | `change_lw` scales line width with zoom. Ctrl-`+`/Ctrl-`-`/Alt-`-` change it, `_` toggles. | |
| Crosshair | `draw_crosshair` (Alt-X), `crosshair_layer` (8), `crosshair_size` (0 = full window). `snap_cursor`/`snap_cursor_size` draw a snapped cursor marker. | callback.c:1585, 1461 |

### 4B.12 Windows, tabs, contexts

| Item | Detail | Source |
|---|---|---|
| Limit | `MAX_NEW_WINDOWS`=20 per process, main window included | xschem.h:145 |
| Modes | `tabbed_interface` 1 (default): one toplevel with a tab bar. 0: separate toplevels `.x1`, `.x2`, … The main win_path is `.drw`; the others are `.xN.drw`. | xschem.tcl:9947 |
| C context | Each window or tab owns an `Xschem_ctx` (`xctx`) holding the schematic, undo, raw files, highlights and zoom. Switching swaps the pointer (`save_xctx[]`). | xinit.c:41, 1504-2033 |
| Tcl context | `save_ctx`/`restore_ctx` copy about 180 globals (`tctx::global_list`) and arrays (`sim`, `dircolor`, `enable_layer`, `replace_key`, `ngspice::ngspice_data`) into `tctx::<win_path>` | xschem.tcl:8313-8425 |
| Commands | `xschem new_schematic create\|destroy\|destroy_all\|switch win_path ?file?`, `xschem switch`, `xschem tab_list`, `xschem load_new_window`, `xschem schematic_in_new_window ?new_process?`, `xschem swap_windows`, `xschem copy_hierarchy to from`, `xschem preview_window` | scheduler.c:3525, 5930, 5971 |
| Keys | Ctrl-Left/Right previous/next tab. Alt-E opens the selected schematic in a new window. Alt-Shift-E opens it in a new process. Alt-I opens the symbol. `x` starts a new xschem process (`new_xschem_process`). | keys.help, actions.c:423 |
| Tab context menu | Open dir, copy filename, terminal in file dir or sim dir, edit file, netlist, … | xschem.tcl:7645 |

### 4B.13 File change detection, save safety

| Item | Detail | Source |
|---|---|---|
| mtime check | On load, `time_last_modify = st_mtime`. On save to the same name, a changed mtime triggers "Has been changed since opening. Save anyway?". `save()` forces a save when the on-disk mtime differs. | save.c:3416-3437, actions.c:554 |
| Reload | Alt-S / `xschem reload` (asks if modified) | scheduler.c:4602 |
| Ask save | `ask_save_optional`. `no_ask_save` skips the confirm on Ctrl-S. | xschem.tcl:9906 |
| Autosave / backup | **None** (no autosave timer and no .bak files) | n/a |
| Header/license | Shift-B edits the file header text (`xschem set header_text`) | xschem.tcl:1404 |

### 4B.14 Remote control / server mode

| Item | Detail | Source |
|---|---|---|
| xschem server | Enabled by setting `xschem_listen_port` (rc var or `--tcp_port`, which must be ≥1024). Each connection's lines are evaluated at global scope with `puts` captured; the result is written back and the socket closed. `setup_tcp_xschem ?port?` rebinds the port (0 = auto). Typical port: 2021. | xschem.tcl:2378, 2432, 9634, xinit.c:2674, doc xschem_remote.html |
| Console | Tcl prompt on stdin with optional tclreadline (`-r/--no_readline`, `--pipe`). `-b/--detach` detaches. | options.c |
| bespice server | See 4B.1 | |

### 4B.15 Startup hooks, plugins and extensibility

Execution order (xinit.c):

| # | Step | Source |
|---|---|---|
| 1 | `--preinit` Tcl | xinit.c:2546 |
| 2 | Source xschemrc: the system rc in `XSCHEM_SHAREDIR`, then `--rcfile`, or `./xschemrc`, or `~/.xschem/xschemrc`. Skipped with `-i`. | xinit.c:2552-2613 |
| 3 | `--tcl` | xinit.c:2670 |
| 4 | Set the TCP port | xinit.c:2674 |
| 5 | Source `xschem.tcl`, which applies `set_ne` defaults | xinit.c:2686 |
| 6 | Load file / netlist / print / simulate / waves per CLI flags | xinit.c:2954-3075 |
| 7 | `source_user_tcl_files` (`tcl_files` list) | xinit.c:3082 |
| 8 | `eval_user_startup_commands` (`user_startup_commands`), also run for each new window | xinit.c:3083, 1678 |
| 9 | `--script file` | xinit.c:3086 |
| 10 | `--command` | xinit.c:3102 |
| 11 | `eval_postinit_commands` (`postinit_commands`) | xinit.c:3107 |

| Hook / extension | Detail |
|---|---|
| `cursor_2_hook` | Tcl evaluated when cursor b moves (callback.c:276) |
| `execute(callback)` / `xschem simulate callback` | End-of-job callback |
| Custom menu | `add_custom_menu.tcl` example: `$topwin.menubar insert Netlist cascade ...` |
| Custom toolbar button | `add_custom_button.tcl` example: append to `toolbar_list` with a base64 GIF |
| `tcleval()` / `@tcleval` | Attribute-level scripting (4B.9) |
| `xschem callback` | Inject synthetic events (scheduler.c:436) |
| `replace_key` array | Key remapping, saved per context |
| Format-level hooks | `@tcleval` inside `format` |

There is no binary plugin ABI.

#### Command-line options (options.c `check_opt`; help text in src/xschem.help) — 35 options

| Short | Long | Arg | Effect |
|---|---|---|---|
| `-h` | `--help` | | Print help and quit |
| `-b` | `--detach` | | Detach from the console |
| `-n` | `--netlist` | | Netlist the given cell |
| `-v` | `--version` | | Print the version and quit |
| `-V` | `--vhdl` | | Netlist type VHDL |
| `-S` | `--simulate` | | Run a simulation after load |
| `-w` | `--verilog` | | Netlist type Verilog |
| | `--tcl` | cmd | Tcl after the rc file, before xschem.tcl |
| | `--preinit` | cmd | Tcl before the rc file |
| | `--script` | file | Source a Tcl file after startup |
| | `--command` | cmd | Tcl after startup completes |
| | `--diff` | file | Visual diff against a file (`sch_to_compare`) |
| | `--tcp_port` | n | Listen port (≥1024) |
| `-i` | `--no_rcload` | | Skip xschemrc |
| `-o` | `--netlist_path` | path | Netlist directory |
| `-N` | `--netlist_filename` | file | Top netlist file name |
| `-t` | `--tedax` | | Netlist type tEDAx |
| `-s` | `--spice` | | Netlist type SPICE |
| `-y` | `--symbol` | | Netlist type SYMBOL |
| `-x` | `--no_x` | | No GUI |
| `-z` | `--rainbow` | | Rainbow layer colors |
| `-W` | `--waves` | | Open waves |
| `-f` | `--flat_netlist` | | Flat SPICE netlist |
| `-r` | `--no_readline` | | No tclreadline |
| | `--pipe` | | Same as `-r`; keeps stdio open for pipes |
| `-c` | `--color_ps` | | Color PostScript |
| | `--plotfile` | file | Output file for print |
| | `--rcfile` | file | Alternate rc file |
| `-p` | `--postscript` | | Export PDF |
| | `--pdf` | | Export PDF |
| | `--png` | | Export PNG |
| | `--svg` | | Export SVG |
| `-q` | `--quit` | | Quit after the actions |
| `-l` | `--log` | file | Log stderr to a file |
| `-d` | `--debug` | n | Debug level (+ for C, − for Tcl) |

Long options accept `--opt=value` or `--opt value`. Short options can be combined (`-nqx`). The first non-option argument is the file to open. Remaining ones are kept in argv (options.c:155-266).

### 4B.16 Misc editing infrastructure

| Feature | Detail | Source |
|---|---|---|
| Text rendering | Cairo toy font `cairo_font_name`, scaled by `cairo_font_scale` and `cairo_font_line_spacing`. Text attributes: `font`, `weight=bold`, `slant=italic\|oblique`, `hcenter`, `vcenter`, `layer`, `hide=true\|instance`, `name`/`floater` (floater text bound to an instance). Tabs expand to `tabstop` (8). | actions.c:895, draw.c:392 |
| Zoom-dependent text | Text is skipped when the on-screen size is < 3 px or > 1600 px (size = xscale·52·font_scale·zoom) | draw.c:407-408 |
| Hidden texts | `hide=true` texts are drawn only when `show_hidden_texts` is set (View menu). The Waves > Op Annotate action turns it on. | draw.c:835, 5243 |
| Symbol texts | `sym_txt` (Ctrl-B) toggles all symbol texts. Instance `hide_texts=true` hides them per instance. Instance `hide=true` shows only the bbox. `hide_symbols` (Alt-B) shows bboxes only. | actions.c:851-854, draw.c:825 |
| Highlight colors | Each highlight takes the next active layer (`incr_hilight`, default 1). Color = `active_layer[value % n_active_layers]`. Negative values select a fixed layer. Options: `en_hilight_conn_inst` (also color connected instances), symbol `highlight=true`. | hilight.c:327-344 |
| Selection by area | LMB drag left→right selects objects fully **inside**. Right→left selects objects **touched** (when `select_touch`=1). `incremental_select` shows the selection live. Ctrl-drag selects for stretch. Alt-drag / Shift-D deselects. `unselect_partial_sel_wires`. | actions.c:3818, callback.c:2390 |
| Move / copy | `m` move, `c` copy. Ctrl-M stretches attached wires. Alt-M / Shift-M / Alt-C insert wires where touching pins separate ("kissing"). Ctrl-Shift-M combines both. `enable_stretch` swaps the defaults. `y` toggles stretch. `h` / `v` constrain to horizontal/vertical (`constr_mv`). Arrows nudge during a move. | keys.help, scheduler.c:3333, 691 |
| Rotate / flip | Shift-R rotates the selection 90° CW around the mouse point. Alt-R rotates each object around its own anchor. Shift-F / Alt-F flip H. Shift-V / Alt-V flip V. All also work mid-move. `rot` 0..3, `flip` 0/1. `xschem rotate\|flip ?x0 y0?`. | scheduler.c:1195, 4832 |
| Object order | Shift-S / `xschem change_elem_order n` | editprop.c:1773 |
| Attach labels | Shift-H / `attach_labels_to_inst` dialog: Prefix, use wire labels (`lab_wire.sym` instead of `lab_pin.sym`), Do all, Rotated Text, use prefix | actions.c:1273, xschem.tcl:6032 |
| Labels from highlights | Alt-J / Alt-Shift-J (with `i` prefix). Ctrl-J creates ports from highlighted nets. Alt-L / Alt-Shift-L place `lab_pin` / `lab_wire`. Alt-H creates symbol pins from schematic pins. | keys.help |
| Wire cleanup | `&` join, break and collapse wires (`trim_wires`). `!` breaks wires at pins (`break_wires`). `autotrim_wires` trims automatically. Alt-RMB cuts a wire at a snapped point; Alt-Shift-RMB cuts it unsnapped (`wire_cut`). | check.c:151, 534, scheduler.c:6497 |
| Connectivity selection | Shift-RMB selects connected nets. Ctrl-RMB stops at junctions. Alt-K. `xschem connected_nets 0..3`. | scheduler.c:653 |
| Reset props | `xschem reset_inst_prop inst` resets from the symbol template. Shift-T toggles `*_ignore` on the selection. | scheduler.c:4719 |
| Overlap check | `xschem warning_overlapped_symbols ?sel?` | scheduler.c:6403 |
| Locking | `lock=true` on any object makes it unselectable by click (findnet.c, select.c) | findnet.c:46-478 |
| Infowindow / log | `show_infowindow`, `show_infowindow_after_netlist` (`onerror`). `xschem log f` redirects stderr. | xschem.tcl:9904 |
| Fullscreen | `\` cycles: fullscreen with menus → fullscreen → normal (`xschem fullscreen`) | xinit.c:1109 |
| Colorscheme | Shift-O toggles dark/light (`toggle_colorscheme`). `dark_gui_colorscheme` styles the Tk widgets. | scheduler.c:6157 |
| Dim background | `enable_dim_bg`, `dim_bg`, `dim_value` | xschem.tcl:10011 |

## 4C. Scripting API and configuration

Sources: `src/scheduler.c` (the `xschem` Tcl command, `int xschem(...)`), `src/xschemrc` (template rc, installed as `$XSCHEM_SHAREDIR/xschemrc`), `src/xschem.tcl` (defaults via `set_ne` = "set if not exists"), `src/xinit.c` (startup / rc search), `src/xschem.help` (CLI), `doc/xschem_man/developer_info.html` (generated copy of the scheduler.c comments; no extra subcommands there).

### 4C.1 `xschem` Tcl command: dispatch model

- One C command, `xschem <subcmd> args...`, registered at `xinit.c:2650` (`Tcl_CreateCommand(interp, "xschem", ...)`). Dispatch is `switch(argv[1][0])` then `strcmp` chains, alphabetically sorted (`scheduler.c:166`).
- Operates on the **current context** `xctx` (current tab/window). Switch with `xschem switch` / `xschem new_schematic switch`.
- Unknown subcommand -> error `xschem <cmd>: invalid command.` (`scheduler.c:6629`).
- Object references: instances accept name **or** index; wires/texts by index; rect/line/poly/arc by `(layer, index)`.
- 216 names = 214 documented subcommands + 2 aliases (`raw_query` = `raw`, `searchmenu` = `search` with a "Not found" message box) (`scheduler.c:4191`, `5011`).
- The GUI itself is ~260 Tcl procs in `xschem.tcl` that call these subcommands; menus/keys are thin wrappers, so this table is effectively the editor's command set.

### 4C.2 Subcommand table (all 214, scheduler.c order; last column = `scheduler.c` line)

| subcommand | args | description | line |
|---|---|---|---|
| `abort_operation` |  | Resets UI state, unselect all and abort any pending operation | 172 |
| `add_symbol_pin` | `[x y name dir [draw]]` | Add a symbol pin (pin rect on PINLAYER + line + label text) at x,y with dir in\|out\|inout; no args = GUI placement | 184 |
| `add_graph` |  | Start a GUI placement of a graph object | 238 |
| `add_image` |  | Ask user to choose a png/jpg file and start a GUI placement of the image | 275 |
| `align` |  | Align currently selected objects to current snap setting | 307 |
| `annotate_op` | `[raw_file] [level]` | Back-annotate operating-point voltages/currents from raw file (default <sch>.raw) into schematic probes; `level` = hier level the raw refers to | 329 |
| `arc` |  | Start GUI arc placement (3 non-aligned clicks) | 373 |
| `attach_labels` |  | Attach net labels to selected component(s) instance(s) | 382 |
| `bbox` | `begin\|end` | Start/end bounding box calculation: parameter is either 'begin' or 'end' | 393 |
| `break_wires` | `[remove]` | Break wires at selected instance pins; `1` deletes wire pieces fully inside selected instances | 414 |
| `build_colors` |  | Rebuild color palette using values of tcl vars dim_value and dim_bg | 425 |
| `callback` | `win_path event mx my key button aux state` | Invoke the callback event dispatcher with a software event | 436 |
| `case_insensitive` | `1\|0` | Case-insensitive symbol lookup (only for case-insensitive filesystems) | 448 |
| `change_elem_order` | `n` | Move selected object to index n in its object array (affects netlist order) | 466 |
| `change_sch_path` | `n [draw]` | When inside a vector instance, switch to instance number n without pop/push | 481 |
| `check_symbols` |  | List all used symbols in current schematic and warn if some symbol is newer | 494 |
| `check_unique_names` | `[1\|0]` | Highlight instances with duplicate `name`; `1` renames duplicates | 524 |
| `closest_object` |  | Return `type layer n` of object closest to mouse (type = wire\|text\|line\|poly\|rect\|arc\|inst) | 546 |
| `circle` |  | Start GUI circle placement (3 non-aligned clicks) | 574 |
| `clear` | `[force] [symbol\|schematic]` | Clear window, reset hierarchy, purge symbols; `force` skips save prompt; arg picks new-window type | 586 |
| `clear_drawing` |  | Clears drawing but does not purge symbols | 601 |
| `color_dim` | `value` | Dim colors or brite colors depending on value parameter: -5 <= value <= 5 | 613 |
| `compare_schematics` | `[sch_file]` | Diff current schematic against another file (highlight differences); prompt if no file | 630 |
| `connected_nets` | `[0\|1\|2\|3]` | Select nets/labels connected to selected instance; 1 = stop at junctions, 2 = only directly attached wires, 3 = both | 653 |
| `copy` |  | Copy selection to clipboard | 664 |
| `copy_hierarchy` | `to from` | Copy hierarchy state between windows/tabs, e.g. `.drw .x1.drw` | 676 |
| `copy_objects` | `[dx dy] [kissing] [stretch]` | Copy selection by dx,dy or start GUI copy; `kissing` adds wires at touching pins, `stretch` stretches attached wires | 691 |
| `count_items` | `string separator quoting_chars` | Debug: count fields in string | 719 |
| `create_plot_cmd` |  | Write xplot file in netlist dir listing highlighted nodes for the selected viewer (bespice, gaw, ngspice) | 730 |
| `cursor` | `n e` | Show/hide graph cursor n (1=a, 2=b); position via `xschem set cursorN_x` | 742 |
| `cut` |  | Cut selection to clipboard | 770 |
| `debug` | `n` | Set C debug level (0 = off) | 784 |
| `delete` |  | Delete selection | 795 |
| `delete_files` |  | Bring up a file selector the user can use to delete files | 804 |
| `descend` | `[n] [notitle]` | Push into selected instance schematic; n = vector instance index (1 = leftmost, -1 = rightmost) | 815 |
| `descend_symbol` |  | Descend into the symbol view of selected component instance | 837 |
| `destroy_all` | `[force]` | Close all extra windows/tabs; returns count remaining | 848 |
| `display_hilights` | `[nets\|instances]` | Print list of highlighted nets/labels/pins/instances | 864 |
| `draw_graph` | `[n] [flags]` | Redraw graph rectangle n with optional flags bitmask | 882 |
| `draw_hilight_net` | `[1\|0]` | Redraw only highlight colors (to window or back buffer) | 906 |
| `drc_check` | `[i]` | Run instance DRC rule check (symbol `drc` attr) on instance i or all | 920 |
| `edit_file` |  | Open .sch (or selected instance .sym) in text editor | 936 |
| `edit_prop` |  | Open attribute editor for selection or global schematic/symbol attrs | 955 |
| `edit_vi_prop` |  | Same as edit_prop but in external `$editor` | 966 |
| `embed_rawfile` | `raw_file` | Embed raw file base64-encoded as `spice_data` attr of selected element | 977 |
| `enable_layers` |  | Enable/disable layers depending on tcl array variable enable_layer() | 992 |
| `escape_chars` | `source [charset]` | Backslash-escape Tcl special chars (or chars in charset) | 1002 |
| `eval_expr` | `str` | Debug: evaluate arithmetic expression | 1014 |
| `exit` | `[exit_code] [closewindow] [force]` | Exit/close window; asks if modified unless `force`; returns remaining windows | 1028 |
| `expandlabel` | `lab` | Expand bus/vector label, e.g. `{2*A[3:0]}` -> `A[3],A[2],...  8` (last = bit count) | 1117 |
| `fill_reset` | `[nodraw]` | Reload fill patterns from Tcl `pixdata(n)` | 1141 |
| `fill_type` | `n solid\|stipple\|empty [nodraw]` | Set fill type of layer n | 1158 |
| `find_nth` | `string sep quote keep_quote n` | Return n-th (1-based) field, quote aware | 1185 |
| `flip` | `[x0 y0]` | Flip selection horizontally around point x0 y0 | 1195 |
| `flip_in_place` |  | Flip selection horizontally, each object around its center | 1219 |
| `flipv` | `[x0 y0]` | Flip selection vertically around point x0 y0 | 1236 |
| `flipv_in_place` |  | Flip selection vertically, each object around its center | 1270 |
| `floaters_from_selected_inst` |  | Flatten selected instance texts into floating texts at current level | 1296 |
| `fullscreen` |  | Cycle fullscreen modes: fullscreen+menu/status, fullscreen, normal | 1305 |
| `get` | `var` | Read a C-side variable/constant (see `xschem get` table) | 1318 |
| `get_additional_symbols` | `what` | Create derived symbols for instance-based implementation selection (`schematic=` attr on instance) | 1782 |
| `get_cell` | `cell n_dirs` | Strip path keeping last n_dirs components (library-relative cell name) | 1792 |
| `get_cell_w_ext` | `cell n_dirs` | Same as get_cell, keep extension | 1801 |
| `get_fqdevice` | `instname param modelparam` | Fully qualified ngspice vector name for a device param/current of instance (hier-path aware); modelparam 0 = current, 1 = model param (gm), 2 = model voltage (vth) | 1820 |
| `getprop` | `instance\|instance_notcl\|instance_pin\|symbol\|rect\|text\|wire ...` | Read attributes of objects (see getprop/setprop table) | 1883 |
| `get_sch_from_sym` | `inst [symbol]` | Return schematic file associated with instance (or symbol if inst = -1) | 2015 |
| `get_tok` | `str tok [with_quotes]` | Get value of attribute tok in attribute string str | 2049 |
| `get_tok_size` |  | Length of last looked-up attribute name (0 = not found) | 2065 |
| `globals` |  | Dump internal global state (debug) | 2073 |
| `go_back` | `[what]` | Pop one hierarchy level; what = 1 ask to save, 2 keep title | 2192 |
| `grabscreen` |  | Grab root window (debug) | 2205 |
| `hash_file` | `file [skip_path_lines]` | Simple content hash of file | 2220 |
| `hash_string` | `str` | Simple hash of string | 2237 |
| `help` |  | Print command help | 2250 |
| `hier_psprint` | `[file]` | Hierarchical PS/PDF print of whole hierarchy | 2258 |
| `hilight` | `[drill]` | Highlight selected nets/pins/labels/instances; `drill` propagates through `propag` pins | 2272 |
| `hilight_instname` | `[-fast] inst` | Highlight instance by name/number | 2285 |
| `hilight_netname` | `[-fast] net` | Highlight net by name | 2338 |
| `image` | `invert\|white_transp\|black_transp\|transp_white\|transp_black\|blend_white\|blend_black\|write_back` | Pixel ops on selected embedded images; write_back stores result in `image_data` | 2374 |
| `instance` | `sym_name x y rot flip [prop] [n]` | Place instance programmatically; prop defaults to symbol `template`; n != 0 for batch speed-up | 2433 |
| `instance_bbox` | `inst` | Instance and symbol bounding boxes | 2457 |
| `instance_coord` | `[instance]` | `{name} {sym} x y rot flip` for selected (or given) instances | 2488 |
| `instance_list` |  | List of `{name} {symref} {type}` for all instances | 2535 |
| `instance_net` | `inst pin` | Net attached to pin of instance | 2554 |
| `instance_nodemap` | `inst [pin]` | Instance name followed by `pin net` pairs | 2589 |
| `instance_number` | `inst [n]` | Get (or set to n) the array index of instance | 2618 |
| `instance_pin_coord` | `inst attr value` | Name and x y of pin whose `attr` = value | 2647 |
| `instance_pins` | `inst` | List of pin names of instance | 2703 |
| `instance_pos` | `inst` | Array index of instance name | 2729 |
| `instances_to_net` | `net` | List `{inst} {pin} {x} {y}` attached to net | 2746 |
| `is_generator` | `symbol` | True if symbol ref is a generator `name(param,...)` (Tcl/exec script producing symbol) | 2784 |
| `line` | `[x1 y1 x2 y2] [pos] [prop] [draw] \| gui` | Add line on current layer at coords (pos = array index, -1 append); no args = GUI; `gui` = GUI starting at mouse | 2807 |
| `line_width` | `n` | Set line width | 2856 |
| `list_hierarchy` |  | List schematics at/below current level with mtimes | 2873 |
| `list_hilights` | `[sep\|all\|all_nets\|all_inst]` | Sorted list of highlighted nets / instances | 2889 |
| `list_nets` |  | List nets with type in/out/inout/net | 2906 |
| `list_tokens` | `str with_quotes` | Tokenize attribute string (0 eat quotes, 1 keep, 2 eat backslashes) | 2921 |
| `load` | `[-gui\|-noundoreset\|-nosymbols\|-nofullzoom\|-nodraw\|-keep_symbols] f` | Load file into current window | 2940 |
| `load_new_window` | `[f]` | Open file in new tab/window (prompt if no f, `{}` = untitled.sch) | 3045 |
| `log` | `[f]` | Redirect stderr to log file f / close log | 3102 |
| `load_symbol` | `[symbol_file]` | Load symbol definition; returns >=0 if loaded | 3124 |
| `log_write` | `text` | Write text to log file | 3148 |
| `logic_get_net` | `net_name` | Built-in logic sim: get net state 0/1/2(X)/3(Z) | 3159 |
| `logic_set_net` | `net_name n [num]` | Built-in logic sim: force net to 0/1/X/Z or toggle (-1), num times | 3202 |
| `logic_set` | `n [num]` | Built-in logic sim: force selected nets/labels/pins to level n | 3225 |
| `make_sch` |  | Create a schematic (.sch) with pins from selected symbol | 3242 |
| `make_sch_from_sel` |  | Turn selection into an LCC sub-schematic instance (optionally creating .sym) | 3252 |
| `make_symbol` |  | Generate circuit.sym from circuit.sch using ipin/opin/iopin instances | 3263 |
| `merge` | `[f]` | Merge (paste) another file into current schematic | 3277 |
| `move_instance` | `inst x y rot flip [nodraw] [noundo]` | Set instance placement; `-` keeps a value | 3297 |
| `move_objects` | `[dx dy] [kissing] [stretch]` | Move selection by dx,dy or start GUI move | 3333 |
| `my_strtok_r` | `str delim quote keep_quote` | Debug tokenizer | 3360 |
| `net_label` | `[type]` | Place net label: 1 = lab_pin.sym, 0 = lab_wire.sym (GUI) | 3384 |
| `net_pin_mismatch` |  | Highlight nets on selected symbols whose name differs from pin name | 3394 |
| `netlist` | `[-keep_symbols\|-noalert\|-messages\|-erc\|-nohier] [filename]` | Netlist current schematic in current format; bare filename -> netlist dir; `-messages` returns ERC text; `-nohier` current level only | 3414 |
| `new_process` | `[f]` | Spawn a separate xschem process | 3513 |
| `new_schematic` | `create\|destroy\|destroy_all\|switch win_path [file] [draw]` | Create/destroy/switch tabs or windows (main = `.drw`, others `.x1.drw`, ...) | 3543 |
| `only_probes` |  | Dim schematic to show highlights/probes only | 3570 |
| `origin` | `x y [zoom]` | Set view origin (and zoom); `-` keeps value | 3582 |
| `parse_cmd` |  | Debug: test command-string splitter | 3604 |
| `parselabel` | `str` | Debug: test bus label parser | 3618 |
| `paste` | `[x y]` | Paste clipboard at x,y or GUI placement | 3627 |
| `pinlist` | `inst [attr]` | List `{idx} {attrs\|attr value}` for all pins of instance | 3653 |
| `place_symbol` | `[sym_name] [prop]` | GUI placement of symbol (prompt if none) | 3684 |
| `place_text` |  | Start a GUI placement of a text object | 3717 |
| `polygon` | `[gui]` | Start GUI polygon placement | 3739 |
| `preview_window` | `create\|draw\|destroy\|close [win_path] [file]` | Schematic preview pane used by file selector | 3764 |
| `print` | `png\|svg\|ps\|eps\|pdf\|ps_full\|pdf_full file [w h] [x1 y1 x2 y2]` | Export image; size 0 0 = auto; area defaults to selection bbox else full drawing; `*_full` = whole schematic, multi-page | 3798 |
| `print_hilight_net` | `show` | From highlighted nets: 0 create pins, 1 list dialog, 2 create `i`-prefixed labels, 3 list with path, 4 create labels | 4023 |
| `print_spice_element` | `inst` | Print SPICE netlist line for one instance | 4033 |
| `propagate_hilights` | `[set clear]` | Debug wrapper of propagate_hilights() | 4049 |
| `push_undo` |  | Push current state onto undo stack | 4062 |
| `raw` | `subcmd ...` | Raw-file / waveform DB API (see `xschem raw` table); alias `raw_query` | 4191 |
| `raw_clear` |  | Unload all raw files | 4410 |
| `raw_read` | `[file] [sim] [sweep1 sweep2]` | Load raw file analysis (tran/dc/ac/op...), optionally only sweep window | 4425 |
| `raw_read_from_attr` | `[sim]` | Load raw data embedded (base64) in selected instance `spice_data` attr | 4464 |
| `rebuild_connectivity` |  | Rebuild net connectivity | 4485 |
| `rebuild_selection` |  | Rebuild selection list | 4501 |
| `record_global_node` | `n node` | Internal: manage netlist global node list | 4509 |
| `rect` | `[x1 y1 x2 y2] [pos] [prop] [draw] \| gui` | Add rectangle on current layer or start GUI placement | 4536 |
| `redo` |  | Redo last undone action | 4584 |
| `redraw` |  | redraw window | 4593 |
| `reload` |  | Force reload current schematic from disk | 4602 |
| `reload_symbols` |  | Reload all used symbols from disk | 4618 |
| `remove_symbols` |  | Internal: purge all symbol definitions | 4630 |
| `replace_symbol` | `inst new_symbol [fast]` | Swap instance's symbol keeping attributes | 4644 |
| `reset_caches` |  | Reset instance/symbol cached flags | 4710 |
| `reset_inst_prop` | `inst` | Reset instance attrs to symbol `template` | 4719 |
| `reset_symbol` | `inst symref` | Low level: change instance symbol reference name only | 4767 |
| `resetwin` | `create_pixmap clear_pixmap force w h` | Internal: window resize handling | 4786 |
| `resolved_net` | `[net]` | Top-most hierarchical name of net (maps through ports upward) | 4799 |
| `rotate` | `[x0 y0]` | Rotate selection 90 deg around x0,y0 (default mouse) | 4832 |
| `rotate_in_place` |  | Rotate each selected object around its own origin | 4857 |
| `round_to_n_digits` | `i n` | Round number to n significant digits | 4873 |
| `save` | `[fast]` | Save if modified (no confirm) | 4889 |
| `saveas` | `[file] [schematic\|symbol]` | Save as file / prompt; type sets .sch/.sym extension | 4915 |
| `sch_pinlist` |  | `{pin} {dir}` list of schematic ports (ipin/opin/iopin) | 4944 |
| `schematic_in_new_window` | `[new_process] [nodraw] [force]` | Open selected instance's schematic in new tab/window | 4973 |
| `search` | `regex\|exact select tok val [match_case]` | Find objects whose attr tok matches val; select 0 = highlight, 1 = select, -1 = unselect; tok may be `propstring`, `cell::name`, `cell::<attr>`; alias `searchmenu` shows 'Not found' box | 5011 |
| `select` | `instance\|wire\|text id \| rect\|line\|poly\|arc layer id  [clear] [fast] [nodraw]` | Select (or unselect with `clear`) one object | 5049 |
| `select_all` |  | Selects all objects in schematic | 5148 |
| `select_dangling_nets` |  | Select nets/labels not attached to any real component | 5158 |
| `select_hilight_net` |  | Select all highlighted objects | 5167 |
| `select_inside` | `x1 y1 x2 y2 [sel]` | Area select (sel = 0 unselect) | 5177 |
| `selected_set` | `[inst\|rect\|text]` | List selected instances (or rects/texts with coords) | 5196 |
| `selected_wire` |  | List selected nets | 5239 |
| `send_to_viewer` |  | Send selected nets/currents to external viewer (gaw/bespice) | 5257 |
| `set` | `var value` | Write a C-side variable (see `xschem set` table) | 5285 |
| `set_different_tok` | `str new_str old_str` | Apply only changed attributes between old/new to str (multi-edit merge) | 5488 |
| `set_modify` | `[n]` | Force modified flag: 0 clear, 1 set (+title/buttons), 2/3 raw clear/set, -1 title, -2 floater caches | 5509 |
| `setprop` | `[-fast\|-fastundo] instance\|symbol\|text\|rect ...` | Write attributes (see getprop/setprop table) | 5550 |
| `simulate` | `[callback]` | Run default simulator from sim config; callback proc run on completion | 5805 |
| `snap_wire` |  | Start wire from closest pin/net endpoint | 5818 |
| `str_replace` | `str rep with [escape] [count]` | Replace substring unless escaped | 5828 |
| `subst_tok` | `str tok newval` | Replace/add attribute tok in attribute string | 5843 |
| `symbol_base_name` | `n` | Base symbol name of a derived (instance-overloaded) symbol | 5857 |
| `symbol_in_new_window` | `[new_process]` | Open selected instance's symbol in new tab/window | 5880 |
| `swap_cursors` |  | Swap graph cursors a and b | 5892 |
| `swap_windows` |  | Internal: swap first/second window (window i/f) | 5916 |
| `switch` | `[win_path\|schematic_name]` | Switch current context to tab/window | 5930 |
| `symbols` | `[n\|derived_symbols]` | List loaded symbols | 5943 |
| `tab_list` |  | List tabs/windows with path and filename | 5971 |
| `table_read` | `[table_file]` | Load whitespace column table (header row; blank-blank line = new dataset; `#` comments) as waveform data | 6018 |
| `test` |  | Test mode (debug) | 6042 |
| `text` | `x y rot flip text props size draw` | Create text object | 6127 |
| `text_string` | `n` | Get text string of text n | 6141 |
| `toggle_colorscheme` |  | Toggle dark/light colors | 6157 |
| `toggle_ignore` |  | Toggle `spice_ignore`/`verilog_ignore`/`vhdl_ignore`/`tedax_ignore` (per current netlist mode) on selected instances | 6174 |
| `touch` | `x1 y1 x2 y2 x0 y0` | 1 if segment touches point | 6183 |
| `translate` | `n str` | @-substitute str using instance n attrs (-1 = no instance) | 6205 |
| `translate3` | `str eat_escapes s1 [s2] [s3]` | @-substitute str looking up tokens in s1, then s2, then s3 | 6229 |
| `trim_chars` | `str sep` | Trim leading/trailing chars in sep | 6249 |
| `trim_wires` |  | Remove overlapping wires, join collinear, split at intersections | 6259 |
| `undo` | `[redo [set_modify]]` | Undo last action | 6272 |
| `undo_type` | `disk\|memory` | Select undo storage | 6289 |
| `unhilight_all` | `[fast]` | Clear all highlights | 6321 |
| `unhilight` |  | Unhighlight selected nets/pins | 6335 |
| `unselect_all` | `[draw]` | Clear selection | 6343 |
| `unselect_attached_floaters` |  | Unselect floating objects attached to named instances | 6354 |
| `update_all_sym_bboxes` |  | Recompute symbol bboxes | 6361 |
| `update_op` |  | Refresh Tcl `ngspice::ngspice_data` array from raw file point 0 (OP) | 6377 |
| `view_prop` |  | Read-only attribute view of first selected element | 6391 |
| `warning_overlapped_symbols` | `[sel]` | Highlight (or select) perfectly overlapping instances | 6403 |
| `windowid` | `topwin_path` | Internal: set window icon | 6414 |
| `wire_coord` | `n` | Coordinates of wire n | 6422 |
| `wire` | `[x1 y1 x2 y2] [pos] [prop] [sel] \| gui` | Add wire or start GUI wire placement | 6445 |
| `wire_cut` | `[x y] [noalign]` | Cut wire at point (snapped unless noalign) or start GUI cut (Alt+right click) | 6497 |
| `xcb_info` |  | Debug | 6524 |
| `zoom_box` | `[x1 y1 x2 y2] [factor]` | Zoom to area (factor <1 shrink, >1 border) or GUI zoom box | 6536 |
| `zoom_full` | `[center\|nodraw\|nolinewidth]` | Zoom to fit | 6565 |
| `zoom_hilighted` |  | Zoom to highlighted objects | 6588 |
| `zoom_in` |  | Zoom in drawing | 6597 |
| `zoom_out` |  | Zoom out drawing | 6606 |
| `zoom_selected` |  | Zoom to selection | 6615 |

### 4C.3 `xschem raw` sub-subcommands (`scheduler.c:4072-4190`)

| form | meaning |
|---|---|
| `raw read file [type [sweep1 sweep2]]` | load analysis `type` (tran/dc/ac/op/...; default first found); optional sweep window `sweep1 <= x < sweep2` |
| `raw clear [rawfile [type] \| n]` | unload given file/type, n-th file, or all |
| `raw del name` | delete vector from current raw |
| `raw rename old new` | rename a node |
| `raw info` | describe loaded raws, show active one |
| `raw new name type sweepvar start end step` | create synthetic raw dataset |
| `raw list` / `raw vars` | list variable names / count |
| `raw switch [n \| rawfile type]` / `raw switch_back` | make another loaded raw active / go back (multiple raws can be loaded at once) |
| `raw datasets` | number of datasets (runs, e.g. `.step`/Monte Carlo) |
| `raw value node n [dset]` | n-th value; `n = {}` = value at cursor b; `dset = -1` absolute index |
| `raw values node [dset]` | all values (dset -1 = all) |
| `raw pos_at node value [dset] [from] [to]` | first index where node brackets value |
| `raw points [dset]` | point count |
| `raw set node n value [dset]` | overwrite a data point |
| `raw index node` | variable index, e.g. `v(led)` -> 46 |
| `raw loaded` / `raw rawfile` / `raw sim_type` | hier level raw was loaded at (-1 none) / filename / analysis type |
| `raw table_read file` | load column text table as data |
| `raw add varname [expr] [sweep_var]` | add vector, optionally computed from RPN expression, e.g. `{outm outp - i(@r1[i]) *}` |

### 4C.4 `xschem get <var>` (66 names; `scheduler.c:1316-1778`)

Unknown names silently return empty (`cmd_found = 0`).

| var | returns |
|---|---|
| `backlayer` `wirelayer` `gridlayer` `sellayer` `textlayer` `pinlayer` | fixed layer numbers 0, 1, 2, 2, 3, 5 (`xschem.h:148-156`; SYMLAYER = 4) |
| `bbox` / `bbox_hilighted` / `bbox_selected` | `x1 y1 x2 y2` of drawing / highlighted / selected |
| `build_date` `version` `help` | build stamp / xschem version / help flag (-h) |
| `cadlayers` | number of layers (22 default) |
| `case_insensitive` | symbol lookup mode |
| `color_ps` | color PS/SVG export flag |
| `constr_mv` | constrained move 0 none, 1 horiz, 2 vert |
| `current_dirname` `current_name` | dir / name (no path) of current design |
| `current_win_path` `top_path` `topwindow` `last_created_window` `ntabs` | window path (`.drw`, `.x1.drw`), toplevel prefix (`""`/`.x1`), same with `.` for main, last created, extra tab count |
| `currsch` | hierarchy depth (0 = top) |
| `cursor1_x` `cursor2_x` | graph cursor positions |
| `debug_var` `semaphore` `ui_state` `draw_window` `no_draw` | debug/internal state |
| `first_sel` | `type n col` of first selected object |
| `fix_broken_tiled_fill` `fix_mouse_coord` | workaround flags |
| `format` | alternate netlist format attribute name (e.g. `spectre_format`) or empty |
| `graph_lastsel` | last clicked graph index |
| `header_text` | file header/license comment block |
| `infowindow_text` | ERC messages |
| `instances` `wires` `symbols` | object counts |
| `lines n` `rects n` | count on layer n |
| `intuitive_interface` | click-drag move enabled |
| `lastsel` | number selected |
| `line_width` | current line width |
| `modified` | needs save |
| `netlist_name [fallback]` | custom netlist name; `fallback` gives default `<sch>.<ext>` |
| `netlist_type` | spice/vhdl/verilog/tedax/symbol |
| `raw_level` | hier level raw refers to |
| `rectcolor` | current drawing layer |
| `schname [n]` | full path of current (or level n) schematic |
| `sch_path [n]` | hierarchy instance path, e.g. `.x1.x3.` |
| `sim_sch_path` | sch_path relative to level where raw was loaded |
| `sch_to_compare` | file current design is diffed against |
| `schprop` `schvhdlprop` `schverilogprop` `schsymbolprop` `schtedaxprop` | global attribute blocks (S/V/G/K/E records) |
| `temp_dir` | Windows temp dir |
| `text_svg` | use `<text>` in SVG export |
| `xschem_web_dirname` | temp dir for schematics/symbols loaded from http(s) URLs (`save.c:3531`) |
| `xorigin` `yorigin` `zoom` | view transform |

### 4C.5 `xschem set <var> <value>` (32 names; `scheduler.c:5283-5484`)

`cadgrid` `cadsnap` `color_ps` `crosshair_layer` `constr_mv` `cursor1_x` `cursor2_x` `draw_window` `fix_broken_tiled_fill` `fix_mouse_coord` `format` (custom format attr name for netlisting) `header_text` `hide_symbols` (0 show, 1 bbox for subckt symbols, 2 bbox all) `hilight_color` (layer for next highlight) `infowindow_text` `intuitive_interface` `netlist_name` `netlist_type` (spice\|verilog\|vhdl\|tedax\|symbol) `no_draw` `no_undo` `raw_level` `rectcolor` `sch_to_compare` `schsymbolprop` `schprop` `schverilogprop` `schvhdlprop` `schtedaxprop` `text_svg` `semaphore` `show_hidden_texts` `sym_txt` (0 hides symbol texts).

Note: Tcl globals of the same name (e.g. `cadsnap`) are separate; menus set both (`xschem.tcl:8689`).

### 4C.6 `getprop` / `setprop` object types (`scheduler.c:1835-2011`, `5517-5797`)

| form | effect |
|---|---|
| `getprop instance inst` | full attribute string |
| `getprop instance inst attr` | attr value; `cell::attr` reads the referenced symbol's attr; `cell::name` = symbol name |
| `getprop instance_notcl inst attr` | same, no `tcleval()` substitution |
| `getprop instance_pin inst pin [pin_attr]` | pin attribute string / one attr (pin by name or index) |
| `getprop symbol sym [attr [with_quotes]]` | symbol global attrs (`K` block) / one attr |
| `getprop rect layer n attr [with_quotes]` | rect attr (graphs/images are rects) |
| `getprop text n attr` | text attr; `txt_ptr` = text string |
| `getprop wire n attr` | wire attr |
| `setprop [-fast] instance inst [tok] [val]` | set attr; `tok=allprops` replaces whole string; no val deletes attr; no tok clears; renaming `name` is uniquified; `-fast` = no redraw, no undo |
| `setprop symbol name tok [val]` | in-memory only (not saved; netlister reloads symbols) |
| `setprop rect [-fast\|-fastundo] lay n tok [val]` | rect attr; special `setprop rect 2 n fullxzoom\|fullyzoom` (graph full zoom); `fill` updates fill |
| `setprop [-fast\|-fastundo] text n [tok] [val]` | text attr; no tok or `txt_ptr` sets text string |

### 4C.7 Other programmatic interfaces

| interface | details | cite |
|---|---|---|
| TCP command server | `xschem_listen_port` (or `--tcp_port`); each connection sends Tcl, gets result, server closes socket | `xschem.tcl:9634`, `2378` |
| bespice socket | `bespice_listen_port` (0 = auto) cross-probe to bespice wave | `xschem.tcl:2407` |
| gaw socket | `gaw_tcp_address {localhost 2020}` | `xschemrc:509` |
| CLI hooks | `--preinit` (before rc), `--tcl` (after rc, before xschem.tcl), `--script file`, `--command` (after startup) | `xschem.help`, `xinit.c:2547, 2670` |
| batch CLI | `-n` netlist, `-S` simulate, `-q` quit, `-x` no X, `--png/--svg/--pdf/-p`, `--plotfile`, `-o/--netlist_path`, `-N/--netlist_filename`, `-s/-V/-w/-t/-y` netlist type, `-f` flat, `--diff file`, `-l` log, `-d n` debug, `-z` rainbow colors, `-r` no readline, `--pipe`, `-b` detach, `-i/--no_rcload`, `--rcfile` | `src/xschem.help` |
| startup hooks (vars) | `tcl_files` (sourced after init), `postinit_commands` (eval after startup), `user_startup_commands` (eval after every new window/tab creation), `netlist_postprocess` (eval after `xschem netlist`), `cursor_2_hook` (eval on graph cursor b move), `replace_key(<from>) <to>` (rebind keys) | `xinit.c:3082-3107`, `xinit.c:1678`, `scheduler.c:3498`, `callback.c:276`, `xschem.tcl:9591` |
| custom menu/buttons | `src/add_custom_menu.tcl`, `src/add_custom_button.tcl` sample scripts for `tcl_files` | src/ |
| generators | symbol ref `name(param,...)` runs an executable/Tcl script in library path that prints a .sym; `is_generator` | `scheduler.c:2784` |

---

### 4C.8 Startup, rc search order, paths

**XSCHEM_SHAREDIR** (`xinit.c:2366-2375`): env `XSCHEM_SHAREDIR` if exists -> `$PWD` if it contains `xschem.tcl`, `systemlib`, `xschem` (running in src dir) -> compile-time `PREFIX/share/xschem` (`config.h.in:56`).

**USER_CONF_DIR**: compile time, default `~/.xschem` (`scconfig/hooks.c:137`); created on first run and seeded with a copy of system `xschemrc` (`xinit.c:2531-2543`). Windows: `%USERPROFILE%/xschem/<USER_CONF_DIR>`.

**Default XSCHEM_LIBRARY_PATH** (compile time, `scconfig/hooks.c:144-180`): `~/.xschem/xschem_library : PREFIX/share/xschem/xschem_library/devices : PREFIX/share/doc/xschem/{examples,ngspice,ngspice_verilog_cosim,logic,xschem_simulator,generators,inst_sch_select,binto7seg,pcb,rom8k}`. `~/` expanded at startup (`xinit.c:2378`). Running in src dir replaces it with `../xschem_library/*` (`xinit.c:2393-2443`). Separator `:` (Windows `;`). Overridable at configure time with `--xschem-lib-path`.

**rc files sourced** (`xinit.c:2549-2610`), in order, unless `-i/--no_rcload`:
1. `--preinit` Tcl.
2. `$XSCHEM_SHAREDIR/xschemrc` (system) - always.
3. `--rcfile <file>` if given (fatal if missing), **else** `./xschemrc` if it exists in CWD (and not running in src dir), **else** `$USER_CONF_DIR/xschemrc`. Note: `./xschemrc` and `~/.xschem/xschemrc` are mutually exclusive, not cumulative.
4. `--tcl` Tcl (`xinit.c:2670`), then `$XSCHEM_SHAREDIR/xschem.tcl` (whose `set_ne` defaults only fill variables the rc did not set).
5. After GUI build: `$USER_CONF_DIR/colors` (saved by color editor) (`xschem.tcl:10226`), `tcl_files`, `user_startup_commands`, `postinit_commands`.

**Search path cache**: `set_paths` splits `XSCHEM_LIBRARY_PATH` into `pathlist`, keeps entries that exist (or `.`, or contain `../` normalized), `~` expanded; empty -> `[pwd]` (`xschem.tcl:9481-9520`). A write trace re-runs it whenever `XSCHEM_LIBRARY_PATH` changes (`xschem.tcl:10251`). Placed symbol refs are stored relative to the first `pathlist` dir that prefixes the file (`rel_sym_path`, `xschem.tcl:7190`).

**Per-tab globals**: on tab/window switch xschem saves/restores a list of Tcl globals per context, including `XSCHEM_LIBRARY_PATH PDK PDK_ROOT SKYWATER_MODELS SKYWATER_STDCELLS netlist_dir netlist_type ...` and arrays `replace_key dircolor sim enable_layer ngspice::ngspice_data` (`xschem.tcl:8312-8350`). So each tab can have a different library path / PDK.

**Files in USER_CONF_DIR**: `xschemrc`, `simrc` (sim tool config), `recent_files`, `colors`, `.clipboard.sch` (cross-window/process clipboard), `.selection.sch`, `simulations/` (default `netlist_dir`), `xschem_library/` (`xinit.c:2627-2631`, `xschem.tcl:1210,1550,5249,6270,9837`).

**PDK handling**: xschem core has **no** PDK lookup logic; it is done in the PDK-supplied rc (e.g. open_pdks `sky130A/libs.tech/xschem/xschemrc`), which reads `env(PDK_ROOT)`, falls back to first existing of `/usr/share/pdk`, `/usr/local/share/pdk`, `~/share/pdk`, defaults `PDK=sky130A`, sets `SKYWATER_MODELS`/`SKYWATER_STDCELLS` and appends `$PDK_ROOT/$PDK/libs.tech/xschem` to `XSCHEM_LIBRARY_PATH` (doc `tutorial_xschem_sky130.html` "PDK_ROOT and PDK environment variables"; `tutorial_xschem_libraries.html` "Change project setup runtime"). Schematics reference models portably through `tcleval()` attributes using these Tcl vars (e.g. `.lib $::SKYWATER_MODELS/sky130.lib.spice tt`). Core only treats `PDK PDK_ROOT SKYWATER_MODELS SKYWATER_STDCELLS` as per-tab globals (`xschem.tcl:8313`). No gf180 snippets ship with xschem 3.4.7.

**Simulator config persistence** (`xschem.tcl:1173-1700`): Tcl array `sim`, saved to `$USER_CONF_DIR/simrc` by "Accept, Save and Close" in Simulation > Configure simulators. Keys: `sim(tool_list)` = `spice spicewave verilog verilogwave vhdl vhdlwave`; `sim(<tool>,n)`, `sim(<tool>,default)`, `sim(<tool>,<i>,cmd|name|fg|st)` (fg = wait for process, st = status dialog). Commands are Tcl-`subst`ed with `$N` netlist path, `$n` netlist path w/o ext, `$S` schematic path, `$s` schematic name, `$d` netlist dir, `$terminal`. Built-in defaults (`xschem.tcl:1566-1671`):

| tool | i | default cmd |
|---|---|---|
| spice | 0 (default) | "Ngspice interactive": `$terminal -e {ngspice -i "$N" -a \|\| sh}` (Windows: no terminal) |
| spice | 1 | "Ngspice Control mode": `ngspice "$N" -a` |
| spice | 2 | `ngspice -b -r "$n.raw" "$N"` (batch) |
| spice | 3 | `Xyce "$N"` |
| spice | 4 | `mpirun /path/to/parallel/Xyce "$N"` |
| spicewave | 0-3 | `gaw "$n.raw"`; `$terminal -e ngspice`; `rawtovcd ... && gtkwave`; bespice `bspwave --socket localhost $bespice_listen_port "$n.raw"` |
| verilog | 0-1 | `iverilog -g2012 ... && vvp`; `verilator --binary --timing ...` |
| verilogwave | 0 | `gtkwave dumpfile.vcd "$N.sav"` |
| vhdl | 0 | `ghdl -c --ieee=synopsys -fexplicit "$N" -r "$s" "--wave=$n.ghw"` |
| vhdlwave | 0 | `gtkwave "$n.ghw" "$N.sav"` |

**Colors**: `cadlayers` (22; 35 with `-z` rainbow), `dark_colors` / `light_colors` lists (one `#rrggbb` per layer), `dark_colorscheme` (1), `dark_gui_colorscheme` (Tk widgets, startup only), `pixdata(n)` 16x16 fill bitmaps, `enable_layer(n)`, `dircolor(pattern) color` (symbol color by library dir regex). Defaults `xschem.tcl:10021-10061`; missing entries padded with black (`xschem.tcl:9532`). User edits saved to `$USER_CONF_DIR/colors`.

### 4C.9 xschemrc options (documented in `src/xschemrc`, 113 names)

Default = value used when unset (from `set_ne` in `xschem.tcl` unless noted). Bold = most important.

| var | default | meaning | xschemrc line |
|---|---|---|---|
| **`XSCHEM_SHAREDIR`** | compile time | install dir of xschem.tcl, systemlib, devices | 12 |
| **`XSCHEM_LIBRARY_PATH`** | see 4C.8 | `:`-separated symbol/schematic search path; build with `set XSCHEM_LIBRARY_PATH {}` + `append XSCHEM_LIBRARY_PATH :dir`; prepend with `set XSCHEM_LIBRARY_PATH dir:$XSCHEM_LIBRARY_PATH` | 48-86 |
| `xschem_doc_dir` | - | helper var used in template only | 47 |
| `dircolor(pattern)` | `/share/xschem/` red, `/share/doc/xschem/` #338844 | color of symbols from matching dirs | 95 |
| **`XSCHEM_START_WINDOW`** | `{}` | file (library-relative) to open when none given | 104 |
| **`netlist_dir`** | `$USER_CONF_DIR/simulations` | netlist + simulation output dir | 110 |
| `local_netlist_dir` | 0 | 1: `<sch_dir>/simulation/`, 2: `<sch_dir>/simulation/<schname>/` | 117 |
| **`xschem_libs`** | `{}` | regex list of cells excluded from netlisting (treated as leaf) | 129 |
| `noprint_libs` | `{}` | excluded from hierarchical print | 133 |
| `nolist_libs` | `{}` | excluded from `list_hierarchy` | 137 |
| `bus_replacement_char` | `{}` | replace `[]` in SPICE bus names, e.g. `{<>}` or `{__}` (XSPICE) | 145 |
| `netlist_type` | spice | initial netlist mode: spice/verilog/vhdl/tedax | 153 |
| `verilog_2001` | 1 | Verilog-2001 port style | 156 |
| `line_width` / `change_lw` | 0 / 1 | fixed line width vs zoom-scaled | 160-161 |
| `color_ps` | 1 | color PS/PDF/SVG export | 164 |
| `ps_page_title` | 1 | title in PS/PDF page corner | 167 |
| `ps_paper_size` | `{a4 842 595}` | name h w in points | 171 |
| `initial_geometry` | `900x600` | main window geometry | 176 |
| `file_dialog_default_geometry` `file_dialog_sp0` `file_dialog_sp1` `file_dialog_v_sp0` | unset | file dialog geometry / pane splits | 179-185 |
| `load_file_dialog_fullpath` | 1 | show full dir paths in file dialog | 188 |
| `unzoom_nodrift` | 0 | zoom-out does not drift toward mouse | 193 |
| `zoom_full_center` | 0 | full zoom centers instead of lower-left anchor | 197 |
| `disable_unique_names` | 0 | allow duplicate refdes | 202 |
| `persistent_command` | 0 | keep drawing lines/wires after click | 206 |
| **`infix_interface`** | 1 | command key also sets first point (0: click after key) | 211 |
| **`intuitive_interface`** | 1 | click-drag moves objects without `m` | 216 |
| `unselect_partial_sel_wires` | 0 | unselect stretched wires after move | 222 |
| `select_touch` | 1 | right-to-left drag selects touched objects | 226 |
| `incremental_select` | 1 | show selection live while dragging | 231 |
| **`orthogonal_wiring`** | 0 | wires follow Manhattan routes | 237 |
| `autotrim_wires` | 0 | auto join/trim wires while editing | 242 |
| `no_ask_save` | 0 | Ctrl-S without confirm | 246 |
| `tk_scaling` | unset | Tk widget scaling (HiDPI) | 250 |
| `toolbar_icon_zoom` / `ctxmenu_icon_zoom` | 1 / 1 | icon integer scale | 256-259 |
| `use_tclreadline` | 1 | readline prompt in terminal | 262 |
| `dark_colorscheme` | 1 | dark drawing colors | 265 |
| `dark_gui_colorscheme` | 0 | dark Tk widgets (startup only) | 269 |
| `enable_layer(n)` | 1 | layer visibility | 272 |
| `draw_grid` | 1 | show grid | 275 |
| `auto_hilight_graph_nodes` | 0 | highlight nets shown in graphs | 278 |
| **`enable_stretch`** | 0 | swap normal/stretch drag modes (stretch normally Ctrl+drag) | 284 |
| `draw_crosshair` / `crosshair_layer` / `crosshair_size` | 0 / 8 / 0 (full screen) | mouse crosshair | 287-293 |
| `snap_cursor` / `snap_cursor_size` | 0 / 6 | diamond cursor snapping to nearest endpoint | 296-299 |
| `cadence_compat` | 0 | Cadence-style keybinds (`src/cadence_style_rc` sets it with crosshair, orthogonal_wiring, persistent_command, infix 0) | 302 |
| `use_cursor_for_selection` | 0 | click selects object under snap cursor | 306 |
| `big_grid_points` / `grid_point_size` | 0 / -1 | grid dot sizing | 309-312 |
| `draw_grid_axes` | 1 | draw x=0/y=0 axes | 315 |
| `verilog_bitblast` | 0 | "Group bus slices in Verilog instances" (Tcl post-processing of Verilog netlist, `xschem.tcl:1026`) | 319 |
| `search_schematic` | 0 | search whole lib path for a symbol's schematic, not only symbol dir | 323 |
| `keep_symbols` | 0 | don't purge symbols when descending/netlisting | 327 |
| `autofocus_mainwindow` | 0 | focus canvas on mouse-over even with dialog open | 335 |
| `text_replace_selection` | 1 | paste replaces selected text in widgets | 339 |
| `component_browser_on_top` | 1 | symbol browser stays above canvas | 343 |
| `graph_linewidth_mult` | 1.4 (xschemrc comment says 2.0) | graph line width multiplier | 347 |
| `to_png` | `{gm convert}` | xpm->png converter for PNG export (non-cairo path) | 357 |
| `to_pdf` | `ps2pdf` | ps->pdf converter | 364 |
| `svg_to_png` | unset | filter (stdin svg -> stdout png) for importing SVG images | 373 |
| **`undo_type`** | disk | `disk` (crash-recoverable) or `memory` | 387 |
| **`cadgrid` / `cadsnap`** | 20 / 10 | grid / snap; changing breaks symbol libs compatibility | 394-395 |
| `cadlayers` / `light_colors` / `dark_colors` | 22 / lists | layer count and palettes | 400-413 |
| `pixdata(n)` | per layer | 16 rows of 16-bit hex fill pattern | 424-432 |
| `cairo_font_scale` | 1.0 | font scale | 438 |
| `nocairo_font_xscale` / `nocairo_font_yscale` | .85 / .88 | non-cairo font metrics | 442-443 |
| `cairo_font_line_spacing` | 1.0 | text line spacing | 446 |
| `cairo_font_name` / `svg_font_name` | `Sans-Serif` | fonts | 449-450 |
| `cairo_vert_correct` / `nocairo_vert_correct` | 0 | text baseline correction | 455-456 |
| `replace_key(<key>)` | - | key rebinding, e.g. `set replace_key(Control-q) Escape` | 463 |
| **`terminal`** | `xterm` | terminal for `$terminal` in sim commands | 473 |
| **`editor`** | `gvim -f` | external editor (must not detach) | 483 |
| `show_infowindow` | 0 | show ERC window at start | 495 |
| `show_infowindow_after_netlist` | onerror | never/onerror/always | 502 |
| `gaw_tcp_address` | `{localhost 2020}` | gaw socket | 509 |
| `xschem_listen_port` | `{}` | TCP command server port | 515 |
| `bespice_listen_port` | `{}` | bespice socket (0 = auto) | 523 |
| **`tcl_files`** | `{}` | Tcl files sourced after startup (e.g. `ngspice_backannotate.tcl`) | 529 |
| `postinit_commands` | unset | Tcl eval'd after startup | 538 |
| `user_startup_commands` | unset | Tcl eval'd after each new window/tab | 567 |
| `netlist_postprocess` | unset | Tcl eval'd after netlisting | 576 |
| `download_url_helper` | `{curl -f -s -O -J}` | fetch http(s) schematic/symbol URLs | 582 |
| **`toolbar_visible` / `toolbar_horiz`** | 1 / 1 | toolbar | 589-590 |
| **`tabbed_interface`** | 1 | tabs (1) vs separate windows (0); switchable only with one window open | 597 |
| `new_symbol_browser` (+ `_paths`, `_depth` 2, `_ext` `\.(sch\|sym\|tcl)$`) | 0 | alternate insert-symbol browser | 603-610 |
| `new_file_browser` (+ `_paths`, `_depth`, `_ext`) | 0 | alternate open-file browser | 616-623 |
| `case_insensitive` | 0 | case-insensitive symbol lookup | 636 |
| `show_hidden_texts` | 0 | show texts with `hide=true` | 643 |
| `graph_use_ctrl_key` | 0 | require Ctrl for mouse ops on graphs | 651 |
| `hide_empty_graphs` | 0 | hide graphs with no data | 658 |
| `cursor_2_hook` | `{}` | Tcl run when graph cursor b moves (e.g. `conducting_devices`, `hilight_high_nets`) | 666 |
| `live_cursor2_backannotate` | 1 | back-annotate probe values at cursor b | 675 |
| `add_all_windows_drives` | 1 | Windows: add all drives to path | 683 |
| `fix_broken_tiled_fill` | **1** in code (`xschem.tcl:10014`), rc comment says 0 | alternate fill drawing for buggy GPUs | 690 |
| `fix_mouse_coord` | 0 | RDP mouse-coord workaround on key events | 699 |
| (`source .../cadence_style_rc`) | - | Cadence emulation preset | 702 |

### 4C.10 Other `set_ne` defaults not in xschemrc (selected; 61 names total, `xschem.tcl:9796-10186`)

| var | default | meaning |
|---|---|---|
| `flat_netlist` | 0 | flat SPICE netlist (Options > Netlist) |
| `split_files` | 0 | one netlist file per subcircuit |
| `netlist_show` | 0 | open netlist after netlisting (Shift+A) |
| `spiceprefix` | 1 | honor `spiceprefix` attr (Simulation > LVS) |
| `lvs_netlist` | 0 | LVS mode: use `lvs_format` attr instead of `format`, implies top-level .subckt (`spice_netlist.c:176-179`) |
| `top_is_subckt` | 0 | wrap top level in .subckt |
| `uppercase_subckt` | 0 | `.SUBCKT`/`.ENDS` in upper case |
| `lvs_ignore` | 0 | honor `lvs_ignore` attrs (short/ignore devices) |
| `transparent_svg` | 0 | SVG export without background |
| `only_probes` | 0 | dim all but probes |
| `incr_hilight` | 1 | each highlight uses next color |
| `auto_hilight` | 0 | highlight net on selection |
| `en_hilight_conn_inst` | 0 | also highlight connected instances |
| `compare_sch` | 0 | diff-highlight vs `sch_to_compare` |
| `hide_symbols` | 0 | 0 details, 1 bbox subckts, 2 bbox all (Alt+B) |
| `sym_txt` | 1 | show symbol texts |
| `symbol_width` | 150 | symbol body width passed to `make_sym.awk` by make_symbol (`xschem.tcl:5440`) |
| `rainbow_colors` | 0 | 35-layer rainbow palette |
| `enable_dim_bg` / `dim_bg` / `dim_value` | 0 / 0.0 / 0.0 | color dimming |
| `constr_mv` | 0 | constrained move |
| `draw_window` | 0 | draw directly to window (no pixmap) |
| `no_change_attrs` / `preserve_unchanged_attrs` / `copy_cell` | 0 | multi-instance edit_prop behaviour / copy cell on rename |
| `launcher_default_program` | `xdg-open` (Win: `start`) | opens `url=` / launcher targets |
| `edit_prop_size` | `80x12` | property dialog text size |
| `tabstop` | 8 | text widget tab stop |
| `graph_bus` `graph_logx` `graph_logy` `graph_rainbow` `graph_private_cursor` `graph_sort` `graph_sel_color` | 0/0/0/0/0/0/4 | graph dialog defaults |
| `measure_text` | `y=\nx=` | graph measure tooltip |
| `XSCHEM_TMP_DIR` | `/tmp` | temp dir |
| `debug_var` / `debug_tcleval` / `menu_debug_var` | 0 | debug levels (`debug_tcleval` traces `tcleval()` attrs) |
| `tclstop` | 0 | Esc-requested stop flag |
| `file_dialog_globfilter` / `file_dialog_ext` | `*` | file dialog filters |
| `toolbar_list` | list | toolbar button ids (`xschem.tcl:7877`) |
| `sim(...)` | see 4C.8 | simulator table |

Non-`set_ne` Tcl vars read by C: `use_label_prefix`, `custom_label_prefix`, `use_lab_wire`, `do_all_inst`, `user_wants_copy_cell`, `retval`, `props`, `symbol`, `rotated_text` (dialog plumbing between Tcl dialogs and C).

### 4C.11 Notable discrepancies / gotchas

- `./xschemrc` shadows `~/.xschem/xschemrc` entirely (not merged) (`xinit.c:2596-2608`).
- `fix_broken_tiled_fill`: rc says default 0, code defaults 1 (`xschem.tcl:10014`).
- `graph_linewidth_mult`: rc says 2.0, code 1.4 (`xschem.tcl:9938`).
- `search_schematic` comment in xschem.tcl:9853 ("search in [pwd]") differs from xschemrc:321 ("search full search path").
- `xschem get constr_mv` / `intuitive_interface` source comments are copy-paste errors ("color postscript flag", "ERC messages").
- Many settings are per-tab (4C.8), so a reimplementation should scope config per document/window context, not process-global.

## 5. File format (.sch / .sym)

Sources: `src/save.c` (reader/writer), `src/token.c` (attribute grammar), `src/xschem.h` (geometry macros), `src/xschem.tcl` (path resolution), `doc/xschem_man/developer_info.html` (spec). All paths below relative to the source root `/nix/store/rhvgzl9mjl58nr0qm6l98fxbiadg3q98-source`.

### 5.1 General structure

- `.sch` and `.sym` share one grammar. The extension only changes defaults (symbol-editing mode, how `G`/`K` are written). `load_schematic` switches to symbol mode when `is_xschem_file` says SYMBOL or the file has 0 instances (save.c:3619-3632).
- Line oriented. A record is one tag char plus space-separated fields, and every free-text field is a `{...}` brace string. Brace strings may span lines.
- Reader loop (save.c:3058-3201): `fscanf(" %c")` skips any whitespace (blank lines and leading spaces are OK), dispatches on the tag char, parses the fields, then `read_line()` throws away the rest of the physical line (save.c:3184). Trailing garbage after the last `{}` is ignored. Only one record per line.
- Numbers are read with `%lf`/`%d`/`%hd` (C float syntax, so `1e3` works). The writer uses `%.16g` for doubles, `%d` for layers and `%hd` for rot/flip (save.c:2603-2703). xschem forces locale `C` (Changelog:731), so the decimal separator is always `.`.
- CR (`\r`) is dropped everywhere inside brace strings, including after a `\` (save.c:3215). CRLF files load fine.
- Record order on load is free. Every record type may appear anywhere.

### 5.2 Record types

| Tag | Fields (in order) | Types | Where | Loader |
|---|---|---|---|---|
| `v` | `{version-string}` | brace str | first record of sch/sym and of each `[ ]` block | save.c:3075 |
| `G` | `{props}` | brace str | sch: **VHDL** global text (`schvhdlprop`). sym (pre-1.2 / legacy): symbol attributes | save.c:3114 |
| `K` | `{props}` | brace str | sym: symbol attributes (`type`, `format`, `template`, …). sch (1.2+): attributes used when the .sch itself is placed as a symbol | save.c:3111 |
| `V` | `{text}` | brace str | **Verilog** global text | save.c:3108 |
| `S` | `{text}` | brace str | **SPICE** global text | save.c:3105 |
| `E` | `{text}` | brace str | **tEDAx** global text | save.c:3102 |
| `F` | rest of line ignored | — | reserved ("future symbol floater labels") | save.c:3099 |
| `#` | rest of line ignored | — | comment line | save.c:3096 |
| `L` | `layer x1 y1 x2 y2 {props}` | int, 4×double | both | load_line save.c:3022 |
| `B` | `layer x1 y1 x2 y2 {props}` | int, 4×double | both. On layer 5 (PINLAYER) a `B` is a **symbol pin** | load_box save.c:2961 |
| `P` | `layer npoints x1 y1 … xn yn {props}` | int, int, 2n×double | both | load_polygon save.c:2860 |
| `A` | `layer cx cy r start sweep {props}` | int, 5×double (deg) | both | load_arc save.c:2918 |
| `T` | `{text} x y rot flip xscale yscale {props}` | str, 2×double, 2×short, 2×double | both | load_text save.c:2761 |
| `N` | `x1 y1 x2 y2 {props}` | 4×double | sch. In a .sym it is drawn as a line on layer 1 with no connectivity (save.c:4761) | load_wire save.c:2787 |
| `C` | `{symref} x y rot flip {props}` | str, 2×double, 2×short | sch. In a .sym the instances are flattened into graphics (save.c:4799) | load_inst save.c:2811 |
| `[` | opens an embedded symbol definition for the **immediately preceding `C`** | — | sch | save.c:3138 |
| `]` | closes the embedded block. Must be at the start of a line | — | sch | save.c:4928 |
| other | skipped up to the newline, honouring `{…}` (`read_record`) | — | — | save.c:2310, 3179 |

Total: 15 tags (`v G K V S E F # L B P A T N C`) plus the `[`/`]` delimiters.

Malformed-record handling: a bad or missing layer (`<0` or `>= cadlayers`; the default `cadlayers` is 22, xschem.tcl:10022) or a missing numeric field prints a warning and drops that record (save.c:2868-2878, 2925-2935).

### 5.3 Brace strings (`load_ascii_string` save.c:3204 / `save_ascii_string` save.c:2460)

- Read: **every character before the first `{` is skipped**. After the `{`, a `\` escapes the next character (the backslash is dropped and the next char is kept literally). The first unescaped `}` ends the string. An unescaped `{` inside is a **literal**: there is no nesting count. EOF before `}` gives a warning and the string is discarded.
- Write: wraps the string in `{}` and puts a `\` before every `\`, `{` and `}`. No other escapes are written. A NULL or empty string is written as `{}`.
- An empty `{}` loads as NULL (`my_strdup` of `""` frees the destination, editprop.c:232). Treat empty and absent as the same.
- Round trip is exact.

### 5.4 Attribute-string grammar (inside `{props}`; `get_tok_value` token.c:438, `subst_token` token.c:1220)

- Separators are space, TAB, newline, `;` and NUL (`SPACE()` token.c:24). **`;` counts as whitespace** outside quotes.
- Shape: `tok=value tok2 = "value with spaces" flag …`. Whitespace around `=` is allowed. A token with no `=` is a bare flag: lookup returns `""` and `tok_size>0`.
- Values: `"` toggles quoting unless it is escaped. An unquoted, unescaped separator ends the value. `\` escapes the next char.
- With `with_quotes=0` (the normal lookup), unescaped `"` and `\` are removed from the returned value, so `\"` becomes `"` and `\\` becomes `\`. Other modes: 1 keeps them; 4 strips only the surrounding quotes; bit 2 disables tcl evaluation (token.c:430-437).
- Duplicate keys: the **first** occurrence wins.
- A value that starts with `tcleval(` is passed through `tclpropeval2` in Tcl (`tcl_hook2` token.c:59). Inside it, `\}` is unescaped to `}` first.
- Writer rule (`subst_token`): if a new value (key other than `name`) contains `;`, space, TAB or newline and is not already quoted, it is wrapped in `"…"`. An empty value is written as `""`. A key that is not present is appended as `\nkey=value`, or ` key=value` when the string already ends in space/newline (token.c:1253-1260, 1372-1390).
- There are two escaping layers. Example from `xschem_library/devices/res.sym`: the file text `verilog_format="tran @name (@@P\\\\, @@M\\\\);"` becomes the brace string `…@@P\\, …`, and the attribute value is then `…@@P\, …`.
- Booleans (`strboolcmp`/`get_attr_val`, editprop.c:87-130): case-insensitive `true|on|yes|1+` / `false|off|no|0`.

### 5.5 Per-record attributes the loader interprets

| Rec | Attribute → effect |
|---|---|
| L | `bus=` (bool/int: thick line), `dash=<int>`, `lock`, `symbol_ignore=true` (dropped when loaded as a symbol) |
| B | `fill=true` (**default**, stippled) / `false` / `full` (solid); `dash=`; `ellipse=a,b` draws an ellipse/elliptic arc inscribed in the box, with start a and sweep b in degrees (`ellipse=` with bad numbers gives 0,360); `flags=graph[,unlocked,private_cursor]` makes a waveform graph (usually layer 2); `flags=image[,unscaled]` with `image=<path>`, `image_data=<base64>` (PNG `iVBOR…`, JPEG `/9j/…`, or SVG when `filter` is set), `alpha=`, `filter=<cmd>`. A file image is embedded into `image_data` on first draw (draw.c:4862-4891, actions.c:766). On layer 5: pin attributes `name`, `dir=in\|out\|inout`, `pinnumber`, `sim_pinnumber`, `propag`, `goto`, … |
| P | `fill=true\|full` (**default false**), `bezier=true`, `bus=`, `dash=`. Closed when last point == first point |
| A | `fill=true\|full` (**default false**), `dash=`. Angles in degrees: `start` is CCW from 3 o'clock, `sweep` is CCW. A circle has sweep 360. Point at θ = (cx + r·cosθ, cy − r·sinθ) (draw.c:1943) |
| T | `layer=<n>` (default 3), `font=`, `hcenter=true`, `vcenter=true`, `weight=bold`, `slant=italic\|oblique`, `hide=true\|instance` (`instance` hides it on placed instances only), `name=<inst>` or `floater` makes a floater text (its `@attr` resolve against that instance) (actions.c:895-925) |
| N | `lab=` (net name, cached by xschem on netlist/connectivity, normally auto-written), `bus=true`, `lock` |
| C | `name=` (instance name, required), `embed=true`, `hide`, `hide_texts`, `highlight`, `lock`, `{spice,verilog,vhdl,tedax,lvs}_ignore=true\|open\|short`, `schematic=`, `symbol_ignore`, plus free user parameters (actions.c:836-893) |
| K/G (sym) | `type`, `format`, `template`, `*_format`, `*_ignore`, `highlight`, `hide`, … (see netlisting section) (actions.c:787-834) |

Normalisation on load: `L`/`N` endpoints are ORDERed (x1<x2, or y1<=y2 when x1==x2). `B` goes through RECTORDER (x1<=x2, y1<=y2) (xschem.h:348-358).

### 5.6 Geometry conventions

- Coordinates are doubles, unitless, **y grows down** (X11 convention). Default snap is 10 and grid 20 (xschem.h:158-159). Pin boxes are conventionally ±2.5 around the pin point.
- `ROTATION(rot, flip, x0,y0, x,y)` (xschem.h:339): apply flip first (`x → 2·x0 − x`), then rotate. rot 1: `(x,y)→(x0−(y−y0), y0+(x−x0))`. rot 2: point mirror. rot 3: inverse of rot 1. Instance placement is `inst.x0 + R(sym.x)`.
  - With y down, rot=1 is **90° clockwise on screen**. Example: the symbol point (1,0) goes to (0,1).
- Arcs inside a placed instance: `start' = flip ? 270·rot + 180 − sweep − start : start + 270·rot` (mod 360) (save.c:4547-4553). The sweep is unchanged.
- Texts inside a placed instance: `rot' = (t.rot + ((inst.flip && (t.rot&1)) ? inst.rot+2 : inst.rot)) & 3`, `flip' = inst.flip ^ t.flip` (save.c:4748-4751).
- Text anchor: at rot=0, flip=0 the anchor is the top-left of the text box. The box (w,h) is mapped through the same `ROTATION(rot,flip)`, so flip=1 makes the anchor the top-right (right-aligned). The glyphs are not mirrored. `hcenter`/`vcenter` shift the anchor by w/2 or h/2. Font size = `xscale·52·cairo_font_scale` (actions.c:3483-3567). Multi-line text uses `\n`.
- Layer numbers (xschem.h:148-157): 0 background, 1 wire/net (also used for net labels), 2 selection/grid (graphs live here), 3 text, 4 symbol graphics, 5 pins, 6+ general purpose.

### 5.7 Version record and header

- Written form: `v {xschem version=3.4.7 file_version=1.2}`. If `header_text` exists, it follows on a new line inside the braces (save.c:2709-2721). Library files use this for license headers.
- Read: `file_version` comes from the token. The tokens `xschem` (only if it occurs within the first 50 chars), `version` and `file_version` are removed. The remainder, with leading blanks and one leading newline stripped, becomes `header_text` and is saved back verbatim (save.c:3073-3093).
- If there is no `file_version`, the reader assumes **1.0** (save.c:3196).
- Version effects:
  - **1.0**: symrefs have no extension. The loader appends `.sym` on load (save.c:2824), and `abs_sym_path(name, ".sym")` is used on lookup (save.c:4279, 4859, 3916). Saving a 1.0 file keeps adding `.sym` (save.c:2571, 2578).
  - **1.1**: symrefs carry an explicit extension (Changelog:1198).
  - **1.2** (current, xschem.h:27): adds `K`. Before 1.2, a `.sym` stored its symbol attributes in `G`.
- The writer always emits `file_version=1.2`.

### 5.8 Global property records: G/K legacy rules

- Write order: `v, G, K, V, S, E` (save.c:2705-2750). For a `.sym` whose `G` holds a `type=` token and `K` is empty, it writes `G {}` and `K {<old G>}` (migration, save.c:2724-2741).
- Read, schematic editor: after any record, if `G` contains `type=`, `K` is NULL and the file is `.sym`, then G and K are swapped (save.c:3186-3195).
- Read, as a symbol (`load_sym_def` save.c:4376-4397): `K` always overwrites the symbol attributes, so an empty `K {}` resets them to NULL. `G` is used only if nothing has been loaded yet. The writer puts `G` before `K`, so the effective rule is "K if present and non-empty, else G".
- `V`/`S`/`E` are ignored when a file is loaded as a symbol.

### 5.9 Save order (`write_xschem_file` save.c:2705)

`v` → `G` → `K` → `V` → `S` → `E` → all `L` (layer 0..N, file order within each layer) → all `B` (by layer) → all `A` (by layer) → all `P` (by layer) → all `T` → all `N` → all `C` (each one followed by its `[ … ]` block if it is the first instance with `embed=true` of that symbol).

- Before saving, the PINLAYER boxes of the current file are sorted by `sim_pinnumber`, but only if every pin has one (save.c:3323, 3432).
- Embedded block order differs: `v, K, G{}, V{}, S{}, E{}, L, B, A, T, P` (texts before polygons) (save.c:2492-2560).
- Other writers use other orders (`make_schematic` save.c:3257 writes `v G V E S K`), so **the reader must not depend on order**.

### 5.10 Symbol references (`C {symref}`)

Resolution pipeline:
1. `translate(inst, symref)`: `@attr` tokens are expanded from the instance's props, so symrefs like `symbolgen.tcl(inv,@ROUT\\)` work.
2. `tcl_hook2`: a `tcleval(...)` symref is evaluated (save.c:3477).
3. `match_symbol`: reuse an already-loaded symbol with the same name (token.c:182), otherwise `load_sym_def`.

`abs_sym_path` (xschem.tcl:7241-7320):
- `http(s)://` and absolute paths (`/…`, `X:/…` on Windows) are used as they are.
- The name is cleaned (collapse `//`, `/./`, `a/..`).
- A `./` or `../` prefix resolves against the process CWD (`[pwd]`) and is returned if the file or its directory exists.
- Otherwise each `XSCHEM_LIBRARY_PATH` element is tried in order as `<elem>/<symref>`; the first existing file wins. The path variable is split on `:` (`;` on Windows), and `.` means the CWD (xschem.tcl:9500-9518).
- Fallback: `<cwd>/<symref>`.

Other rules:
- Not found → `systemlib/missing.sym` (save.c:4301-4308). From a web schematic, the symbol is downloaded next to it (save.c:4285-4294).
- On load, absolute symrefs are rewritten to relative ones with `rel_sym_path`, which strips the **first** matching library-path prefix (save.c:2830, xschem.tcl:7190). So save writes the shortest path relative to the library.
- Generators: a symref matching `^[^ \t()]+\([^()]*\)[ \t]*$` (e.g. `gen.tcl(a,b)`) runs `"<abs path of gen.tcl>" a b` and reads the symbol from its stdout (token.c:81-180, save.c:4269-4277).
- `.sch` as symbol: `C {cmos_inv.sch} …` loads the schematic as a symbol (save.c:4799-4920).
  - Its `ipin`/`opin`/`iopin` instances become 5×5 PINLAYER boxes carrying `name=<lab> dir=…` plus the pin's other attributes (save.c:4055-4092).
  - Nested instances are flattened recursively into graphics (depth < `CADMAXHIER`=40). Nested pins move to layer 7.
  - Types `logo`, `netlist_commands`, `netlist_options`, `architecture`, `launcher`, … are skipped (save.c:4836-4852). Floater texts and graphs are dropped.
  - Pin order is aligned with a same-name `.sym` if one exists (save.c:4005-4053).
- The symbol name used for matching and for embedded blocks is the symref string itself (after `translate`/`tcl_hook2`).

### 5.11 Embedded symbols (`[ … ]`)

```
C {title.sym} 160 -30 0 0 {name=l2 author="Stefan" embed=true}
[
v {xschem version=3.0.0 file_version=1.2}
G {type=logo
template="name=l1 author=\\"Stefan Schippers\\""
...}
V {}
S {}
E {}
L 6 225 0 1020 0 {}
T {@schname} 235 5 0 0 0.4 0.4 {}
P 5 13 5 -30 -25 0 ... 5 -30 {fill=true}
]
```
(`xschem_library/pcb/pcb_test1_embed.sch`:53-72)

- `[` must be the first non-blank char of the line after a `C`. The rest of the `[` line is ignored (save.c:3161).
- The block is a complete sym-file body, parsed by `load_sym_def(name, fd)` until a line starting with `]` (save.c:4339, 4928).
- On load, `embed=true` is forced onto the preceding instance (save.c:3141).
- If a symbol with that symref is already loaded (an earlier instance), the block is skipped (save.c:3146-3176). Every instance with the same symref uses the **first** definition, embedded or not, depending on load order.
- Saving writes only one copy, after the first instance carrying `embed=true` (save.c:2588-2596). If no instance has `embed=true`, the symbol is no longer embedded.
- To descend into an embedded symbol, xschem writes it to `$XSCHEM_TMP_DIR/.xschem_embedded_<pid>_<cell>` (save.c:3143, 5225).
- Inside a block, nested `[ ]` blocks (embedded symbols of LCC sub-instances) are skipped (save.c:4921).

### 5.12 Symbol pins and pin order

- Pins are `B 5 …` records. Their **order in the file is the pin index** used by `@pinlist`, `@#n:…` and netlist port order.
- If every pin has `sim_pinnumber`, the pins are sorted by it on load (save.c:4991) and on save (save.c:3432).
- For a `.sch`, `ipin`/`opin`/`iopin` instances define the ports (`sort_schematic_pins` save.c:3345).

### 5.13 Clipboard / selection files

- Same grammar, written by `save_selection` (save.c:5358).
- `G { x y }` stores the paste anchor (the snapped mouse position). The merge reader uses it to offset the pasted objects (paste.c:416-424).
- `[ ]` blocks are not handled by the merge reader.

### 5.14 File type detection (`is_xschem_file` xschem.tcl:4006)

- Returns SCHEMATIC or SYMBOL when a line matches `^v\s+\{xschem\s+version\s*=.*\s+file_version\s*=`, or when the heuristic score is above 4. The score counts lines matching the `T/K/G/V/S/E {`, `B/L`, `N`, `C` regexes.
- Any `C` line → SCHEMATIC, otherwise SYMBOL. A first line starting with `#!` → GENERATOR.

### 5.15 Annotated real examples

`.sym`, from `xschem_library/devices/ipin.sym` (license header trimmed):
```
v {xschem version=3.4.4 file_version=1.2
* (license comment lines → header_text)
}
G {}                                   # legacy slot, empty in 1.2
K {type=ipin                           # symbol attributes
format="*.ipin @lab"
template="name=p1 lab=xxx"
}
V {}
S {}
E {}
L 5 -5 0 0 0 {}                        # line, layer 5
B 5 -1.25 -1.25 1.25 1.25 {name=p dir=out}   # pin #0 at (0,0)
P 5 7 -5 0 -8.75 -5 -17.5 -5 -13.75 0 -17.5 5 -8.75 5 -5 0 {fill=true}  # closed (first==last)
T {@lab} -18.75 -8.75 0 1 0.33 0.33 {}       # flip=1 → right-aligned at anchor
T {@spice_get_voltage} 1.875 3.90625 0 0 0.2 0.2 {layer=15}
```

`.sch`, from `xschem_library/ngspice_verilog_cosim/tgate.sch` (complete file):
```
v {xschem version=3.4.7RC file_version=1.2}
G {}
K {}
V {}
S {}
E {}
N 290 -240 290 -220 {lab=A}                  # wire; lab= is cached by xschem
N 290 -160 290 -140 {lab=B}
C {switch_ngspice.sym} 290 -190 0 0 {name=SWITCH model=TG
device_model=".MODEL TG SW( VT=1.5 RON=2k )"}   # multi-line props, quoted value with spaces
C {lab_pin.sym} 250 -190 0 0 {name=p1 lab=CTL}
C {lab_pin.sym} 290 -240 0 1 {name=p2 lab=A}  # flip=1
C {lab_pin.sym} 290 -140 0 1 {name=p3 lab=B}
C {lab_pin.sym} 250 -170 0 0 {name=p4 lab=0}
C {ipin.sym} 110 -170 0 0 { name=p5 lab=CTL }  # leading/trailing blanks in props are fine
C {iopin.sym} 110 -190 0 1 { name=p6 lab=B }
C {iopin.sym} 110 -210 0 1 { name=p7 lab=A }
C {ipin.sym} 110 -150 0 0 { name=p8 lab=VDD }
C {noconn.sym} 110 -150 0 1 {name=l1}
```

### 5.16 Importer gotchas (checklist)

1. `;` is an attribute separator, and there are two escape layers (file braces, then attribute quoting).
2. An unescaped `{` inside a brace string is literal. Only `}` needs balancing via escapes.
3. Default fill differs by type: `B` fill defaults to **true**, while `P`/`A` default to false.
4. `G` means VHDL in a `.sch` but symbol attributes in a legacy `.sym`. Apply the K-over-G rule.
5. A missing `file_version` means 1.0, so append `.sym` to symrefs.
6. Embedded `[ ]` binds to the previous `C` and is shared by every instance with the same symref. The first definition wins.
7. Pin index = order of the `B 5` records, unless every pin has `sim_pinnumber`.
8. Normalise L/N/B endpoints the way xschem does, or selection and hashing will differ.
9. Rotation is clockwise on screen (y down), and flip is applied before rotate.
10. Symrefs can contain `@attr` and `tcleval(...)`, and can be generators `x(args)` or `.sch` files.
11. `N`/`C` records inside a `.sym` are legal and render as graphics.
12. A layer outside `[0, cadlayers)` drops the object, and `cadlayers` is configurable (22 by default).

## 6. Counts (parity checklist sizing)

| Item | Count | Source |
|---|---|---|
| Menubar entries | 15 = 12 pull-down menus + Netlist/Simulate buttons + 1 disabled spacer | §1.0 |
| Cascade submenus inside menus | 9 | §1 |
| Static menu items (excl. separators) | 218 (209 leaves + 9 cascades); Layers menu adds 1 entry per layer (22 by default); Open recent holds up to 10 | §1.18 |
| Toolbar buttons | 30 (27 default `toolbar_list` + Netlist/Simulate/Waves) | §1.14 |
| Canvas context-menu entries | 25 distinct (14 with nothing selected, 14 with a selection) | §1.15 |
| Keyboard bindings (`handle_key_press`) | 165 distinct key+modifier combos (156 table rows) | §2.1 |
| Graph-only keys | 12 | §2.2 |
| Mouse gestures | 61 (46 on the schematic canvas + 15 in graphs) | §2.3, §4B.3 |
| Dialog windows (toplevel) | 32 distinct (built by 33 procs), plus 4 native Tk choosers and about 20 message boxes | §3.8 |
| Object attributes recognized by the C code | 114 | §4A.2 |
| @-substitution token forms | 38 | §4A.5 |
| Graph attributes | 35 | §4B.3 |
| RPN waveform operators | 45 (1 three-arg + 17 two-arg + 27 one-arg) | §4B.3 |
| CLI options | 35 | §4B.15 |
| `xschem` Tcl subcommands | 216 names (214 distinct + 2 aliases) | §4C.2 |
| `xschem raw` sub-subcommands | 22 | §4C.3 |
| `xschem get` / `xschem set` variables | 66 / 32 | §4C.4, §4C.5 |
| xschemrc options documented in `src/xschemrc` | 113 | §4C.9 |
| `set_ne` Tcl defaults in `xschem.tcl` | 159 unique (61 of them not in xschemrc) | §4C.10 |
| File record types | 15 tags (`v G K V S E F # L B P A T N C`) plus `[ ]` | §5.2 |
