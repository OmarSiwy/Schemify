//! Clean-room importers (D1, D14, D24): xschem `.sch`/`.sym` designs and PDK libraries,
//! SPICE model-card bin ranges, tech-file grids and Cadence `cdfDump` output.
//!
//! Emits Schemify files; every construct it cannot translate is reported in
//! `needs_review`, never dropped. Research: `docs/research/pdk-xschem-devices.md`.
