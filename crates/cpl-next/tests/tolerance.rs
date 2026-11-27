//! Contract 3, "total": an error in one unit stops nothing else. Each file in `tolerance/` has
//! errors injected into the units whose first line says `// injected`; every other unit must
//! type, lower and say nothing, and every diagnostic must be in an injected unit.
use cpl_next::facade::{Session, Snapshot};
use std::path::Path;
use std::sync::Arc;

#[test]
fn every_unit_an_error_does_not_reach_types_and_lowers() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tolerance");
    let mut files: Vec<_> = (std::fs::read_dir(&dir).unwrap().flatten())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    let std = Snapshot::new(Arc::new(cpl_next::Std::read()), cpl_next::Roots::std(), "");
    let mut bad = Vec::new();
    for path in &files {
        let src = std::fs::read_to_string(path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy();
        let session = Session::open(std.with_program(&src));
        let (units, diags) = session.units();
        let first_line = |at: u32| {
            let from = src[..at as usize].rfind('\n').map_or(0, |i| i + 1);
            src[from..].lines().next().unwrap_or("")
        };
        let injected = |u: &cpl_next::Unit| first_line(u.span.start).contains("// injected");
        let marked = src.lines().filter(|l| l.contains("// injected")).count();
        let found = units.iter().filter(|u| injected(u)).count();
        if found != marked {
            bad.push(format!(
                "{name}: {marked} injected lines, {found} units start on one"
            ));
        }
        if diags.is_empty() {
            bad.push(format!("{name}: no diagnostics"));
        }
        for u in units {
            let line = src[..u.span.start as usize].matches('\n').count() + 1;
            if injected(u) {
                if u.diags.is_empty() {
                    bad.push(format!("{name}:{line}: an injected unit says nothing"));
                }
            } else if !u.diags.is_empty() || u.untyped > 0 || !u.lowered {
                bad.push(format!(
                    "{name}:{line}: {} untyped, lowered {}: {:?}",
                    u.untyped,
                    u.lowered,
                    u.diags.iter().map(|d| d.to_string()).collect::<Vec<_>>()
                ));
            }
        }
        for d in diags {
            let inside = (d.at.as_ref()).is_some_and(|a| {
                (units.iter()).any(|u| {
                    injected(u)
                        && a.file == "program"
                        && u.span.start <= a.span.start
                        && a.span.end <= u.span.end
                })
            });
            if !inside {
                bad.push(format!("{name}: outside an injected unit: {d}"));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
