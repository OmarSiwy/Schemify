# Drives xschem under Xvfb: dumps menu tree, screenshots menus + dialogs.
set OUT $env(OUT); set XDO $env(XDO); set LIB $env(SP)/xs/xschem_library
set log [open $OUT/driver.log w]
proc say {s} { puts $::log $s; flush $::log }
proc pause {ms} { set ::_p 0; after $ms {set ::_p 1}; vwait ::_p }
proc xshot {name} { exec import -window root $::OUT/$name.png; say "snap $name" }
proc snap {name {ms 500}} { update; pause $ms; xshot $name }
proc tops {} { set r {}; foreach w [winfo children .] { if {[winfo toplevel $w] eq $w} {lappend r $w} }; return $r }
proc modal {name script {ms 1200}} {
  set before [tops]
  after $ms [list modal_done $name $before]
  if {[catch {uplevel #0 $script} e]} { say "ERR $name: $e" }
  pause [expr {$ms + 400}]
}
proc modal_done {name before} {
  xshot $name
  foreach w [tops] { if {[lsearch -exact $before $w] < 0} { catch {destroy $w} } }
  set ::retval {}; set ::rcode {}; set ::search_found 1
}
proc dump_menu {m indent fh} {
  catch {uplevel #0 [$m cget -postcommand]}
  set last [$m index end]; if {$last eq "none"} return
  for {set i 0} {$i <= $last} {incr i} {
    set t [$m type $i]
    if {$t eq "tearoff"} continue
    if {$t eq "separator"} { puts $fh "${indent}----"; continue }
    set lab [$m entrycget $i -label]; set acc ""; catch {set acc [$m entrycget $i -accelerator]}
    puts $fh "${indent}- $lab[expr {$acc ne {} ? "  \[$acc\]" : {}}]  ($t)"
    if {$t eq "cascade"} { dump_menu [$m entrycget $i -menu] "$indent    " $fh }
  }
}

wm geometry . 1600x1000+0+0
update; pause 800; xschem zoom_full
snap 01_main_dark 800

if {![info exists env(TAIL)]} {
# ---- menu tree text dump
set fh [open $OUT/menu-tree.md w]
puts $fh "# XSchem 3.4.7 menu tree (dumped live from Tk)\n"
set mb .menubar
for {set i 0} {$i <= [$mb index end]} {incr i} {
  set t [$mb type $i]
  if {$t ni {cascade command}} continue
  puts $fh "## [$mb entrycget $i -label]  ($t)"
  if {$t eq "cascade"} { dump_menu [$mb entrycget $i -menu] "" $fh }
  puts $fh ""
}
close $fh

# ---- menu screenshots (x = label position on menubar)
set xs {File 5 Edit 37 Options 83 View 131 Properties 186 Layers 246 Tools 291 Symbol 339 Highlight 398 Simulation 467 Help 524 Waves 702}
set n 10
for {set i 0} {$i <= [$mb index end]} {incr i} {
  if {[$mb type $i] ne "cascade"} continue
  set lab [$mb entrycget $i -label]; set m [$mb entrycget $i -menu]
  set x [expr {[dict exists $xs $lab] ? [dict get $xs $lab] : 5}]
  $m post $x 28; snap [format "%02d_menu_%s" $n $lab] 300
  # each submenu
  set last [$m index end]
  for {set j 0} {$j <= $last} {incr j} {
    if {[$m type $j] ne "cascade"} continue
    set sub [$m entrycget $j -label]
    $m activate $j; $m postcascade $j
    snap [format "%02d_menu_%s__%s" $n $lab [regsub -all {[^A-Za-z0-9]+} $sub _]] 300
    $m postcascade none
  }
  $m unpost; incr n
}

# ---- right-click context menu, no selection then with selection
exec $XDO mousemove 700 450
modal 30_ctxmenu_noselection context_menu
xschem select instance x1
exec $XDO mousemove 700 450
modal 31_ctxmenu_selection context_menu
xschem unselect_all

# ---- dialogs
modal 40_dlg_insert_symbol {insert_symbol} 2000
modal 41_dlg_open_file {load_file_dialog {Load file} *.\{sch,sym\} INITIALLOADDIR} 2000
xschem select instance R1
modal 42_dlg_edit_prop_instance {xschem edit_prop}
xschem unselect_all
modal 43_dlg_graph_props {graph_edit_properties 0} 1500
modal 44_dlg_simconf {simconf} 1500
modal 45_dlg_search {property_search}
modal 46_dlg_layers {select_layers}
modal 47_dlg_about {about}
modal 48_dlg_keys_help {textwindow $XSCHEM_SHAREDIR/keys.help ro}
modal 49_dlg_tclcmd {tclcmd}
set props {}; modal 50_dlg_enter_text {enter_text {text:}}

}
# ---- light color scheme
xschem toggle_colorscheme; xschem redraw
snap 60_main_light 600
xschem toggle_colorscheme; xschem redraw

# ---- hierarchy descend into x1 (mos_power_ampli.sch)
xschem select instance x1
xschem descend; xschem zoom_full
snap 61_hier_descend 600
xschem go_back; xschem zoom_full

# ---- netlist viewer
set netlist_show 1
modal 63_netlist_view {xschem netlist} 2500
set netlist_show 0

# ---- waveforms: load solar_panel + its ngspice raw
xschem load $LIB/ngspice/solar_panel.sch
xschem raw_read $env(SP)/xs/simulations/solar_panel.raw tran
xschem zoom_full; xschem redraw
snap 64_waveforms_solar_panel 1500

# ---- symbol editing in new tab
xschem load_new_window $LIB/examples/mos_power_ampli.sym
pause 800
foreach w [tops] { catch {wm geometry $w 1600x1000+0+0} }
snap 62_symbol_view 800

modal 51_dlg_change_color {change_color}
say done
close $log
exit
