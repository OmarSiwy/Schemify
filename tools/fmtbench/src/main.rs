//! fmtbench: size / speed / diff / merge comparison of schematic text formats.
//! Usage: fmtbench [perf|diff|merge|all] [sizes...]

mod fmt;
mod model;

use fmt::*;
use model::*;
use std::fmt::Write as _;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn formats() -> Vec<Box<dyn Format>> {
    let s = |ids, nets, index| -> Box<dyn Format> { Box::new(Sect::new(ids, nets, index, Layout::Line)) };
    vec![
        Box::new(Sexpr { ml: false }),
        Box::new(Sexpr { ml: true }),
        Box::new(Kdl),
        Box::new(Xschem),
        Box::new(Jsonl),
        s(Ids::Random, Nets::Labels, false),
        s(Ids::Seq, Nets::Labels, false),
        s(Ids::Hash, Nets::Labels, false),
        s(Ids::Random, Nets::Wires, false),
        s(Ids::Random, Nets::Table, false),
        s(Ids::Random, Nets::Labels, true),
        Box::new(Sect::new(Ids::Random, Nets::Labels, false, Layout::Blank)),
        Box::new(Sect::new(Ids::Random, Nets::Labels, false, Layout::Block)),
    ]
}

fn best<T>(reps: usize, mut f: impl FnMut() -> T) -> (f64, T) {
    let mut best = f64::MAX;
    let mut out = None;
    for _ in 0..reps {
        let t = Instant::now();
        let r = f();
        best = best.min(t.elapsed().as_secs_f64());
        out = Some(r);
    }
    (best, out.unwrap())
}

/// Library-browser path: read only as much of the file as the symbol section needs.
fn sym_extract(f: &dyn Format, path: &Path) -> Sym {
    if !f.early_exit() {
        let s = std::fs::read_to_string(path).unwrap();
        return f.parse(&s).unwrap().sym;
    }
    let mut cap = 8192;
    loop {
        let mut buf = Vec::with_capacity(cap);
        std::fs::File::open(path).unwrap().take(cap as u64).read_to_end(&mut buf).unwrap();
        if buf.len() < cap { return f.parse(std::str::from_utf8(&buf).unwrap()).unwrap().sym; }
        let cut = buf.iter().rposition(|&c| c == b'\n').map_or(0, |p| p + 1);
        if let Some(sym) = f.parse_sym(std::str::from_utf8(&buf[..cut]).unwrap()) { return sym; }
        cap *= 4;
    }
}

fn selfcheck(fs: &[Box<dyn Format>]) {
    let d = generate(3000, 7);
    for f in fs {
        let a = f.write(&d);
        let p = f.parse(&a).unwrap_or_else(|e| panic!("{}: {e}", f.name()));
        assert_eq!(f.write(&p), a, "{} roundtrip", f.name());
        assert_eq!(p.sym, d.sym, "{} sym", f.name());
        assert_eq!(p.len(), d.len(), "{} len", f.name());
        if f.early_exit() { assert_eq!(f.parse_sym(&a).as_ref(), Some(&d.sym), "{} parse_sym", f.name()); }
    }
    let s = Sect::new(Ids::Random, Nets::Labels, false, Layout::Line);
    let a = s.write(&d);
    assert_eq!(s.write(&s.parse_par(&a, 7).unwrap()), a, "parallel parse");
    let s = Sect::new(Ids::Random, Nets::Labels, false, Layout::Block);
    let a = s.write(&d);
    assert_eq!(s.write(&s.parse_par(&a, 7).unwrap()), a, "parallel parse block");
    eprintln!("selfcheck ok");
}

fn perf(fs: &[Box<dyn Format>], sizes: &[usize], dir: &Path, out: &mut String) {
    writeln!(out, "\n## Performance (release, single thread unless noted; times are best-of-N)\n").unwrap();
    writeln!(out, "| format | objects | size MB | B/obj | lines/obj | write ms | read ms | parse ms | parse MB/s | symbol-only µs |").unwrap();
    writeln!(out, "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|").unwrap();
    for &n in sizes {
        let d = generate(n, 42);
        let reps = match n { ..=10_000 => 10, ..=100_000 => 5, _ => 3 };
        for f in fs {
            let (tw, text) = best(reps, || f.write(&d));
            let path = dir.join(format!("{}_{n}.txt", f.name()));
            std::fs::write(&path, &text).unwrap();
            let (tr, _) = best(reps, || std::fs::read(&path).unwrap().len());
            let (tp, pd) = best(reps, || f.parse(&text).unwrap());
            assert_eq!(pd.len(), d.len());
            drop(pd);
            let (ts, sym) = best(if n >= 1_000_000 && !f.early_exit() { 3 } else { 30 }, || sym_extract(f.as_ref(), &path));
            assert_eq!(sym, d.sym);
            let mb = text.len() as f64 / 1e6;
            let lines = text.bytes().filter(|&b| b == b'\n').count();
            writeln!(out, "| {} | {n} | {mb:.2} | {:.0} | {:.2} | {:.2} | {:.2} | {:.2} | {:.0} | {:.0} |",
                f.name(), text.len() as f64 / n as f64, lines as f64 / n as f64, tw * 1e3, tr * 1e3, tp * 1e3, mb / tp, ts * 1e6).unwrap();
            eprintln!("perf {} {n} done", f.name());
            if f.name() != "sect" { std::fs::remove_file(&path).unwrap(); }
        }
    }
    let n = *sizes.last().unwrap();
    let d = generate(n, 42);
    let s = Sect::new(Ids::Random, Nets::Labels, false, Layout::Line);
    let text = s.write(&d);
    writeln!(out, "\n### Parallel parse of `sect` ({n} objects, {:.1} MB)\n", text.len() as f64 / 1e6).unwrap();
    writeln!(out, "| threads | sect ms | sect-block ms |\n|---:|---:|---:|").unwrap();
    let sb = Sect::new(Ids::Random, Nets::Labels, false, Layout::Block);
    let tb_text = sb.write(&d);
    for t in [1, 2, 4, 8, 16] {
        let (tp, pd) = best(3, || s.parse_par(&text, t).unwrap());
        assert_eq!(pd.len(), d.len());
        let (tq, pd) = best(3, || sb.parse_par(&tb_text, t).unwrap());
        assert_eq!(pd.len(), d.len());
        writeln!(out, "| {t} | {:.1} | {:.1} |", tp * 1e3, tq * 1e3).unwrap();
    }
    drop(tb_text);
    let (tl1, ld) = best(5, || parse_lean(&text).unwrap());
    assert_eq!(ld.insts.len() + ld.wires.len() + ld.labels.len() + ld.texts.len(), d.len());
    drop(ld);
    writeln!(out, "\nZero-copy loader (values borrowed from the buffer, keys interned), 1 thread: {:.1} ms = {:.0} MB/s", tl1 * 1e3, text.len() as f64 / 1e6 / tl1).unwrap();
    let path = dir.join(format!("sect_{n}.txt"));
    let mut r = Rng::new(5);
    let picks: Vec<u32> = (0..200).map(|_| d.wires[r.below(d.wires.len())].id).collect();
    let t0 = Instant::now();
    for &id in &picks { assert!(sect_find(&path, "wire", id).unwrap().is_some()); }
    let tb = t0.elapsed().as_secs_f64() / picks.len() as f64;
    let bpath = dir.join("block.txt");
    std::fs::write(&bpath, Sect::new(Ids::Random, Nets::Labels, false, Layout::Block).write(&d)).unwrap();
    let ipicks: Vec<u32> = (0..200).map(|_| d.insts[r.below(d.insts.len())].id).collect();
    let t0 = Instant::now();
    for &id in &ipicks { assert!(sect_find(&bpath, "inst", id).unwrap().is_some()); }
    let tbb = t0.elapsed().as_secs_f64() / ipicks.len() as f64;
    let (tl, _) = best(3, || {
        let s = std::fs::read_to_string(&path).unwrap();
        let mut pat = String::from("\nwire ");
        b32(picks[0], &mut pat);
        s.find(&pat).unwrap()
    });
    writeln!(out, "\n### Lookup of one record by id in the {n}-object `sect` file (page cache warm)\n").unwrap();
    writeln!(out, "| method | time |\n|---|---:|").unwrap();
    writeln!(out, "| binary search on sorted (kind,id) lines with pread, no index | {:.1} µs |", tb * 1e6).unwrap();
    writeln!(out, "| same, instance lookup in `sect-block` layout (skips indented lines) | {:.1} µs |", tbb * 1e6).unwrap();
    writeln!(out, "| read whole file + substring search | {:.1} ms |", tl * 1e3).unwrap();
    for f in std::fs::read_dir(dir).unwrap() { std::fs::remove_file(f.unwrap().path()).unwrap(); }
}

fn edited(base: &Design, es: &[Edit]) -> Design {
    let mut d = base.clone();
    for e in es { apply(&mut d, e); }
    d
}

/// (lines removed, lines added, hunks) from plain `diff` output.
fn diff_count(a: &Path, b: &Path) -> (usize, usize, usize) {
    let o = Command::new("diff").arg(a).arg(b).output().unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    let c = |p: fn(&str) -> bool| s.lines().filter(|l| p(l)).count();
    (c(|l| l.starts_with('<')), c(|l| l.starts_with('>')), c(|l| l.starts_with(|c: char| c.is_ascii_digit())))
}

fn diffs(fs: &[Box<dyn Format>], n: usize, dir: &Path, out: &mut String) {
    let base = generate(n, 42);
    let mut r = Rng::new(99);
    let edits: Vec<(&str, Vec<Edit>)> = vec![
        ("move inst", vec![Edit::MoveInst(base.insts[123].id, 20)]),
        ("move wire", vec![Edit::MoveWire(base.wires[456].id, 20)]),
        ("rename net (50 users)", vec![Edit::Rename(RENAME_NET.into(), "VREF".into())]),
        ("change 1 param", vec![Edit::SetParam(base.insts[789].id, "w".into(), "3.3u".into())]),
        ("insert inst", vec![Edit::AddInst(new_inst(&base, &mut r))]),
        ("delete wire", vec![Edit::DelWire(base.wires[1011].id)]),
    ];
    writeln!(out, "\n## Diff size per edit ({n}-object design; `diff` output, lines removed/added)\n").unwrap();
    write!(out, "| format |").unwrap();
    for (name, _) in &edits { write!(out, " {name} |").unwrap(); }
    write!(out, "\n|---|").unwrap();
    for _ in &edits { write!(out, "---:|").unwrap(); }
    writeln!(out).unwrap();
    let (pa, pb) = (dir.join("a.txt"), dir.join("b.txt"));
    for f in fs {
        std::fs::write(&pa, f.write(&base)).unwrap();
        write!(out, "| {} |", f.name()).unwrap();
        for (_, es) in &edits {
            std::fs::write(&pb, f.write(&edited(&base, es))).unwrap();
            let (del, add, h) = diff_count(&pa, &pb);
            write!(out, " -{del} +{add}{} |", if h > 1 { format!(" ({h} hunks)") } else { String::new() }).unwrap();
        }
        writeln!(out).unwrap();
    }
}

type Scen = fn(&Design, &mut Rng) -> (Vec<Edit>, Vec<Edit>);

fn two(r: &mut Rng, n: usize) -> (usize, usize) {
    let i = r.below(n);
    let mut j = r.below(n);
    while j == i { j = r.below(n); }
    (i, j)
}

fn merges(fs: &[Box<dyn Format>], n: usize, trials: usize, dir: &Path, out: &mut String) {
    let base = generate(n, 42);
    let scen: Vec<(&str, Scen)> = vec![
        ("move inst X ∥ param on inst Y", |d, r| {
            let (i, j) = two(r, d.insts.len());
            (vec![Edit::MoveInst(d.insts[i].id, 20)], vec![Edit::SetParam(d.insts[j].id, "m".into(), "4".into())])
        }),
        ("insert inst ∥ insert inst", |d, r| (vec![Edit::AddInst(new_inst(d, r))], vec![Edit::AddInst(new_inst(d, r))])),
        ("insert wire ∥ delete wire", |d, r| (vec![Edit::AddWire(new_wire(d, r))], vec![Edit::DelWire(d.wires[r.below(d.wires.len())].id)])),
        ("move inst X ∥ param on same X", |d, r| {
            let id = d.insts[r.below(d.insts.len())].id;
            (vec![Edit::MoveInst(id, 20)], vec![Edit::SetParam(id, "w".into(), "7u".into())])
        }),
        ("rename net ∥ move wire on it", |d, r| {
            let ws: Vec<u32> = d.wires.iter().filter(|w| w.net == RENAME_NET).map(|w| w.id).collect();
            (vec![Edit::Rename(RENAME_NET.into(), "VREF".into())], vec![Edit::MoveWire(ws[r.below(ws.len())], 20)])
        }),
        ("5 wire moves ∥ 5 wire deletes", |d, r| {
            let mut ids: Vec<u32> = Vec::new();
            while ids.len() < 10 { let id = d.wires[r.below(d.wires.len())].id; if !ids.contains(&id) { ids.push(id) } }
            (ids[..5].iter().map(|&i| Edit::MoveWire(i, 20)).collect(), ids[5..].iter().map(|&i| Edit::DelWire(i)).collect())
        }),
        ("10 moves ∥ 10 param edits", |d, r| {
            let mut a = Vec::new();
            for _ in 0..5 { a.push(Edit::MoveInst(d.insts[r.below(d.insts.len())].id, 20)); }
            for _ in 0..5 { a.push(Edit::MoveWire(d.wires[r.below(d.wires.len())].id, 20)); }
            let b = (0..10).map(|_| Edit::SetParam(d.insts[r.below(d.insts.len())].id, "m".into(), "3".into())).collect();
            (a, b)
        }),
    ];
    writeln!(out, "\n## Concurrent-edit merges ({n}-object design, {trials} random trials each, `git merge-file`)\n").unwrap();
    writeln!(out, "Cell = clean&correct / conflict / clean-but-WRONG (merged file parses to something other than both edits applied).\n").unwrap();
    write!(out, "| format |").unwrap();
    for (name, _) in &scen { write!(out, " {name} |").unwrap(); }
    write!(out, "\n|---|").unwrap();
    for _ in &scen { write!(out, "---|").unwrap(); }
    writeln!(out).unwrap();
    let p = |s: &str| -> PathBuf { dir.join(s) };
    // Edits are drawn once per trial and shared by all formats (paired comparison).
    let cases: Vec<Vec<(Vec<Edit>, Vec<Edit>)>> = scen.iter().enumerate()
        .map(|(si, (_, sc))| (0..trials).map(|t| sc(&base, &mut Rng::new(1000 + t as u64 + 7919 * si as u64))).collect())
        .collect();
    for f in fs {
        let tb = f.write(&base);
        std::fs::write(p("base.txt"), &tb).unwrap();
        write!(out, "| {} |", f.name()).unwrap();
        for cs in &cases {
            let (mut ok, mut conf, mut wrong) = (0, 0, 0);
            for (ea, eb) in cs {
                let da = edited(&base, ea);
                let expect = f.write(&edited(&da, eb));
                std::fs::write(p("a.txt"), f.write(&da)).unwrap();
                std::fs::write(p("b.txt"), f.write(&edited(&base, eb))).unwrap();
                let o = Command::new("git").args(["merge-file", "-p"]).arg(p("a.txt")).arg(p("base.txt")).arg(p("b.txt")).output().unwrap();
                match o.status.code() {
                    Some(0) => {
                        let m = String::from_utf8(o.stdout).unwrap();
                        match f.parse(&m) { Ok(pd) if f.write(&pd) == expect => ok += 1, _ => wrong += 1 }
                    }
                    Some(c) if (1..128).contains(&c) => conf += 1,
                    c => panic!("git merge-file failed: {c:?} {}", String::from_utf8_lossy(&o.stderr)),
                }
            }
            write!(out, " {ok} / {conf} / {}{wrong}{} |", if wrong > 0 { "**" } else { "" }, if wrong > 0 { "**" } else { "" }).unwrap();
        }
        writeln!(out).unwrap();
        eprintln!("merge {} done", f.name());
    }
    // Same cases through a semantic, id-keyed merge driver over the `sect` format.
    let f = Sect::new(Ids::Random, Nets::Labels, false, Layout::Line);
    let pb = f.parse(&f.write(&base)).unwrap();
    write!(out, "| sect + merge driver |").unwrap();
    for cs in &cases {
        let (mut ok, mut conf, mut wrong) = (0, 0, 0);
        for (ea, eb) in cs {
            let da = edited(&base, ea);
            let expect = f.write(&edited(&da, eb));
            let (pa, pbb) = (f.parse(&f.write(&da)).unwrap(), f.parse(&f.write(&edited(&base, eb))).unwrap());
            match merge3(&pb, &pa, &pbb) { Some(m) if f.write(&m) == expect => ok += 1, Some(_) => wrong += 1, None => conf += 1 }
        }
        write!(out, " {ok} / {conf} / {wrong} |").unwrap();
    }
    writeln!(out).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("all").to_string();
    let mut sizes: Vec<usize> = args.iter().skip(1).filter_map(|a| a.parse().ok()).collect();
    if sizes.is_empty() { sizes = vec![1_000, 10_000, 100_000, 1_000_000]; }
    let dir = PathBuf::from(format!("work_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fs = formats();
    selfcheck(&fs);
    let mut out = String::new();
    if mode == "sample" {
        let d = generate(40, 1);
        for f in &fs {
            let t = f.write(&d);
            let i = t.find("\ninst").or_else(|| t.find("\n(inst")).or_else(|| t.find("{\"t\":\"inst")).or_else(|| t.find("\nN ")).unwrap_or(0);
            println!("==== {}\n{}", f.name(), t[i..].lines().take(12).collect::<Vec<_>>().join("\n"));
        }
        return;
    }
    if mode == "perf" || mode == "all" { perf(&fs, &sizes, &dir, &mut out); }
    if mode == "diff" || mode == "all" { diffs(&fs, 10_000, &dir, &mut out); }
    if mode == "merge" || mode == "all" {
        merges(&fs, 1_000, 50, &dir, &mut out);
        merges(&fs, 200, 50, &dir, &mut out);
    }
    print!("{out}");
    std::fs::remove_dir_all(&dir).ok();
}
