//! A std fn that takes a sequence and names its element writes the element
//! in the bound, `S: Seq<T>`: a call then binds `T` from the argument. A
//! bare `S: Seq` beside a `T` that nothing else fixes leaves `T` unbound at
//! every call, so its result and its lambdas' parameters are unknown.

use cpl_syntax::ast::{FnDecl, Stmt, StmtKind, Type, TypeKind};
use cpl_syntax::visit::{walk_stmt, Visitor};
use std::path::PathBuf;

/// Whether `ty` names `g` where an argument fixes it: anywhere but a fn
/// type's parameters, which only the element can give.
fn fixes(ty: &Type, g: &str) -> bool {
    match &ty.kind {
        TypeKind::Named { name, args } => name.text == g || args.iter().any(|a| fixes(a, g)),
        TypeKind::Array { index, elem } => index.iter().any(|i| fixes(i, g)) || fixes(elem, g),
        TypeKind::OwnDomain { elem, .. } | TypeKind::Optional(elem) | TypeKind::Mut(elem) => {
            fixes(elem, g)
        }
        TypeKind::Tuple(parts) => parts.iter().any(|p| fixes(p, g)),
        TypeKind::Record(fields) => fields.iter().any(|(_, t)| fixes(t, g)),
        TypeKind::Fn { ret, .. } => fixes(ret, g),
        _ => false,
    }
}

/// The generics of `f` beside a bare `S: Seq` that no parameter or bound fixes.
fn unbound(f: &FnDecl) -> Vec<String> {
    let bare = f.bounds.iter().any(|b| {
        b.traits
            .iter()
            .any(|t| matches!(&t.kind, TypeKind::Named { name, args } if name.text == "Seq" && args.is_empty()))
    });
    if !bare {
        return Vec::new();
    }
    let bounded: Vec<&str> = f.bounds.iter().map(|b| b.name.text.as_str()).collect();
    (f.generics.iter())
        .map(|g| g.text.as_str())
        .filter(|g| !bounded.contains(g))
        .filter(|g| {
            !f.params
                .iter()
                .filter_map(|p| p.ty.as_ref())
                .any(|t| fixes(t, g))
        })
        .filter(|g| !f.bounds.iter().flat_map(|b| &b.traits).any(|t| fixes(t, g)))
        .map(str::to_string)
        .collect()
}

#[derive(Default)]
struct Bare(Vec<String>);

impl Visitor for Bare {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Fn(f) = &stmt.kind {
            let free = unbound(f);
            if !free.is_empty() {
                self.0.push(format!(
                    "`{}` leaves {} unbound",
                    f.name.text,
                    free.join(", ")
                ));
            }
        }
    }
}

#[test]
fn a_seq_bound_names_the_element_its_fn_uses() {
    let std_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&std_dir)
        .expect("std exists")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    files.sort();
    let mut bad = Vec::new();
    for file in files {
        let src = std::fs::read_to_string(&file).unwrap();
        let mut found = Bare::default();
        for (_, item) in cpl_syntax::parse_file(&src).items {
            for stmt in &item.stmts {
                walk_stmt(stmt, &mut found);
            }
        }
        let module = file.file_stem().unwrap().to_string_lossy().into_owned();
        bad.extend(found.0.into_iter().map(|b| format!("std.{module}: {b}")));
    }
    assert!(
        bad.is_empty(),
        "write `S: Seq<T>` so a call binds the element:\n{}",
        bad.join("\n")
    );
}
