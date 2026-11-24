//! Every `Int` `+`, `-` and `*` in a std body that a std conformance case
//! reaches is proved to stay an `Int` (handbook/domains/stdlib.md, "No
//! overflow inside std").
use crate::ranges::{Facts, Sign};
use cpl_ir::{BinOp, Expr, ExprKind, Function, Ty};
use std::collections::BTreeMap;
use std::sync::Mutex;

struct Op {
    func: String,
    text: String,
    base: bool,
    assumed: bool,
    why: String,
}

fn show(func: &Function, e: &Expr, depth: u8) -> String {
    if depth > 4 {
        return "..".into();
    }
    let s = |e: &Expr| show(func, e, depth + 1);
    match &e.kind {
        ExprKind::Int(n) => n.to_string(),
        ExprKind::Local(slot) => func.locals.get(*slot).map_or(format!("l{slot}"), |l| l.name.clone()),
        ExprKind::Global(g) => format!("g{g}"),
        ExprKind::Binary(op, l, r) => format!("({} {op:?} {})", s(l), s(r)),
        ExprKind::Field(b, i) => format!("{}.{i}", s(b)),
        ExprKind::Index(b, ix) => format!("{}[{}]", s(b), ix.iter().map(s).collect::<Vec<_>>().join(", ")),
        ExprKind::Cast(c, x) => format!("{c:?}({})", s(x)),
        ExprKind::Builtin(b, a) => format!("{b:?}({})", a.iter().map(s).collect::<Vec<_>>().join(", ")),
        ExprKind::Call(f, _) => format!("call#{f}(..)"),
        ExprKind::DomainLen(d) => format!("len(D{d})"),
        k => format!("{k:?}").chars().take(30).collect(),
    }
}

/// The leaf that keeps `e` from a sign (or, with a sign, from a bound).
fn blame(f: &Facts, id: usize, func: &Function, e: &Expr, depth: u8) -> String {
    let unsigned = f.sign(Some(id), e) == Sign::Unknown;
    let unbounded = f.upper_bound(Some(id), e, 1).is_none();
    let bad = |x: &Expr| (unsigned && f.sign(Some(id), x) == Sign::Unknown) || (!unsigned && f.upper_bound(Some(id), x, 1).is_none());
    let tag = if unsigned { "sign" } else if unbounded { "bound" } else { "ok" };
    if depth > 6 {
        return format!("{tag}:deep");
    }
    match &e.kind {
        ExprKind::Local(slot) => {
            let name = func.locals.get(*slot).map_or("?".into(), |l| l.name.clone());
            let v = f.local(Some(id), *slot);
            if v.0 == Some(id) && *slot < func.params.len() {
                format!("{tag}:param {name}")
            } else if f.stores(v) > 1 {
                format!("{tag}:reassigned {name}")
            } else if let Some(init) = f.init(v) {
                match v.0 {
                    Some(h) if h == id => format!("{name}={}", blame(f, id, func, init, depth + 1)),
                    _ => format!("{tag}:captured {name}"),
                }
            } else {
                format!("{tag}:local {name}")
            }
        }
        ExprKind::Binary(BinOp::Sub, _, _) if unsigned => format!("{tag}:difference"),
        ExprKind::Binary(_, l, r) => match [l, r].into_iter().find(|x| bad(x)) {
            Some(x) => blame(f, id, func, x, depth + 1),
            None => format!("{tag}:binary"),
        },
        ExprKind::Int(n) if *n < 0 => format!("{tag}:negative literal"),
        ExprKind::Field(b, _) => format!("{tag}:field of {:?}", b.ty),
        ExprKind::Index(..) => format!("{tag}:element"),
        ExprKind::Call(..) | ExprKind::CallExpr(..) => format!("{tag}:call result"),
        k => format!("{tag}:{}", format!("{k:?}").split(['(', ' ', '{']).next().unwrap_or("")),
    }
}

fn scan(src: &str) -> Option<Vec<((String, u32, u32), Op)>> {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let a = db.analyze("t.cp")?;
    if !a.diagnostics.is_empty() {
        return None;
    }
    let p = cpl_ir::lower(&a).ok()?;
    let (base, assumed) = (Facts::collect(&p), Facts::collect_with(&p, true));
    let mut out = Vec::new();
    for (id, func) in p.fns.iter().enumerate() {
        if !p.library_fns[id] {
            continue;
        }
        cpl_ir::walk::deep_exprs(&func.body, &mut |x| {
            let ExprKind::Binary(op @ (BinOp::Add | BinOp::Sub | BinOp::Mul), l, r) = &x.kind else {
                return;
            };
            if l.ty != Ty::Int {
                return;
            }
            let (b, a) = (base.fits(Some(id), *op, l, r), assumed.fits(Some(id), *op, l, r));
            let why = if a {
                String::new()
            } else {
                let side = [l, r].into_iter().find(|s| assumed.sign(Some(id), s) == Sign::Unknown || assumed.upper_bound(Some(id), s, 1).is_none());
                side.map_or("overflow at the bounds".into(), |s| blame(&assumed, id, func, s, 0))
            };
            let key = (p.fn_display[id].clone(), x.span.start, x.span.end);
            out.push((key, Op { func: p.fn_display[id].clone(), text: show(func, x, 0), base: b, assumed: a, why }));
        });
    }
    Some(out)
}

#[test]
fn every_int_op_in_std_bodies_is_proved() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = vec![];
    for dir in std::fs::read_dir(root.join("spec/conformance")).unwrap() {
        let dir = dir.unwrap().path();
        if dir.file_name().unwrap().to_str().unwrap().starts_with("std-") {
            for f in std::fs::read_dir(&dir).unwrap() {
                paths.push(f.unwrap().path());
            }
        }
    }
    paths.retain(|f| f.extension().is_some_and(|e| e == "cp"));
    paths.sort();
    let queue = Mutex::new(paths);
    let ops: Mutex<BTreeMap<(String, u32, u32), Op>> = Mutex::default();
    let files = Mutex::new(0usize);
    std::thread::scope(|s| {
        for _ in 0..std::thread::available_parallelism().map_or(4, |n| n.get()) {
            std::thread::Builder::new().stack_size(1 << 28).spawn_scoped(s, || loop {
                let Some(f) = queue.lock().unwrap().pop() else { return };
                let src = std::fs::read_to_string(&f).unwrap();
                if !src.starts_with("// expect: ok") {
                    continue;
                }
                let Some(found) = scan(&src) else { continue };
                *files.lock().unwrap() += 1;
                let mut ops = ops.lock().unwrap();
                for (k, op) in found {
                    let Some(e) = ops.get_mut(&k) else {
                        ops.insert(k, op);
                        continue;
                    };
                    if !op.assumed && e.assumed {
                        e.assumed = false;
                        e.why = op.why;
                        e.text = op.text;
                    }
                    e.base &= op.base;
                }
            }).unwrap();
        }
    });
    let ops = ops.into_inner().unwrap();
    let base = ops.values().filter(|o| !o.base).count();
    let assumed: Vec<&Op> = ops.values().filter(|o| !o.assumed).collect();
    let mut tsv = String::new();
    for o in &assumed {
        tsv += &format!("{}\t{}\t{}\n", o.func, o.text, o.why);
    }
    std::fs::write(root.join("target/std-overflow.tsv"), tsv).unwrap();
    eprintln!("files {} ops {} unproven {base} unproven-with-len-facts {}", files.into_inner().unwrap(), ops.len(), assumed.len());
    assert_eq!(base, 0, "std ops not proved to stay an Int: see target/std-overflow.tsv");
}

/// One `Session` reused, `set_file("t.cp", ..)` per std case in sorted
/// order, as the oracle first ran: prints how long each step takes.
#[test]
#[ignore]
fn session_reuse_timing() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = vec![];
    for dir in std::fs::read_dir(root.join("spec/conformance")).unwrap() {
        let dir = dir.unwrap().path();
        if dir.file_name().unwrap().to_str().unwrap().starts_with("std-") {
            for f in std::fs::read_dir(&dir).unwrap() {
                paths.push(f.unwrap().path());
            }
        }
    }
    paths.retain(|f| f.extension().is_some_and(|e| e == "cp"));
    paths.sort();
    let only = std::env::var("ONLY").ok();
    std::thread::Builder::new().stack_size(1 << 28).spawn(move || {
        let mut db = cpl_check::Session::new();
        for f in paths {
            let src = std::fs::read_to_string(&f).unwrap();
            if !src.starts_with("// expect: ok") {
                continue;
            }
            if let Some(o) = &only {
                if !o.split(',').any(|o| f.ends_with(o)) {
                    continue;
                }
            }
            let t = std::time::Instant::now();
            db.set_file("t.cp", src.as_str());
            let a = db.analyze("t.cp");
            let ta = t.elapsed();
            let lowered = a.as_ref().filter(|a| a.diagnostics.is_empty()).and_then(|a| cpl_ir::lower(a).ok());
            let tl = t.elapsed();
            if let Some(p) = &lowered {
                let _ = Facts::collect(p);
            }
            eprintln!("{:>8.2?} {:>8.2?} {:>8.2?} {}", ta, tl - ta, t.elapsed() - tl, f.strip_prefix(&root).unwrap().display());
        }
    }).unwrap().join().unwrap();
}
