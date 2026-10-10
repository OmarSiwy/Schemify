//! Candidate on-disk formats. Each has a hand-written writer and parser.
//! All formats (except xschem-like) order records by kind, then by random id,
//! so the syntax comparison is not confounded by ordering policy.

use crate::model::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write as _;

pub trait Format: Sync {
    fn name(&self) -> String;
    fn write(&self, d: &Design) -> String;
    fn parse(&self, s: &str) -> Result<Design, String>;
    /// Symbol section from a prefix that ends on a line boundary.
    /// None = the prefix ended before the symbol section did.
    fn parse_sym(&self, _prefix: &str) -> Option<Sym> { None }
    /// False when the format's spec allows records in any order (must full-scan).
    fn early_exit(&self) -> bool { true }
}

// ---------------------------------------------------------------- shared lexing

const B32: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// Fixed-width 6-char Crockford base32, so string order == numeric order.
pub fn b32(v: u32, o: &mut String) {
    for i in (0..6).rev() { o.push(B32[((v >> (i * 5)) & 31) as usize] as char); }
}
pub fn unb32(s: &str) -> Result<u32, String> {
    if s.len() != 6 { return Err(format!("bad id {s:?}")); }
    let mut v = 0u32;
    for b in s.bytes() {
        let d = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'h' => b - b'a' + 10,
            b'j' | b'k' => b - b'j' + 18,
            b'm' | b'n' => b - b'm' + 20,
            b'p'..=b't' => b - b'p' + 22,
            b'v'..=b'z' => b - b'v' + 27,
            _ => return Err(format!("bad id {s:?}")),
        };
        v = v << 5 | d as u32;
    }
    Ok(v)
}

/// Double-quoted string; escapes \" \\ \n \t \r (JSON-compatible subset).
pub fn q(o: &mut String, s: &str) {
    o.push('"');
    if s.bytes().any(|b| b == b'"' || b == b'\\' || b < 0x20) {
        for c in s.chars() {
            match c {
                '"' => o.push_str("\\\""),
                '\\' => o.push_str("\\\\"),
                '\n' => o.push_str("\\n"),
                '\t' => o.push_str("\\t"),
                '\r' => o.push_str("\\r"),
                c => o.push(c),
            }
        }
    } else {
        o.push_str(s);
    }
    o.push('"');
}

/// `s` starts with '"'. Returns the unescaped value and the rest after the closing quote.
pub fn read_q(s: &str) -> Result<(Cow<'_, str>, &str), String> {
    let b = s.as_bytes();
    let mut i = 1;
    while i < b.len() && b[i] != b'"' && b[i] != b'\\' { i += 1; }
    if i >= b.len() { return Err("unterminated string".into()); }
    if b[i] == b'"' { return Ok((Cow::Borrowed(&s[1..i]), &s[i + 1..])); }
    let mut out = String::from(&s[1..i]);
    while i < b.len() {
        match b[i] {
            b'"' => return Ok((Cow::Owned(out), &s[i + 1..])),
            b'\\' => {
                out.push(match b.get(i + 1) {
                    Some(b'n') => '\n',
                    Some(b't') => '\t',
                    Some(b'r') => '\r',
                    Some(b'"') => '"',
                    Some(b'\\') => '\\',
                    _ => return Err("bad escape".into()),
                });
                i += 2;
            }
            _ => {
                let st = i;
                while i < b.len() && b[i] != b'"' && b[i] != b'\\' { i += 1; }
                out.push_str(&s[st..i]);
            }
        }
    }
    Err("unterminated string".into())
}

/// One whitespace-separated field: `bare`, `"quoted"`, `key=bare` or `key="quoted"`.
pub type Field<'a> = (Option<&'a str>, Cow<'a, str>);
pub fn field<'a>(s: &mut &'a str) -> Result<Option<Field<'a>>, String> {
    let t = s.trim_start_matches([' ', '\t']);
    *s = t;
    if t.is_empty() { return Ok(None); }
    if t.as_bytes()[0] == b'"' {
        let (v, r) = read_q(t)?;
        *s = r;
        return Ok(Some((None, v)));
    }
    let b = t.as_bytes();
    let mut i = 0;
    while i < b.len() && !matches!(b[i], b' ' | b'\t' | b'=' | b'"') { i += 1; }
    if i < b.len() && b[i] == b'=' {
        let (key, r) = (&t[..i], &t[i + 1..]);
        if r.as_bytes().first() == Some(&b'"') {
            let (v, r2) = read_q(r)?;
            *s = r2;
            return Ok(Some((Some(key), v)));
        }
        let e = r.find([' ', '\t']).unwrap_or(r.len());
        *s = &r[e..];
        return Ok(Some((Some(key), Cow::Borrowed(&r[..e]))));
    }
    if i < b.len() && b[i] == b'"' { return Err("quote inside bare token".into()); }
    *s = &t[i..];
    Ok(Some((None, Cow::Borrowed(&t[..i]))))
}
fn pos<'a>(s: &mut &'a str) -> Result<Cow<'a, str>, String> {
    match field(s)? {
        Some((None, v)) => Ok(v),
        Some((Some(k), _)) => Err(format!("unexpected key {k}")),
        None => Err("missing field".into()),
    }
}
fn int(s: &str) -> Result<i32, String> { s.parse().map_err(|_| format!("bad int {s:?}")) }
fn split_kw(t: &str) -> (&str, &str) {
    match t.find([' ', '\t']) { Some(i) => (&t[..i], &t[i..]), None => (t, "") }
}

/// Bare when safe, quoted otherwise (sect + kdl-lite rule).
fn sv(o: &mut String, s: &str) {
    if s.is_empty() || s.starts_with('#') || s.bytes().any(|b| matches!(b, b' ' | b'\t' | b'"' | b'\\' | b'=' | 0..=0x1f)) {
        q(o, s)
    } else {
        o.push_str(s)
    }
}

const ORI: [&str; 8] = ["R0", "R90", "R180", "R270", "MR0", "MR90", "MR180", "MR270"];
fn ori(rot: u8, flip: bool) -> &'static str { ORI[rot as usize + if flip { 4 } else { 0 }] }
fn unori(s: &str) -> Result<(u8, bool), String> {
    ORI.iter().position(|o| *o == s).map(|i| ((i % 4) as u8, i >= 4)).ok_or_else(|| format!("bad orient {s}"))
}

fn by_id<T>(xs: &[T], k: impl Fn(&T) -> u32) -> Vec<&T> {
    let mut v: Vec<&T> = xs.iter().collect();
    v.sort_unstable_by_key(|x| k(x));
    v
}

fn is_schem_kw(kw: &str) -> bool { matches!(kw, "net" | "inst" | "wire" | "label" | "text") }

// ---------------------------------------------------------------- (e) sectioned line records

#[derive(Clone, Copy, PartialEq)]
pub enum Ids { Random, Seq, Hash }
#[derive(Clone, Copy, PartialEq)]
pub enum Nets { Labels, Wires, Table }

/// Line: one record per line. Blank: plus an empty line after every record.
/// Block: instance = header line + indented `at`/param lines.
#[derive(Clone, Copy, PartialEq)]
pub enum Layout { Line, Blank, Block }

pub struct Sect { pub ids: Ids, pub nets: Nets, pub index: bool, pub layout: Layout }
impl Sect {
    pub fn new(ids: Ids, nets: Nets, index: bool, layout: Layout) -> Self { Sect { ids, nets, index, layout } }
}

const IDX_HDR: &str = "schemify 1 shcomp\n";

fn fnv(kw: &str, b: &str) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for x in kw.bytes().chain(b.bytes()) { h ^= x as u64; h = h.wrapping_mul(0x100_0000_01b3); }
    ((h ^ (h >> 32)) as u32) & 0x3fff_ffff
}

impl Sect {
    fn emit<T>(&self, o: &mut String, offs: &mut Vec<usize>, kw: &str, xs: &[T], key: impl Fn(&T) -> (u32, u32), body: impl Fn(&mut String, &T)) {
        if !xs.is_empty() { o.push('\n'); }
        offs.push(o.len());
        match self.ids {
            Ids::Hash => {
                let mut v: Vec<(u32, String)> = xs.iter().map(|t| { let mut b = String::new(); body(&mut b, t); (fnv(kw, &b), b) }).collect();
                v.sort_unstable();
                for (h, b) in v { o.push_str(kw); o.push(' '); b32(h, o); o.push_str(&b); o.push('\n'); if self.layout == Layout::Blank { o.push('\n') } }
            }
            _ => {
                let rnd = self.ids == Ids::Random;
                let mut ix: Vec<(u32, usize)> = xs.iter().enumerate().map(|(i, t)| { let (id, seq) = key(t); (if rnd { id } else { seq }, i) }).collect();
                ix.sort_unstable();
                for (k, i) in ix {
                    o.push_str(kw);
                    o.push(' ');
                    if rnd { b32(k, o) } else { write!(o, "{k}").unwrap() }
                    body(o, &xs[i]);
                    o.push('\n');
                    if self.layout == Layout::Blank { o.push('\n') }
                }
            }
        }
    }

    fn id(&self, t: &str) -> Result<(u32, u32), String> {
        match self.ids { Ids::Seq => Ok((0, t.parse().map_err(|_| format!("bad seq id {t}"))?)), _ => Ok((unb32(t)?, 0)) }
    }

    /// Parses one line into `d`. Stateless per line, so chunks parse in parallel.
    fn line(&self, l: &str, d: &mut Design) -> Result<(), String> {
        let t = l.trim_start();
        if t.is_empty() || t.starts_with('#') { return Ok(()); }
        let (kw, mut s) = split_kw(t);
        let s = &mut s;
        if self.layout == Layout::Block && l.starts_with(' ') {
            let x = d.insts.last_mut().ok_or("continuation line without record")?;
            if kw == "at" {
                (x.x, x.y) = (int(&pos(s)?)?, int(&pos(s)?)?);
                (x.rot, x.flip) = unori(&pos(s)?)?;
            } else {
                x.p.push((kw.to_string(), pos(s)?.into_owned()));
            }
            return Ok(());
        }
        match kw {
            "schemify" | "index" => return Ok(()),
            "pin" => {
                let (name, dir) = (pos(s)?.into_owned(), pos(s)?.into_owned());
                d.sym.pins.push(Pin { name, dir, x: int(&pos(s)?)?, y: int(&pos(s)?)? });
            }
            "param" => { let k = pos(s)?.into_owned(); d.sym.params.push((k, pos(s)?.into_owned())); }
            "bind" => { pos(s)?; d.sym.spice = pos(s)?.into_owned(); }
            "gfx" => {
                let layer = pos(s)?.parse().map_err(|_| "bad layer")?;
                d.sym.gfx.push(Seg { layer, x1: int(&pos(s)?)?, y1: int(&pos(s)?)?, x2: int(&pos(s)?)?, y2: int(&pos(s)?)? });
            }
            "net" => { let (id, _) = self.id(&pos(s)?)?; d.nets.push(Net { id, name: pos(s)?.into_owned() }); }
            "inst" => {
                let (id, seq) = self.id(&pos(s)?)?;
                let sym = pos(s)?.into_owned();
                if self.layout == Layout::Block {
                    d.insts.push(Inst { id, seq, sym, ..Default::default() });
                    return Ok(());
                }
                let (x, y) = (int(&pos(s)?)?, int(&pos(s)?)?);
                let (rot, flip) = unori(&pos(s)?)?;
                let mut p = Vec::with_capacity(6);
                while let Some((k, v)) = field(s)? { p.push((k.ok_or("positional after params")?.to_string(), v.into_owned())); }
                d.insts.push(Inst { id, seq, sym, x, y, rot, flip, p });
            }
            "wire" => {
                let (id, seq) = self.id(&pos(s)?)?;
                let (x1, y1, x2, y2) = (int(&pos(s)?)?, int(&pos(s)?)?, int(&pos(s)?)?, int(&pos(s)?)?);
                let net = if self.nets == Nets::Labels { String::new() } else { pos(s)?.into_owned() };
                d.wires.push(Wire { id, seq, x1, y1, x2, y2, net });
            }
            "label" => {
                let (id, seq) = self.id(&pos(s)?)?;
                let (x, y) = (int(&pos(s)?)?, int(&pos(s)?)?);
                d.labels.push(Label { id, seq, x, y, net: pos(s)?.into_owned() });
            }
            "text" => {
                let (id, seq) = self.id(&pos(s)?)?;
                let (x, y) = (int(&pos(s)?)?, int(&pos(s)?)?);
                d.texts.push(Text { id, seq, x, y, s: pos(s)?.into_owned() });
            }
            _ => return Err(format!("unknown record {kw}")),
        }
        if !s.trim().is_empty() { return Err(format!("trailing fields {s:?}")); }
        Ok(())
    }

    /// Net-table mode stores net ids on wires/labels; resolve them to names.
    fn fix_nets(&self, d: &mut Design) {
        if self.nets != Nets::Table { return; }
        let m: HashMap<String, String> = d.nets.iter().map(|n| { let mut s = String::new(); b32(n.id, &mut s); (s, n.name.clone()) }).collect();
        for w in &mut d.wires { w.net = m.get(&w.net).cloned().unwrap_or_default(); }
        for l in &mut d.labels { l.net = m.get(&l.net).cloned().unwrap_or_default(); }
    }

    /// Splits at line boundaries and parses chunks on `threads` threads.
    pub fn parse_par(&self, s: &str, threads: usize) -> Result<Design, String> {
        let b = s.as_bytes();
        let mut cuts = vec![0usize];
        for t in 1..threads {
            let mut p = (s.len() * t / threads).max(*cuts.last().unwrap()).max(1);
            while p < b.len() && (b[p - 1] != b'\n' || b[p] == b' ') { p += 1; }
            cuts.push(p);
        }
        cuts.push(s.len());
        let parts: Vec<Result<Design, String>> = std::thread::scope(|sc| {
            let hs: Vec<_> = cuts.windows(2).map(|w| {
                let c = &s[w[0]..w[1]];
                sc.spawn(move || -> Result<Design, String> {
                    let mut d = Design::default();
                    for l in c.lines() { self.line(l, &mut d)?; }
                    Ok(d)
                })
            }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut d = Design::default();
        for p in parts {
            let p = p?;
            d.sym.pins.extend(p.sym.pins);
            d.sym.params.extend(p.sym.params);
            d.sym.gfx.extend(p.sym.gfx);
            if !p.sym.spice.is_empty() { d.sym.spice = p.sym.spice; }
            d.nets.extend(p.nets);
            d.insts.extend(p.insts);
            d.wires.extend(p.wires);
            d.labels.extend(p.labels);
            d.texts.extend(p.texts);
        }
        self.fix_nets(&mut d);
        Ok(d)
    }
}

impl Format for Sect {
    fn name(&self) -> String {
        let mut n = String::from("sect");
        n += match self.ids { Ids::Random => "", Ids::Seq => "-seqid", Ids::Hash => "-hashid" };
        n += match self.nets { Nets::Labels => "", Nets::Wires => "-wirenet", Nets::Table => "-nettable" };
        if self.index { n += "-offsetidx"; }
        n += match self.layout { Layout::Line => "", Layout::Blank => "-blank", Layout::Block => "-block" };
        n
    }

    fn write(&self, d: &Design) -> String {
        let mut o = String::with_capacity(d.len() * 64 + 4096);
        o.push_str(IDX_HDR);
        let idx = |offs: &[usize]| format!("index net={:012} inst={:012} wire={:012} label={:012} text={:012}\n", offs[0], offs[1], offs[2], offs[3], offs[4]);
        if self.index { o.push_str(&idx(&[0; 5])); }
        let s = &d.sym;
        for p in &s.pins { o.push_str("pin "); sv(&mut o, &p.name); writeln!(o, " {} {} {}", p.dir, p.x, p.y).unwrap(); }
        for (k, v) in &s.params { o.push_str("param "); sv(&mut o, k); o.push(' '); sv(&mut o, v); o.push('\n'); }
        o.push_str("bind spice "); sv(&mut o, &s.spice); o.push('\n');
        for g in &s.gfx { writeln!(o, "gfx {} {} {} {} {}", g.layer, g.x1, g.y1, g.x2, g.y2).unwrap(); }

        let tbl: HashMap<&str, u32> = d.nets.iter().map(|n| (n.name.as_str(), n.id)).collect();
        let nm = |o: &mut String, net: &str| {
            o.push(' ');
            if self.nets == Nets::Table { b32(tbl[net], o) } else { sv(o, net) }
        };
        let mut offs = Vec::new();
        let no_nets: &[Net] = if self.nets == Nets::Table { &d.nets } else { &[] };
        self.emit(&mut o, &mut offs, "net", no_nets, |n| (n.id, n.id), |o, n| { o.push(' '); sv(o, &n.name) });
        self.emit(&mut o, &mut offs, "inst", &d.insts, |x| (x.id, x.seq), |o, x| {
            o.push(' ');
            sv(o, &x.sym);
            if self.layout == Layout::Block {
                write!(o, "\n  at {} {} {}", x.x, x.y, ori(x.rot, x.flip)).unwrap();
                for (k, v) in &x.p { o.push_str("\n  "); o.push_str(k); o.push(' '); sv(o, v); }
                return;
            }
            write!(o, " {} {} {}", x.x, x.y, ori(x.rot, x.flip)).unwrap();
            for (k, v) in &x.p { o.push(' '); o.push_str(k); o.push('='); sv(o, v); }
        });
        self.emit(&mut o, &mut offs, "wire", &d.wires, |x| (x.id, x.seq), |o, w| {
            write!(o, " {} {} {} {}", w.x1, w.y1, w.x2, w.y2).unwrap();
            if self.nets != Nets::Labels { nm(o, &w.net) }
        });
        self.emit(&mut o, &mut offs, "label", &d.labels, |x| (x.id, x.seq), |o, l| { write!(o, " {} {}", l.x, l.y).unwrap(); nm(o, &l.net) });
        self.emit(&mut o, &mut offs, "text", &d.texts, |x| (x.id, x.seq), |o, t| { write!(o, " {} {} ", t.x, t.y).unwrap(); sv(o, &t.s) });
        if self.index {
            let h = idx(&offs);
            o.replace_range(IDX_HDR.len()..IDX_HDR.len() + h.len(), &h);
        }
        o
    }

    fn parse(&self, s: &str) -> Result<Design, String> {
        let mut d = Design::default();
        for (i, l) in s.lines().enumerate() { self.line(l, &mut d).map_err(|e| format!("line {}: {e}", i + 1))?; }
        self.fix_nets(&mut d);
        Ok(d)
    }

    fn parse_sym(&self, prefix: &str) -> Option<Sym> {
        let mut d = Design::default();
        for l in prefix.lines() {
            if is_schem_kw(split_kw(l.trim_start()).0) { return Some(d.sym); }
            self.line(l, &mut d).ok()?;
        }
        None
    }
}

/// Binary search for `kw id` in a sect file (records sorted by (kind, id)), using
/// positioned reads only: no index, no full read.
pub fn sect_find(path: &std::path::Path, kw: &str, id: u32) -> std::io::Result<Option<String>> {
    use std::os::unix::fs::FileExt;
    let rank = |k: &[u8]| -> u8 { match k { b"net" => 1, b"inst" => 2, b"wire" => 3, b"label" => 4, b"text" => 5, _ => 0 } };
    let target = (rank(kw.as_bytes()), id);
    let f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    let (mut lo, mut hi) = (0u64, len);
    let mut buf = vec![0u8; 4096];
    while hi - lo > 16384 {
        let mid = lo + (hi - lo) / 2;
        let n = f.read_at(&mut buf, mid)?;
        let w = &buf[..n];
        let Some(nl) = w.iter().position(|&c| c == b'\n') else { hi = mid; continue };
        // Next record start: skip blank lines and indented continuation lines.
        let mut st = nl + 1;
        while st < n && (w[st] == b'\n' || w[st] == b' ') {
            st += if w[st] == b'\n' { 1 } else { w[st..].iter().position(|&c| c == b'\n').map_or(n - st, |p| p + 1) };
        }
        if st >= n { hi = mid; continue; }
        let line = &w[st..];
        let sp = line.iter().position(|&c| c == b' ').unwrap_or(line.len());
        let r = rank(&line[..sp]);
        let k = if r == 0 { (0, 0) } else {
            (r, line.get(sp + 1..sp + 7).and_then(|x| unb32(std::str::from_utf8(x).ok()?).ok()).unwrap_or(0))
        };
        let start = mid + st as u64;
        if k < target { lo = start } else { hi = start }
    }
    let end = (hi + 4096).min(len);
    let mut chunk = vec![0u8; (end - lo) as usize];
    f.read_exact_at(&mut chunk, lo)?;
    let mut pat = format!("{kw} ");
    b32(id, &mut pat);
    pat.push(' ');
    let text = String::from_utf8_lossy(&chunk);
    Ok(text.lines().find(|l| l.starts_with(&pat)).map(str::to_string))
}

// ---------------------------------------------------------------- (a) S-expressions

pub struct Sexpr { pub ml: bool }

struct Sx<'a> { s: &'a str, i: usize }
enum Tk<'a> { L, R, A(&'a str), S(Cow<'a, str>), Eof }

impl<'a> Sx<'a> {
    fn tok(&mut self) -> Result<Tk<'a>, String> {
        let b = self.s.as_bytes();
        while self.i < b.len() && b[self.i].is_ascii_whitespace() { self.i += 1; }
        if self.i >= b.len() { return Ok(Tk::Eof); }
        match b[self.i] {
            b'(' => { self.i += 1; Ok(Tk::L) }
            b')' => { self.i += 1; Ok(Tk::R) }
            b'"' => { let (v, rest) = read_q(&self.s[self.i..])?; self.i = self.s.len() - rest.len(); Ok(Tk::S(v)) }
            _ => {
                let st = self.i;
                while self.i < b.len() && !b[self.i].is_ascii_whitespace() && b[self.i] != b'(' && b[self.i] != b')' { self.i += 1; }
                Ok(Tk::A(&self.s[st..self.i]))
            }
        }
    }
    fn atom(&mut self) -> Result<&'a str, String> { match self.tok()? { Tk::A(a) => Ok(a), _ => Err("atom expected".into()) } }
    fn str(&mut self) -> Result<Cow<'a, str>, String> { match self.tok()? { Tk::S(v) => Ok(v), Tk::A(a) => Ok(Cow::Borrowed(a)), _ => Err("string expected".into()) } }
    fn int(&mut self) -> Result<i32, String> { int(self.atom()?) }
    fn close(&mut self) -> Result<(), String> { match self.tok()? { Tk::R => Ok(()), _ => Err("')' expected".into()) } }
    /// Head of the next child list, or None at the parent's closing paren.
    fn child(&mut self) -> Result<Option<&'a str>, String> {
        match self.tok()? { Tk::L => Ok(Some(self.atom()?)), Tk::R => Ok(None), _ => Err("child expected".into()) }
    }
    fn xy(&mut self) -> Result<(i32, i32), String> {
        if self.child()? != Some("xy") { return Err("xy expected".into()); }
        let r = (self.int()?, self.int()?);
        self.close()?;
        Ok(r)
    }

    fn form(&mut self, head: &str, d: &mut Design) -> Result<(), String> {
        match head {
            "schemify" => loop { match self.tok()? { Tk::R => break, Tk::Eof => return Err("eof".into()), _ => {} } },
            "pin" => {
                let (name, dir) = (self.str()?.into_owned(), self.atom()?.to_string());
                if self.child()? != Some("at") { return Err("at expected".into()); }
                let (x, y) = (self.int()?, self.int()?);
                self.close()?;
                self.close()?;
                d.sym.pins.push(Pin { name, dir, x, y });
            }
            "param" => { let k = self.str()?.into_owned(); d.sym.params.push((k, self.str()?.into_owned())); self.close()?; }
            "bind" => { self.atom()?; d.sym.spice = self.str()?.into_owned(); self.close()?; }
            "gfx" => {
                let layer = self.int()? as u8;
                let ((x1, y1), (x2, y2)) = (self.xy()?, self.xy()?);
                self.close()?;
                d.sym.gfx.push(Seg { layer, x1, y1, x2, y2 });
            }
            "inst" => {
                let mut x = Inst::default();
                while let Some(h) = self.child()? {
                    match h {
                        "lib_id" => x.sym = self.str()?.into_owned(),
                        "at" => { x.x = self.int()?; x.y = self.int()?; x.rot = (self.int()? / 90) as u8; }
                        "mirror" => { self.atom()?; x.flip = true; }
                        "uuid" => x.id = unb32(&self.str()?)?,
                        "prop" => { let k = self.str()?.into_owned(); x.p.push((k, self.str()?.into_owned())); }
                        _ => return Err(format!("unknown {h}")),
                    }
                    self.close()?;
                }
                d.insts.push(x);
            }
            "wire" => {
                let mut w = Wire::default();
                while let Some(h) = self.child()? {
                    match h {
                        "pts" => { (w.x1, w.y1) = self.xy()?; (w.x2, w.y2) = self.xy()?; }
                        "uuid" => w.id = unb32(&self.str()?)?,
                        _ => return Err(format!("unknown {h}")),
                    }
                    self.close()?;
                }
                d.wires.push(w);
            }
            "label" | "text" => {
                let (mut id, mut x, mut y) = (0, 0, 0);
                let s = self.str()?.into_owned();
                while let Some(h) = self.child()? {
                    match h {
                        "at" => { x = self.int()?; y = self.int()?; self.int()?; }
                        "uuid" => id = unb32(&self.str()?)?,
                        _ => return Err(format!("unknown {h}")),
                    }
                    self.close()?;
                }
                if head == "label" { d.labels.push(Label { id, seq: 0, x, y, net: s }) } else { d.texts.push(Text { id, seq: 0, x, y, s }) }
            }
            _ => return Err(format!("unknown form {head}")),
        }
        Ok(())
    }
}

impl Format for Sexpr {
    fn name(&self) -> String { if self.ml { "sexpr-kicad".into() } else { "sexpr-1line".into() } }

    fn write(&self, d: &Design) -> String {
        let mut o = String::with_capacity(d.len() * 90 + 4096);
        let (sep, close) = if self.ml { ("\n\t", "\n)\n") } else { (" ", ")\n") };
        o.push_str("(schemify 1 shcomp)\n");
        for p in &d.sym.pins { o.push_str("(pin "); q(&mut o, &p.name); writeln!(o, " {} (at {} {}))", p.dir, p.x, p.y).unwrap(); }
        for (k, v) in &d.sym.params { o.push_str("(param "); q(&mut o, k); o.push(' '); q(&mut o, v); o.push_str(")\n"); }
        o.push_str("(bind spice "); q(&mut o, &d.sym.spice); o.push_str(")\n");
        for g in &d.sym.gfx { writeln!(o, "(gfx {} (xy {} {}) (xy {} {}))", g.layer, g.x1, g.y1, g.x2, g.y2).unwrap(); }
        let uuid = |o: &mut String, id: u32| { o.push_str(sep); o.push_str("(uuid \""); b32(id, o); o.push_str("\")"); };
        for x in by_id(&d.insts, |x| x.id) {
            o.push_str("(inst");
            o.push_str(sep);
            o.push_str("(lib_id ");
            q(&mut o, &x.sym);
            write!(o, "){sep}(at {} {} {})", x.x, x.y, x.rot as u32 * 90).unwrap();
            if x.flip { write!(o, "{sep}(mirror y)").unwrap(); }
            uuid(&mut o, x.id);
            for (k, v) in &x.p { o.push_str(sep); o.push_str("(prop "); q(&mut o, k); o.push(' '); q(&mut o, v); o.push(')'); }
            o.push_str(close);
        }
        for w in by_id(&d.wires, |x| x.id) {
            write!(o, "(wire{sep}(pts (xy {} {}) (xy {} {}))", w.x1, w.y1, w.x2, w.y2).unwrap();
            uuid(&mut o, w.id);
            o.push_str(close);
        }
        for l in by_id(&d.labels, |x| x.id) {
            o.push_str("(label ");
            q(&mut o, &l.net);
            write!(o, "{sep}(at {} {} 0)", l.x, l.y).unwrap();
            uuid(&mut o, l.id);
            o.push_str(close);
        }
        for t in by_id(&d.texts, |x| x.id) {
            o.push_str("(text ");
            q(&mut o, &t.s);
            write!(o, "{sep}(at {} {} 0)", t.x, t.y).unwrap();
            uuid(&mut o, t.id);
            o.push_str(close);
        }
        o
    }

    fn parse(&self, s: &str) -> Result<Design, String> {
        let mut p = Sx { s, i: 0 };
        let mut d = Design::default();
        loop {
            match p.tok()? {
                Tk::Eof => break,
                Tk::L => { let h = p.atom()?; p.form(h, &mut d).map_err(|e| format!("@{}: {e}", p.i))?; }
                _ => return Err(format!("form expected @{}", p.i)),
            }
        }
        Ok(d)
    }

    fn parse_sym(&self, prefix: &str) -> Option<Sym> {
        let mut p = Sx { s: prefix, i: 0 };
        let mut d = Design::default();
        loop {
            match p.tok().ok()? {
                Tk::L => {
                    let h = p.atom().ok()?;
                    if is_schem_kw(h) { return Some(d.sym); }
                    p.form(h, &mut d).ok()?;
                }
                _ => return None,
            }
        }
    }
}

// ---------------------------------------------------------------- (b) KDL v2 style

pub struct Kdl;

/// KDL v2 identifier-string if possible, else quoted.
fn kd(o: &mut String, s: &str) {
    let b = s.as_bytes();
    let ident = !b.is_empty()
        && (b[0].is_ascii_alphabetic() || b[0] == b'_')
        && b.iter().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.'))
        && !matches!(s, "true" | "false" | "null" | "inf" | "nan");
    if ident { o.push_str(s) } else { q(o, s) }
}

impl Format for Kdl {
    fn name(&self) -> String { "kdl".into() }

    fn write(&self, d: &Design) -> String {
        let mut o = String::with_capacity(d.len() * 90 + 4096);
        o.push_str("schemify version=1 kind=shcomp\n");
        for p in &d.sym.pins { o.push_str("pin "); kd(&mut o, &p.name); o.push_str(" dir="); kd(&mut o, &p.dir); writeln!(o, " x={} y={}", p.x, p.y).unwrap(); }
        for (k, v) in &d.sym.params { o.push_str("param "); kd(&mut o, k); o.push(' '); kd(&mut o, v); o.push('\n'); }
        o.push_str("bind spice "); q(&mut o, &d.sym.spice); o.push('\n');
        for g in &d.sym.gfx { writeln!(o, "gfx {} {} {} {} {}", g.layer, g.x1, g.y1, g.x2, g.y2).unwrap(); }
        let id = |o: &mut String, kw: &str, id: u32| { o.push_str(kw); o.push_str(" \""); b32(id, o); o.push('"'); };
        o.push('\n');
        for x in by_id(&d.insts, |x| x.id) {
            id(&mut o, "inst", x.id);
            o.push_str(" sym=");
            kd(&mut o, &x.sym);
            write!(o, " x={} y={} rot={}", x.x, x.y, x.rot as u32 * 90).unwrap();
            if x.flip { o.push_str(" flip=#true"); }
            o.push_str(" {\n");
            for (k, v) in &x.p { o.push_str("    "); kd(&mut o, k); o.push(' '); kd(&mut o, v); o.push('\n'); }
            o.push_str("}\n");
        }
        o.push('\n');
        for w in by_id(&d.wires, |x| x.id) { id(&mut o, "wire", w.id); writeln!(o, " {} {} {} {}", w.x1, w.y1, w.x2, w.y2).unwrap(); }
        o.push('\n');
        for l in by_id(&d.labels, |x| x.id) { id(&mut o, "label", l.id); o.push(' '); kd(&mut o, &l.net); writeln!(o, " x={} y={}", l.x, l.y).unwrap(); }
        o.push('\n');
        for t in by_id(&d.texts, |x| x.id) { id(&mut o, "text", t.id); o.push(' '); q(&mut o, &t.s); writeln!(o, " x={} y={}", t.x, t.y).unwrap(); }
        o
    }

    fn parse(&self, s: &str) -> Result<Design, String> { kdl_parse(s, false).map(|r| r.0) }
    fn parse_sym(&self, prefix: &str) -> Option<Sym> {
        match kdl_parse(prefix, true) { Ok((d, true)) => Some(d.sym), _ => None }
    }
}

/// Line-oriented KDL subset parser (one node per line, `{`/`}` for children).
/// Returns (design, stopped_at_schematic).
fn kdl_parse(s: &str, sym_only: bool) -> Result<(Design, bool), String> {
    let mut d = Design::default();
    let mut cur: Option<Inst> = None;
    for l in s.lines() {
        let t = l.trim();
        if t.is_empty() || t.starts_with("//") { continue; }
        if let Some(x) = cur.as_mut() {
            if t == "}" { d.insts.push(cur.take().unwrap()); continue; }
            let mut s = t;
            let k = pos(&mut s)?.into_owned();
            x.p.push((k, pos(&mut s)?.into_owned()));
            continue;
        }
        let (t, open) = match t.strip_suffix(" {") { Some(t) => (t, true), None => (t, false) };
        let (kw, mut s) = split_kw(t);
        if sym_only && is_schem_kw(kw) { return Ok((d, true)); }
        let s = &mut s;
        match kw {
            "schemify" => {}
            "pin" => {
                let mut p = Pin { name: pos(s)?.into_owned(), ..Default::default() };
                while let Some((k, v)) = field(s)? {
                    match k { Some("dir") => p.dir = v.into_owned(), Some("x") => p.x = int(&v)?, Some("y") => p.y = int(&v)?, _ => return Err("bad pin prop".into()) }
                }
                d.sym.pins.push(p);
            }
            "param" => { let k = pos(s)?.into_owned(); d.sym.params.push((k, pos(s)?.into_owned())); }
            "bind" => { pos(s)?; d.sym.spice = pos(s)?.into_owned(); }
            "gfx" => {
                let layer = pos(s)?.parse().map_err(|_| "bad layer")?;
                d.sym.gfx.push(Seg { layer, x1: int(&pos(s)?)?, y1: int(&pos(s)?)?, x2: int(&pos(s)?)?, y2: int(&pos(s)?)? });
            }
            "inst" => {
                let mut x = Inst { id: unb32(&pos(s)?)?, ..Default::default() };
                while let Some((k, v)) = field(s)? {
                    match k {
                        Some("sym") => x.sym = v.into_owned(),
                        Some("x") => x.x = int(&v)?,
                        Some("y") => x.y = int(&v)?,
                        Some("rot") => x.rot = (int(&v)? / 90) as u8,
                        Some("flip") => x.flip = v == "#true",
                        _ => return Err("bad inst prop".into()),
                    }
                }
                if open { cur = Some(x) } else { d.insts.push(x) }
            }
            "wire" => {
                let id = unb32(&pos(s)?)?;
                d.wires.push(Wire { id, seq: 0, x1: int(&pos(s)?)?, y1: int(&pos(s)?)?, x2: int(&pos(s)?)?, y2: int(&pos(s)?)?, net: String::new() });
            }
            "label" | "text" => {
                let id = unb32(&pos(s)?)?;
                let v = pos(s)?.into_owned();
                let (mut x, mut y) = (0, 0);
                while let Some((k, val)) = field(s)? {
                    match k { Some("x") => x = int(&val)?, Some("y") => y = int(&val)?, _ => return Err("bad prop".into()) }
                }
                if kw == "label" { d.labels.push(Label { id, seq: 0, x, y, net: v }) } else { d.texts.push(Text { id, seq: 0, x, y, s: v }) }
            }
            _ => return Err(format!("unknown node {kw}")),
        }
    }
    Ok((d, false))
}

// ---------------------------------------------------------------- (d) JSON Lines

pub struct Jsonl;

enum J<'a> { S(Cow<'a, str>), N(i64), B(bool), O(Vec<(Cow<'a, str>, J<'a>)>), Null }
struct Jp<'a> { s: &'a str, i: usize }

impl<'a> Jp<'a> {
    fn ws(&mut self) {
        let b = self.s.as_bytes();
        while self.i < b.len() && b[self.i].is_ascii_whitespace() { self.i += 1; }
    }
    fn lit(&mut self, w: &str, v: J<'a>) -> Result<J<'a>, String> {
        if self.s[self.i..].starts_with(w) { self.i += w.len(); Ok(v) } else { Err("bad literal".into()) }
    }
    fn val(&mut self) -> Result<J<'a>, String> {
        self.ws();
        let b = self.s.as_bytes();
        match b.get(self.i) {
            Some(b'"') => { let (v, r) = read_q(&self.s[self.i..])?; self.i = self.s.len() - r.len(); Ok(J::S(v)) }
            Some(b'{') => {
                self.i += 1;
                let mut v = Vec::with_capacity(8);
                self.ws();
                if b.get(self.i) == Some(&b'}') { self.i += 1; return Ok(J::O(v)); }
                loop {
                    let J::S(k) = self.val()? else { return Err("key expected".into()) };
                    self.ws();
                    if b.get(self.i) != Some(&b':') { return Err("':' expected".into()); }
                    self.i += 1;
                    let x = self.val()?;
                    v.push((k, x));
                    self.ws();
                    match b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => { self.i += 1; return Ok(J::O(v)); }
                        _ => return Err("',' or '}' expected".into()),
                    }
                }
            }
            Some(b't') => self.lit("true", J::B(true)),
            Some(b'f') => self.lit("false", J::B(false)),
            Some(b'n') => self.lit("null", J::Null),
            Some(_) => {
                let st = self.i;
                while self.i < b.len() && matches!(b[self.i], b'-' | b'0'..=b'9') { self.i += 1; }
                self.s[st..self.i].parse().map(J::N).map_err(|_| "bad number".into())
            }
            None => Err("eof".into()),
        }
    }
}

type Obj<'a> = Vec<(Cow<'a, str>, J<'a>)>;
fn get<'b, 'a>(o: &'b Obj<'a>, k: &str) -> Result<&'b J<'a>, String> {
    o.iter().find(|x| x.0 == k).map(|x| &x.1).ok_or_else(|| format!("missing {k}"))
}
fn gs<'b>(o: &'b Obj, k: &str) -> Result<&'b str, String> { match get(o, k)? { J::S(s) => Ok(s), _ => Err(format!("{k} not string")) } }
fn gi(o: &Obj, k: &str) -> Result<i32, String> { match get(o, k)? { J::N(n) => Ok(*n as i32), _ => Err(format!("{k} not int")) } }

impl Jsonl {
    fn line(&self, l: &str, d: &mut Design, sym_only: bool) -> Result<bool, String> {
        if l.trim().is_empty() { return Ok(false); }
        let J::O(o) = (Jp { s: l, i: 0 }).val()? else { return Err("object expected".into()) };
        let t = gs(&o, "t")?;
        if sym_only && is_schem_kw(t) { return Ok(true); }
        match t {
            "schemify" => {}
            "pin" => d.sym.pins.push(Pin { name: gs(&o, "name")?.into(), dir: gs(&o, "dir")?.into(), x: gi(&o, "x")?, y: gi(&o, "y")? }),
            "param" => d.sym.params.push((gs(&o, "k")?.into(), gs(&o, "v")?.into())),
            "bind" => d.sym.spice = gs(&o, "v")?.into(),
            "gfx" => d.sym.gfx.push(Seg { layer: gi(&o, "layer")? as u8, x1: gi(&o, "x1")?, y1: gi(&o, "y1")?, x2: gi(&o, "x2")?, y2: gi(&o, "y2")? }),
            "inst" => {
                let J::O(p) = get(&o, "p")? else { return Err("p not object".into()) };
                let p = p.iter().map(|(k, v)| match v { J::S(v) => Ok((k.to_string(), v.to_string())), _ => Err("param not string".to_string()) }).collect::<Result<_, _>>()?;
                let J::B(flip) = get(&o, "flip")? else { return Err("flip".into()) };
                d.insts.push(Inst { id: unb32(gs(&o, "id")?)?, seq: 0, sym: gs(&o, "sym")?.into(), x: gi(&o, "x")?, y: gi(&o, "y")?, rot: gi(&o, "rot")? as u8, flip: *flip, p });
            }
            "wire" => d.wires.push(Wire { id: unb32(gs(&o, "id")?)?, seq: 0, x1: gi(&o, "x1")?, y1: gi(&o, "y1")?, x2: gi(&o, "x2")?, y2: gi(&o, "y2")?, net: String::new() }),
            "label" => d.labels.push(Label { id: unb32(gs(&o, "id")?)?, seq: 0, x: gi(&o, "x")?, y: gi(&o, "y")?, net: gs(&o, "net")?.into() }),
            "text" => d.texts.push(Text { id: unb32(gs(&o, "id")?)?, seq: 0, x: gi(&o, "x")?, y: gi(&o, "y")?, s: gs(&o, "s")?.into() }),
            _ => return Err(format!("unknown t {t}")),
        }
        Ok(false)
    }
}

impl Format for Jsonl {
    fn name(&self) -> String { "jsonl".into() }

    fn write(&self, d: &Design) -> String {
        let mut o = String::with_capacity(d.len() * 110 + 4096);
        o.push_str("{\"t\":\"schemify\",\"v\":1,\"kind\":\"shcomp\"}\n");
        for p in &d.sym.pins { o.push_str("{\"t\":\"pin\",\"name\":"); q(&mut o, &p.name); o.push_str(",\"dir\":"); q(&mut o, &p.dir); writeln!(o, ",\"x\":{},\"y\":{}}}", p.x, p.y).unwrap(); }
        for (k, v) in &d.sym.params { o.push_str("{\"t\":\"param\",\"k\":"); q(&mut o, k); o.push_str(",\"v\":"); q(&mut o, v); o.push_str("}\n"); }
        o.push_str("{\"t\":\"bind\",\"tool\":\"spice\",\"v\":"); q(&mut o, &d.sym.spice); o.push_str("}\n");
        for g in &d.sym.gfx { writeln!(o, "{{\"t\":\"gfx\",\"layer\":{},\"x1\":{},\"y1\":{},\"x2\":{},\"y2\":{}}}", g.layer, g.x1, g.y1, g.x2, g.y2).unwrap(); }
        let id = |o: &mut String, t: &str, id: u32| { write!(o, "{{\"t\":\"{t}\",\"id\":\"").unwrap(); b32(id, o); o.push('"'); };
        for x in by_id(&d.insts, |x| x.id) {
            id(&mut o, "inst", x.id);
            o.push_str(",\"sym\":");
            q(&mut o, &x.sym);
            write!(o, ",\"x\":{},\"y\":{},\"rot\":{},\"flip\":{},\"p\":{{", x.x, x.y, x.rot, x.flip).unwrap();
            for (i, (k, v)) in x.p.iter().enumerate() { if i > 0 { o.push(','); } q(&mut o, k); o.push(':'); q(&mut o, v); }
            o.push_str("}}\n");
        }
        for w in by_id(&d.wires, |x| x.id) { id(&mut o, "wire", w.id); writeln!(o, ",\"x1\":{},\"y1\":{},\"x2\":{},\"y2\":{}}}", w.x1, w.y1, w.x2, w.y2).unwrap(); }
        for l in by_id(&d.labels, |x| x.id) { id(&mut o, "label", l.id); write!(o, ",\"x\":{},\"y\":{},\"net\":", l.x, l.y).unwrap(); q(&mut o, &l.net); o.push_str("}\n"); }
        for t in by_id(&d.texts, |x| x.id) { id(&mut o, "text", t.id); write!(o, ",\"x\":{},\"y\":{},\"s\":", t.x, t.y).unwrap(); q(&mut o, &t.s); o.push_str("}\n"); }
        o
    }

    fn parse(&self, s: &str) -> Result<Design, String> {
        let mut d = Design::default();
        for (i, l) in s.lines().enumerate() { self.line(l, &mut d, false).map_err(|e| format!("line {}: {e}", i + 1))?; }
        Ok(d)
    }
    fn parse_sym(&self, prefix: &str) -> Option<Sym> {
        let mut d = Design::default();
        for l in prefix.lines() { if self.line(l, &mut d, true).ok()? { return Some(d.sym); } }
        None
    }
}

// ---------------------------------------------------------------- (c) xschem-like

pub struct Xschem;

/// xschem brace string: escape \ { }.
fn be(o: &mut String, s: &str) {
    o.push('{');
    for c in s.chars() { if matches!(c, '\\' | '{' | '}') { o.push('\\') } o.push(c) }
    o.push('}');
}
/// xschem attribute value: quote when it has separators, escape " and \.
fn av(o: &mut String, s: &str) {
    if s.is_empty() { return o.push_str("\"\""); }
    if !s.bytes().any(|b| matches!(b, b' ' | b'\t' | b'\n' | b';' | b'"' | b'\\')) { return o.push_str(s); }
    o.push('"');
    for c in s.chars() { if c == '"' || c == '\\' { o.push('\\') } o.push(c) }
    o.push('"');
}
fn attrs<'a>(kv: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    let mut o = String::new();
    for (k, v) in kv { if !o.is_empty() { o.push(' ') } o.push_str(k); o.push('='); av(&mut o, v); }
    o
}
/// xschem `get_tok_value` semantics (with_quotes=0): `;`/space/tab/newline separate,
/// `"` toggles quoting, `\` escapes the next char.
fn attr_parse(s: &str) -> Kv {
    let sep = |c: char| matches!(c, ' ' | '\t' | '\n' | ';');
    let mut out = Vec::new();
    let mut it = s.chars().peekable();
    loop {
        while it.peek().is_some_and(|&c| sep(c)) { it.next(); }
        if it.peek().is_none() { break; }
        let mut k = String::new();
        while let Some(&c) = it.peek() { if c == '=' || sep(c) { break } k.push(c); it.next(); }
        let mut v = String::new();
        if it.peek() == Some(&'=') {
            it.next();
            let mut quoted = false;
            while let Some(&c) = it.peek() {
                match c {
                    '\\' => { it.next(); if let Some(c) = it.next() { v.push(c) } }
                    '"' => { quoted = !quoted; it.next(); }
                    c if !quoted && sep(c) => break,
                    c => { v.push(c); it.next(); }
                }
            }
        }
        out.push((k, v));
    }
    out
}
fn aget(a: &Kv, k: &str) -> String { a.iter().find(|x| x.0 == k).map(|x| x.1.clone()).unwrap_or_default() }

struct Xs<'a> { s: &'a str, i: usize }
impl<'a> Xs<'a> {
    fn ws(&mut self) { let b = self.s.as_bytes(); while self.i < b.len() && b[self.i].is_ascii_whitespace() { self.i += 1; } }
    fn word(&mut self) -> Result<&'a str, String> {
        self.ws();
        let b = self.s.as_bytes();
        let st = self.i;
        while self.i < b.len() && !b[self.i].is_ascii_whitespace() && b[self.i] != b'{' { self.i += 1; }
        if st == self.i { Err(format!("field expected @{}", self.i)) } else { Ok(&self.s[st..self.i]) }
    }
    fn int(&mut self) -> Result<i32, String> { int(self.word()?) }
    fn brace(&mut self) -> Result<Cow<'a, str>, String> {
        let b = self.s.as_bytes();
        while self.i < b.len() && b[self.i] != b'{' { self.i += 1; }
        self.i += 1;
        let st = self.i;
        while self.i < b.len() && b[self.i] != b'}' && b[self.i] != b'\\' { self.i += 1; }
        if self.i >= b.len() { return Err("eof in {}".into()); }
        if b[self.i] == b'}' { self.i += 1; return Ok(Cow::Borrowed(&self.s[st..self.i - 1])); }
        let mut out = String::from(&self.s[st..self.i]);
        while self.i < b.len() {
            match b[self.i] {
                b'}' => { self.i += 1; return Ok(Cow::Owned(out)); }
                b'\\' => {
                    self.i += 1;
                    let c = self.s[self.i..].chars().next().ok_or("eof")?;
                    out.push(c);
                    self.i += c.len_utf8();
                }
                _ => {
                    let st = self.i;
                    while self.i < b.len() && b[self.i] != b'}' && b[self.i] != b'\\' { self.i += 1; }
                    out.push_str(&self.s[st..self.i]);
                }
            }
        }
        Err("eof in {}".into())
    }
    fn eol(&mut self) { let b = self.s.as_bytes(); while self.i < b.len() && b[self.i] != b'\n' { self.i += 1; } }
}

impl Format for Xschem {
    fn name(&self) -> String { "xschem".into() }
    fn early_exit(&self) -> bool { false }

    fn write(&self, d: &Design) -> String {
        let mut o = String::with_capacity(d.len() * 70 + 4096);
        o.push_str("v {xschem version=3.4.7 file_version=1.2}\n");
        let mut k = String::from("type=subcircuit\nformat=");
        av(&mut k, &d.sym.spice);
        k.push_str("\ntemplate=");
        av(&mut k, &attrs(d.sym.params.iter().map(|(a, b)| (a.as_str(), b.as_str()))));
        o.push_str("K ");
        be(&mut o, &k);
        o.push_str("\nG {}\nV {}\nS {}\nE {}\n");
        for g in &d.sym.gfx { writeln!(o, "L {} {} {} {} {} {{}}", g.layer, g.x1, g.y1, g.x2, g.y2).unwrap(); }
        for p in &d.sym.pins {
            write!(o, "B 5 {} {} {} {} ", p.x - 2, p.y - 2, p.x + 2, p.y + 2).unwrap();
            be(&mut o, &attrs([("name", p.name.as_str()), ("dir", p.dir.as_str())].into_iter()));
            o.push('\n');
        }
        for t in by_id(&d.texts, |x| x.seq) { o.push_str("T "); be(&mut o, &t.s); writeln!(o, " {} {} 0 0 0.4 0.4 {{}}", t.x, t.y).unwrap(); }
        for w in by_id(&d.wires, |x| x.seq) {
            write!(o, "N {} {} {} {} ", w.x1, w.y1, w.x2, w.y2).unwrap();
            be(&mut o, &attrs([("lab", w.net.as_str())].into_iter()));
            o.push('\n');
        }
        for x in by_id(&d.insts, |x| x.seq) {
            o.push_str("C ");
            be(&mut o, &format!("{}.sym", x.sym));
            write!(o, " {} {} {} {} ", x.x, x.y, x.rot, x.flip as u8).unwrap();
            be(&mut o, &attrs(x.p.iter().map(|(a, b)| (a.as_str(), b.as_str()))));
            o.push('\n');
        }
        for l in by_id(&d.labels, |x| x.seq) {
            write!(o, "C {{lab_pin.sym}} {} {} 0 0 ", l.x, l.y).unwrap();
            let name = format!("l{}", l.seq);
            be(&mut o, &attrs([("name", name.as_str()), ("lab", l.net.as_str())].into_iter()));
            o.push('\n');
        }
        o
    }

    fn parse(&self, s: &str) -> Result<Design, String> {
        let mut p = Xs { s, i: 0 };
        let mut d = Design::default();
        let mut seq = 0u32;
        loop {
            p.ws();
            let Some(&c) = s.as_bytes().get(p.i) else { break };
            p.i += 1;
            match c {
                b'v' | b'G' | b'V' | b'S' | b'E' => { p.brace()?; }
                b'K' => {
                    for (k, v) in attr_parse(&p.brace()?) {
                        match k.as_str() { "format" => d.sym.spice = v, "template" => d.sym.params = attr_parse(&v), _ => {} }
                    }
                }
                b'L' => {
                    let layer = p.int()? as u8;
                    let (x1, y1, x2, y2) = (p.int()?, p.int()?, p.int()?, p.int()?);
                    p.brace()?;
                    d.sym.gfx.push(Seg { layer, x1, y1, x2, y2 });
                }
                b'B' => {
                    let layer = p.int()?;
                    let (x1, y1, x2, y2) = (p.int()?, p.int()?, p.int()?, p.int()?);
                    let a = attr_parse(&p.brace()?);
                    if layer == 5 { d.sym.pins.push(Pin { name: aget(&a, "name"), dir: aget(&a, "dir"), x: (x1 + x2) / 2, y: (y1 + y2) / 2 }); }
                }
                b'T' => {
                    let t = p.brace()?.into_owned();
                    let (x, y) = (p.int()?, p.int()?);
                    for _ in 0..4 { p.word()?; }
                    p.brace()?;
                    seq += 1;
                    d.texts.push(Text { id: 0, seq, x, y, s: t });
                }
                b'N' => {
                    let (x1, y1, x2, y2) = (p.int()?, p.int()?, p.int()?, p.int()?);
                    let a = attr_parse(&p.brace()?);
                    seq += 1;
                    d.wires.push(Wire { id: 0, seq, x1, y1, x2, y2, net: aget(&a, "lab") });
                }
                b'C' => {
                    let sr = p.brace()?.into_owned();
                    let (x, y, rot, flip) = (p.int()?, p.int()?, p.int()? as u8, p.int()? != 0);
                    let a = attr_parse(&p.brace()?);
                    seq += 1;
                    if sr == "lab_pin.sym" {
                        let lseq = aget(&a, "name").get(1..).and_then(|n| n.parse().ok()).unwrap_or(seq);
                        d.labels.push(Label { id: 0, seq: lseq, x, y, net: aget(&a, "lab") });
                    } else {
                        let sym = sr.strip_suffix(".sym").unwrap_or(&sr).to_string();
                        d.insts.push(Inst { id: 0, seq, sym, x, y, rot, flip, p: a });
                    }
                }
                b'#' => {}
                _ => return Err(format!("bad tag {:?} @{}", c as char, p.i)),
            }
            p.eol();
        }
        Ok(d)
    }
}

// ---------------------------------------------------------------- zero-copy loader (format's lexing floor)

/// What a production loader over an mmap/read buffer would build: values borrowed
/// from the buffer, param keys interned, no per-field allocation.
#[derive(Default)]
pub struct Lean<'a> {
    pub keys: Vec<&'a str>,
    pub insts: Vec<(u32, Cow<'a, str>, i32, i32, u8, bool, u32, u32)>, // id sym x y rot flip p_start p_len
    pub params: Vec<(u16, Cow<'a, str>)>,
    pub wires: Vec<(u32, i32, i32, i32, i32)>,
    pub labels: Vec<(u32, i32, i32, Cow<'a, str>)>,
    pub texts: Vec<(u32, i32, i32, Cow<'a, str>)>,
}

pub fn parse_lean(s: &str) -> Result<Lean<'_>, String> {
    let mut d = Lean::default();
    for l in s.lines() {
        let (kw, mut r) = split_kw(l);
        let r = &mut r;
        match kw {
            "inst" => {
                let id = unb32(&pos(r)?)?;
                let sym = pos(r)?;
                let (x, y) = (int(&pos(r)?)?, int(&pos(r)?)?);
                let (rot, flip) = unori(&pos(r)?)?;
                let st = d.params.len() as u32;
                while let Some((k, v)) = field(r)? {
                    let k = k.ok_or("positional after params")?;
                    let ki = match d.keys.iter().position(|x| *x == k) { Some(i) => i, None => { d.keys.push(k); d.keys.len() - 1 } };
                    d.params.push((ki as u16, v));
                }
                d.insts.push((id, sym, x, y, rot, flip, st, d.params.len() as u32 - st));
            }
            "wire" => d.wires.push((unb32(&pos(r)?)?, int(&pos(r)?)?, int(&pos(r)?)?, int(&pos(r)?)?, int(&pos(r)?)?)),
            "label" => d.labels.push((unb32(&pos(r)?)?, int(&pos(r)?)?, int(&pos(r)?)?, pos(r)?)),
            "text" => d.texts.push((unb32(&pos(r)?)?, int(&pos(r)?)?, int(&pos(r)?)?, pos(r)?)),
            _ => {} // symbol section: tiny, parsed by the normal path
        }
    }
    Ok(d)
}
