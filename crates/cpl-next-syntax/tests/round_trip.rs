//! Every byte of every CP Lang file in the repository survives the tree:
//! the conformance suite, the std library, the cpl-next test programs,
//! the corpus and the lint specs.

use cpl_next_syntax::ast::{self, AstNode};
use cpl_next_syntax::{lex, parse, NodeKind};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn cp_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            cp_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

fn repo_files(dirs: &[&str]) -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = Vec::new();
    for dir in dirs {
        let before = paths.len();
        cp_files(&root.join(dir), &mut paths);
        assert!(paths.len() > before, "no .cp files under {dir}");
    }
    paths.sort();
    paths
        .into_iter()
        .filter_map(|p| Some((p.clone(), std::fs::read_to_string(&p).ok()?)))
        .collect()
}

fn round_trips(dirs: &[&str]) {
    let mut broken = Vec::new();
    for (path, src) in repo_files(dirs) {
        let lexed = lex(&src);
        let tokens: String = lexed
            .lossless(&src)
            .iter()
            .map(|t| t.span.text(&src))
            .collect();
        let file = parse(&src);
        let tree = file.root.text();
        let items_are_whole = file.items().all(|i| i.kind() == NodeKind::Item);
        if tokens != src || tree != src || !items_are_whole {
            broken.push(path.display().to_string());
        }
    }
    assert!(
        broken.is_empty(),
        "{} files lose bytes: {broken:#?}",
        broken.len()
    );
}

#[test]
fn every_conformance_file_round_trips() {
    round_trips(&["spec/conformance"]);
}

#[test]
fn every_std_file_round_trips() {
    round_trips(&["std"]);
}

#[test]
fn every_test_program_and_corpus_file_round_trips() {
    round_trips(&[
        "crates/cpl-next/tests",
        "corpus",
        "spec/lints",
        "spec/checker-lints",
    ]);
}

/// Every child the typed views read in std and conformance is a node some
/// view or category knows, so no parser output falls between the views.
#[test]
fn every_child_a_view_reads_has_a_view() {
    use NodeKind as K;
    let holders = [
        K::Param,
        K::RetType,
        K::FieldDecl,
        K::GenericParam,
        K::WherePred,
        K::Bound,
        K::Dims,
        K::GenericArgs,
        K::ArrayType,
        K::OptionalType,
        K::FnType,
        K::TupleType,
        K::TypeAlias,
        K::AssocType,
        K::ConstDecl,
        K::Block,
    ];
    let viewed = [
        K::Name,
        K::Error,
        K::Missing,
        K::Decorator,
        K::LendSrc,
        K::RetType,
        K::Bound,
        K::GenericParams,
        K::WhereClause,
        K::ElseClause,
        K::Item,
        K::Dims,
        K::GenericArgs,
        K::ArgList,
    ];
    let mut strays = BTreeMap::new();
    for (path, src) in repo_files(&["std", "spec/conformance"]) {
        for n in parse(&src)
            .root
            .descendants()
            .filter(|n| holders.contains(&n.kind()))
        {
            for c in n.children() {
                let known = viewed.contains(&c.kind())
                    || ast::Type::cast(c.clone()).is_some()
                    || ast::Expr::cast(c.clone()).is_some()
                    || ast::Pattern::cast(c.clone()).is_some()
                    || ast::GenericArg::cast(c.clone()).is_some()
                    || ast::Stmt::cast(c.clone()).is_some()
                    || ast::Decl::cast(c.clone()).is_some();
                if !known {
                    let at = path.display().to_string();
                    strays
                        .entry(format!("{:?} in {:?}", c.kind(), n.kind()))
                        .or_insert(at);
                }
            }
        }
    }
    assert!(strays.is_empty(), "{strays:#?}");
}
