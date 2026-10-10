//! In-memory design model, synthetic generator, and edit operations.
//! Every object carries both a random `id` (30 bits) and an insertion `seq`;
//! each format decides which one (if any) it writes.

use std::collections::HashSet;

pub type Kv = Vec<(String, String)>;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Pin { pub name: String, pub dir: String, pub x: i32, pub y: i32 }
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Seg { pub layer: u8, pub x1: i32, pub y1: i32, pub x2: i32, pub y2: i32 }
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Sym { pub pins: Vec<Pin>, pub params: Kv, pub spice: String, pub gfx: Vec<Seg> }
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Inst { pub id: u32, pub seq: u32, pub sym: String, pub x: i32, pub y: i32, pub rot: u8, pub flip: bool, pub p: Kv }
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Wire { pub id: u32, pub seq: u32, pub x1: i32, pub y1: i32, pub x2: i32, pub y2: i32, pub net: String }
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Label { pub id: u32, pub seq: u32, pub x: i32, pub y: i32, pub net: String }
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Text { pub id: u32, pub seq: u32, pub x: i32, pub y: i32, pub s: String }
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Net { pub id: u32, pub name: String }

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Design {
    pub sym: Sym,
    pub nets: Vec<Net>,
    pub insts: Vec<Inst>,
    pub wires: Vec<Wire>,
    pub labels: Vec<Label>,
    pub texts: Vec<Text>,
}

impl Design {
    pub fn len(&self) -> usize { self.insts.len() + self.wires.len() + self.labels.len() + self.texts.len() }
    fn ids(&self) -> HashSet<u32> {
        let mut s = HashSet::new();
        s.extend(self.insts.iter().map(|x| x.id));
        s.extend(self.wires.iter().map(|x| x.id));
        s.extend(self.labels.iter().map(|x| x.id));
        s.extend(self.texts.iter().map(|x| x.id));
        s.extend(self.nets.iter().map(|x| x.id));
        s
    }
    fn max_seq(&self) -> u32 {
        let a = self.insts.iter().map(|x| x.seq).chain(self.wires.iter().map(|x| x.seq));
        a.chain(self.labels.iter().map(|x| x.seq)).chain(self.texts.iter().map(|x| x.seq)).max().unwrap_or(0)
    }
}

/// splitmix64
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self { Rng(seed ^ 0x9e37_79b9_7f4a_7c15) }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize }
    pub fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str { xs[self.below(xs.len())] }
    fn grid(&mut self, side: i32) -> i32 { (self.below((side / 10) as usize) as i32) * 10 - side / 2 }
}

fn fresh_id(used: &mut HashSet<u32>, r: &mut Rng) -> u32 {
    loop {
        let v = (r.next() & 0x3fff_ffff) as u32;
        if used.insert(v) { return v; }
    }
}

const SYMS: [(&str, &str, [&str; 5]); 5] = [
    ("nmos4", "M", ["w", "l", "nf", "m", "model"]),
    ("pmos4", "M", ["w", "l", "nf", "m", "model"]),
    ("res", "R", ["value", "w", "l", "m", "model"]),
    ("cap", "C", ["value", "w", "l", "m", "model"]),
    ("vsource", "V", ["dc", "ac", "tran", "m", "model"]),
];

fn val(r: &mut Rng, sym: &str, k: &str) -> String {
    match k {
        "w" => r.pick(&["0.42u", "1u", "2u", "5u", "10u"]),
        "l" => r.pick(&["0.15u", "0.18u", "0.5u", "1u"]),
        "nf" => r.pick(&["1", "2", "4"]),
        "m" => r.pick(&["1", "1", "2"]),
        "value" if sym == "res" => r.pick(&["10k", "2.2k", "100"]),
        "value" => r.pick(&["1p", "100f", "2.5p"]),
        "dc" => r.pick(&["0", "1.8"]),
        "ac" => r.pick(&["0", "1"]),
        "tran" => "pulse(0 1.8 0 1n 1n 5n 10n)",
        _ => match sym {
            "nmos4" => "nfet_01v8",
            "pmos4" => "pfet_01v8",
            "res" => "res_xhigh_po",
            "cap" => "cap_mim_m3_1",
            _ => "vdc",
        },
    }
    .to_string()
}

fn make_inst(r: &mut Rng, id: u32, seq: u32, side: i32, n: usize) -> Inst {
    let k = r.below(100);
    let si = if k < 35 { 0 } else if k < 70 { 1 } else if k < 85 { 2 } else if k < 95 { 3 } else { 4 };
    let (sym, pre, keys) = SYMS[si];
    let mut p = vec![("name".to_string(), format!("{pre}{n}"))];
    for key in keys { p.push((key.to_string(), val(r, sym, key))); }
    let rot = r.below(4) as u8;
    Inst { id, seq, sym: sym.into(), x: r.grid(side), y: r.grid(side), rot, flip: r.below(4) == 0, p }
}

fn make_wire(r: &mut Rng, id: u32, seq: u32, side: i32, net: String) -> Wire {
    let (x, y, len) = (r.grid(side), r.grid(side), (r.below(40) as i32 + 1) * 10);
    if r.below(2) == 0 { Wire { id, seq, x1: x, y1: y, x2: x + len, y2: y, net } } else { Wire { id, seq, x1: x, y1: y, x2: x, y2: y + len, net } }
}

pub const RENAME_NET: &str = "VBIAS";

pub fn generate(n: usize, seed: u64) -> Design {
    let mut r = Rng::new(seed);
    let mut used = HashSet::new();
    let side = (((n as f64).sqrt() * 150.0) as i32 / 10).max(50) * 10;
    let (ni, nw, nl) = (n * 35 / 100, n * 45 / 100, n * 12 / 100);
    let nt = n - ni - nw - nl;
    let nnets = (nw / 6).max(4);
    let mut d = Design::default();
    for i in 0..nnets {
        let name = match i { 0 => RENAME_NET.to_string(), 1 => "VDD".into(), 2 => "VSS".into(), _ => format!("net{i}") };
        d.nets.push(Net { id: fresh_id(&mut used, &mut r), name });
    }
    let mut seq = 0u32;
    for i in 0..ni {
        seq += 1;
        let id = fresh_id(&mut used, &mut r);
        d.insts.push(make_inst(&mut r, id, seq, side, i + 1));
    }
    // RENAME_NET is used by exactly 50 objects: 47 wires + 3 labels.
    let stride = (nw / 47).max(1);
    for i in 0..nw {
        seq += 1;
        let net = if i % stride == 0 && i / stride < 47 { 0 } else { 1 + r.below(nnets - 1) };
        let id = fresh_id(&mut used, &mut r);
        d.wires.push(make_wire(&mut r, id, seq, side, d.nets[net].name.clone()));
    }
    for i in 0..nl {
        seq += 1;
        let net = if i < 3 { 0 } else { 1 + r.below(nnets - 1) };
        let id = fresh_id(&mut used, &mut r);
        d.labels.push(Label { id, seq, x: r.grid(side), y: r.grid(side), net: d.nets[net].name.clone() });
    }
    for i in 0..nt {
        seq += 1;
        let s = if i % 25 == 0 { format!("see \"spec\" rev {i}\nsecond line") } else { format!("note {i}: bias for stage {}", i % 7) };
        let id = fresh_id(&mut used, &mut r);
        d.texts.push(Text { id, seq, x: r.grid(side), y: r.grid(side), s });
    }
    d.sym = sym();
    d
}

fn sym() -> Sym {
    let pins = [("VDD", "inout", 0, -400), ("VSS", "inout", 0, 400), ("IN_P", "in", -400, -100), ("IN_N", "in", -400, 100),
        ("OUT", "out", 400, 0), ("BIAS", "in", -400, 200), ("EN", "in", -400, -200), ("CLK", "in", -400, 300)];
    let params = [("w", "1u"), ("l", "0.18u"), ("nf", "1"), ("m", "1"), ("gain", "10"), ("note", "default \"x\"")];
    let mut gfx = Vec::new();
    for i in 0..12 { gfx.push(Seg { layer: 4, x1: -300 + i * 50, y1: -300, x2: 300, y2: -300 + i * 50 }); }
    Sym {
        pins: pins.iter().map(|&(n, d, x, y)| Pin { name: n.into(), dir: d.into(), x, y }).collect(),
        params: params.iter().map(|&(k, v)| (k.into(), v.into())).collect(),
        spice: "X@name @pinlist @symname w=@w l=@l".into(),
        gfx,
    }
}

#[derive(Clone, Debug)]
pub enum Edit {
    MoveInst(u32, i32),
    MoveWire(u32, i32),
    Rename(String, String),
    SetParam(u32, String, String),
    AddInst(Inst),
    AddWire(Wire),
    DelWire(u32),
}

pub fn apply(d: &mut Design, e: &Edit) {
    match e {
        Edit::MoveInst(id, dx) => d.insts.iter_mut().filter(|x| x.id == *id).for_each(|x| x.x += dx),
        Edit::MoveWire(id, dy) => d.wires.iter_mut().filter(|x| x.id == *id).for_each(|w| { w.y1 += dy; w.y2 += dy }),
        Edit::Rename(a, b) => {
            d.wires.iter_mut().filter(|w| w.net == *a).for_each(|w| w.net = b.clone());
            d.labels.iter_mut().filter(|l| l.net == *a).for_each(|l| l.net = b.clone());
            d.nets.iter_mut().filter(|n| n.name == *a).for_each(|n| n.name = b.clone());
        }
        Edit::SetParam(id, k, v) => {
            for x in d.insts.iter_mut().filter(|x| x.id == *id) {
                match x.p.iter_mut().find(|p| p.0 == *k) { Some(p) => p.1 = v.clone(), None => x.p.push((k.clone(), v.clone())) }
            }
        }
        Edit::AddInst(i) => d.insts.push(i.clone()),
        Edit::AddWire(w) => d.wires.push(w.clone()),
        Edit::DelWire(id) => d.wires.retain(|w| w.id != *id),
    }
}

/// A new instance as an editor would create it: fresh random id, next seq.
pub fn new_inst(d: &Design, r: &mut Rng) -> Inst {
    let mut used = d.ids();
    let id = fresh_id(&mut used, r);
    let k = 90000 + r.below(1000);
    make_inst(r, id, d.max_seq() + 1, 2000, k)
}
pub fn new_wire(d: &Design, r: &mut Rng) -> Wire {
    let mut used = d.ids();
    let id = fresh_id(&mut used, r);
    let net = d.wires[r.below(d.wires.len())].net.clone();
    make_wire(r, id, d.max_seq() + 1, 2000, net)
}

// ---------------------------------------------------------------- semantic 3-way merge (git merge driver model)

/// 3-way pick: whichever side changed wins; both changed differently = conflict.
fn pick<'a, T: PartialEq>(o: &'a T, a: &'a T, b: &'a T) -> Option<&'a T> {
    if a == o || a == b { Some(b) } else if b == o { Some(a) } else { None }
}

/// Object-level merge keyed by id; `fields` merges an object both sides changed.
fn merge_objs<T: Clone + PartialEq>(o: &[T], a: &[T], b: &[T], id: fn(&T) -> u32, fields: fn(&T, &T, &T) -> Option<T>) -> Option<Vec<T>> {
    use std::collections::HashMap;
    fn m<T>(xs: &[T], id: fn(&T) -> u32) -> HashMap<u32, &T> { xs.iter().map(|x| (id(x), x)).collect() }
    let (mo, ma, mb) = (m(o, id), m(a, id), m(b, id));
    let mut out = Vec::new();
    for x in o {
        match (ma.get(&id(x)), mb.get(&id(x))) {
            (Some(&xa), Some(&xb)) => out.push(match pick(x, xa, xb) { Some(v) => v.clone(), None => fields(x, xa, xb)? }),
            (None, Some(&xb)) if xb == x => {}
            (Some(&xa), None) if xa == x => {}
            (None, None) => {}
            _ => return None, // modify/delete
        }
    }
    for x in a.iter().filter(|x| !mo.contains_key(&id(x))) {
        if mb.get(&id(x)).is_some_and(|y| *y != x) { return None; } // id collision
        out.push(x.clone());
    }
    out.extend(b.iter().filter(|x| !mo.contains_key(&id(x)) && !ma.contains_key(&id(x))).cloned());
    Some(out)
}

fn merge_inst(o: &Inst, a: &Inst, b: &Inst) -> Option<Inst> {
    let mut r = a.clone();
    r.sym = pick(&o.sym, &a.sym, &b.sym)?.clone();
    (r.x, r.y, r.rot, r.flip) = *pick(&(o.x, o.y, o.rot, o.flip), &(a.x, a.y, a.rot, a.flip), &(b.x, b.y, b.rot, b.flip))?;
    let get = |x: &Inst, k: &str| x.p.iter().find(|p| p.0 == k).map(|p| p.1.clone());
    let keys = a.p.iter().map(|p| &p.0).chain(b.p.iter().map(|p| &p.0).filter(|k| get(a, k).is_none()));
    r.p = Vec::new();
    for k in keys {
        if let Some(v) = pick(&get(o, k), &get(a, k), &get(b, k))? { r.p.push((k.clone(), v.clone())); }
    }
    Some(r)
}

/// What `schemify merge-driver %O %A %B` would do: field-level merge, canonical rewrite.
pub fn merge3(o: &Design, a: &Design, b: &Design) -> Option<Design> {
    if a.sym != b.sym && a.sym != o.sym && b.sym != o.sym { return None; }
    Some(Design {
        sym: pick(&o.sym, &a.sym, &b.sym)?.clone(),
        nets: merge_objs(&o.nets, &a.nets, &b.nets, |x| x.id, |_, _, _| None)?,
        insts: merge_objs(&o.insts, &a.insts, &b.insts, |x| x.id, merge_inst)?,
        wires: merge_objs(&o.wires, &a.wires, &b.wires, |x| x.id, |o, a, b| {
            let g = |w: &Wire| (w.x1, w.y1, w.x2, w.y2);
            let mut r = a.clone();
            (r.x1, r.y1, r.x2, r.y2) = *pick(&g(o), &g(a), &g(b))?;
            r.net = pick(&o.net, &a.net, &b.net)?.clone();
            Some(r)
        })?,
        labels: merge_objs(&o.labels, &a.labels, &b.labels, |x| x.id, |o, a, b| {
            let mut r = a.clone();
            (r.x, r.y) = *pick(&(o.x, o.y), &(a.x, a.y), &(b.x, b.y))?;
            r.net = pick(&o.net, &a.net, &b.net)?.clone();
            Some(r)
        })?,
        texts: merge_objs(&o.texts, &a.texts, &b.texts, |x| x.id, |o, a, b| {
            let mut r = a.clone();
            (r.x, r.y) = *pick(&(o.x, o.y), &(a.x, a.y), &(b.x, b.y))?;
            r.s = pick(&o.s, &a.s, &b.s)?.clone();
            Some(r)
        })?,
    })
}
