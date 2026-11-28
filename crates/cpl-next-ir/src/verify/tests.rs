//! Each stage's verifier passes a fn that keeps its contract and fails one IR built to break it.
use super::*;
use crate::ids::ItemId;

fn e<T>(kind: EK<T>, ty: T) -> Expr<T> {
    Expr { kind, ty, site: 0 }
}

fn func<T>(locals: Vec<T>, body: Vec<Stmt<T>>, ret: T) -> Func<T> {
    Func {
        key: FuncKey::Main,
        name: "t".into(),
        params: vec![],
        n_captures: 0,
        maker: None,
        locals,
        names: vec![],
        vars: vec![],
        views: vec![],
        drops: vec![],
        body,
        ret,
        region: None,
        line: 0,
        sites: vec![
            ItemId::file(crate::ids::FileId::new("m"), "m".into()).at(cpl_diag::Span::new(0, 0))
        ],
        origins: vec![0],
        shown: "t".into(),
        guards: Vec::new(),
        declared: Vec::new(),
        fast: None,
    }
}

/// `let x = 1; return x`.
fn kept<T: Clone>(int: T) -> Func<T> {
    let body = vec![
        Stmt::Let(0, e(EK::Int(1), int.clone())),
        Stmt::Return(Some(e(EK::Copy(Place::Local(0)), int.clone()))),
    ];
    func(vec![int.clone()], body, int)
}

#[test]
fn a_fn_keeping_its_contract_passes_every_stage() {
    assert_eq!(
        check(&kept(Ty::Int), Stage::Generic, &[], 0),
        Vec::<String>::new()
    );
    for s in [
        Stage::Instances,
        Stage::Raw,
        Stage::Canonical,
        Stage::Optimized,
    ] {
        assert_eq!(
            check(&kept(CTy::Int), s, &[Callee::default()], 0),
            Vec::<String>::new(),
            "{s:?}"
        );
    }
}

#[test]
fn every_stage_refuses_a_local_read_before_it_is_set() {
    let mut f = kept(CTy::Int);
    f.body.remove(0);
    for s in [
        Stage::Instances,
        Stage::Raw,
        Stage::Canonical,
        Stage::Optimized,
    ] {
        assert!(!check(&f, s, &[Callee::default()], 0).is_empty(), "{s:?}");
    }
    let mut g = kept(Ty::Int);
    g.body.remove(0);
    assert!(!check(&g, Stage::Generic, &[], 0).is_empty());
}

/// An untyped part's trap is a check that always fails: what follows it never runs, so a
/// read there of a local it would have set is no violation.
#[test]
fn nothing_after_an_untyped_trap_is_read_before_it_is_set() {
    let mut g = kept(Ty::Int);
    g.body[0] = Stmt::Check {
        kind: CheckKind::Untyped,
        cond: e(EK::Bool(false), Ty::Bool),
        msg: "untyped".into(),
        site: 0,
    };
    assert_eq!(check(&g, Stage::Generic, &[], 0), Vec::<String>::new());
}

#[test]
fn an_instance_holding_an_error_type_is_named() {
    let bad = kept(CTy::Lent(Box::new(CTy::Error)));
    let named = error_typed(
        &[kept(CTy::Int), bad],
        &[Callee::default(), Callee::default()],
        0,
    );
    assert_eq!(named, vec!["t: local 0 Lent(Error)".to_string()]);
}

#[test]
fn generic_ir_refuses_an_unsolved_type_and_a_chosen_lend() {
    let mut f = kept(Ty::Int);
    f.locals[0] = Ty::Var(0);
    assert!(!check(&f, Stage::Generic, &[], 0).is_empty());
    let mut f = kept(Ty::Int);
    f.body
        .push(Stmt::Expr(e(EK::Lent(Place::Local(0)), Ty::Int)));
    assert!(!check(&f, Stage::Generic, &[], 0).is_empty());
}

#[test]
fn instances_refuse_a_call_that_names_no_instance() {
    let mut f = kept(CTy::Int);
    f.body.insert(
        0,
        Stmt::Expr(e(EK::Call(Target::Case, vec![], 0), CTy::Unit)),
    );
    assert!(!check(&f, Stage::Instances, &[Callee::default()], 0).is_empty());
    let mut f = kept(CTy::Int);
    f.body.insert(
        0,
        Stmt::Expr(e(EK::Call(Target::Inst(3), vec![], 0), CTy::Unit)),
    );
    assert!(!check(&f, Stage::Instances, &[Callee::default()], 0).is_empty());
}

#[test]
fn raw_ir_refuses_a_passed_place_left_unsettled() {
    let mut f = kept(CTy::Int);
    f.body.insert(
        1,
        Stmt::Expr(e(EK::Pass(Place::Local(0), true, 0), CTy::Int)),
    );
    assert_eq!(
        check(&f, Stage::Instances, &[Callee::default()], 0),
        Vec::<String>::new()
    );
    assert!(!check(&f, Stage::Raw, &[Callee::default()], 0).is_empty());
}

#[test]
fn a_call_inside_an_expression_is_refused_but_an_operator() {
    let call = e(EK::Call(Target::Inst(0), vec![], 0), CTy::Int);
    let mut f = kept(CTy::Int);
    f.body
        .insert(0, Stmt::Let(0, e(EK::MakeStruct(vec![call]), CTy::Int)));
    for s in [Stage::Instances, Stage::Raw, Stage::Canonical] {
        assert!(!check(&f, s, &[Callee::default()], 0).is_empty(), "{s:?}");
    }
    assert_eq!(
        check(
            &f,
            Stage::Canonical,
            &[Callee {
                fast: true,
                ..Callee::default()
            }],
            0
        ),
        Vec::<String>::new(),
        "an operator"
    );
}

#[test]
fn optimized_ir_refuses_a_check_left_out_with_no_proof() {
    let mut canon = kept(CTy::Int);
    canon.body.insert(
        1,
        Stmt::Check {
            kind: CheckKind::Assert,
            cond: e(EK::Bool(true), CTy::Bool),
            msg: String::new(),
            site: 0,
        },
    );
    let opt = kept(CTy::Int);
    assert!(!accounted(0, &canon, &opt, &[]).is_empty());
    let proof = Proved {
        func: 0,
        kind: CheckKind::Assert,
        site: 0,
    };
    assert_eq!(accounted(0, &canon, &opt, &[proof]), Vec::<String>::new());
}

/// After ownership a lent or viewed param is the caller's: its frame never releases it.
#[test]
fn raw_ir_refuses_a_frame_releasing_a_param_it_views() {
    let mut f = kept(CTy::Int);
    f.params = vec![(0, false)];
    f.views = vec![true];
    f.drops = vec![0];
    assert!(!check(&f, Stage::Raw, &[Callee::default()], 0).is_empty());
    f.views = vec![false];
    assert_eq!(
        check(&f, Stage::Raw, &[Callee::default()], 0),
        Vec::<String>::new()
    );
    f.params = vec![(0, true)];
    assert!(!check(&f, Stage::Raw, &[Callee::default()], 0).is_empty());
}

/// After ownership a view goes only to a param the callee views; one it owns takes a copy.
#[test]
fn raw_ir_refuses_a_view_given_to_an_owned_param() {
    let call = |arg| {
        let body = vec![
            Stmt::Let(0, e(EK::Int(1), CTy::Int)),
            Stmt::Expr(e(EK::Call(Target::Inst(0), vec![arg], 0), CTy::Unit)),
            Stmt::Return(Some(e(EK::Copy(Place::Local(0)), CTy::Int))),
        ];
        func(vec![CTy::Int], body, CTy::Int)
    };
    let owns = Callee {
        views: vec![false],
        ..Callee::default()
    };
    let views = Callee {
        views: vec![true],
        ..Callee::default()
    };
    let viewed = call(Arg::View(Place::Local(0)));
    assert!(!check(&viewed, Stage::Raw, std::slice::from_ref(&owns), 0).is_empty());
    assert_eq!(
        check(&viewed, Stage::Raw, &[views], 0),
        Vec::<String>::new()
    );
    let copied = call(Arg::Value(e(EK::Copy(Place::Local(0)), CTy::Int)));
    assert_eq!(check(&copied, Stage::Raw, &[owns], 0), Vec::<String>::new());
}

/// A call gives each param a value of the param's type: an instance picked for other marks is
/// refused before a backend runs it.
#[test]
fn instances_refuse_an_argument_not_of_its_params_type() {
    let mut callee = kept(CTy::Int);
    callee.params = vec![(0, false)];
    callee.views = vec![false];
    let call = |t: CTy| {
        let arg = Arg::Value(e(EK::Int(1), t));
        let body = vec![Stmt::Expr(e(
            EK::Call(Target::Inst(0), vec![arg], 0),
            CTy::Int,
        ))];
        func(vec![], body, CTy::Unit)
    };
    let ok = instances(&[callee.clone(), call(CTy::Int)], Stage::Raw, 0);
    assert!(
        ok.is_empty(),
        "{:?}",
        ok.iter().map(|m| &m.text).collect::<Vec<_>>()
    );
    let lent = CTy::Lent(Box::new(CTy::Int));
    assert_eq!(instances(&[callee, call(lent)], Stage::Raw, 0).len(), 1);
}

#[test]
fn instances_refuse_a_call_or_return_not_of_the_result_type() {
    let lent = CTy::Lent(Box::new(CTy::Int));
    let call = |t: CTy| {
        let body = vec![Stmt::Expr(e(EK::Call(Target::Inst(0), vec![], 0), t))];
        func(vec![], body, CTy::Unit)
    };
    let ok = instances(&[kept(CTy::Int), call(CTy::Int)], Stage::Raw, 0);
    assert!(
        ok.is_empty(),
        "{:?}",
        ok.iter().map(|m| &m.text).collect::<Vec<_>>()
    );
    assert_eq!(
        instances(&[kept(CTy::Int), call(lent.clone())], Stage::Raw, 0).len(),
        1
    );
    let mut wrong = kept(CTy::Int);
    wrong.ret = lent;
    assert_eq!(instances(&[wrong], Stage::Raw, 0).len(), 1);
}

/// `kept` with a value parameter's read first.
fn read<T: Clone>(t: T) -> Func<T> {
    let mut f = kept(t);
    f.body.insert(
        0,
        Stmt::Expr(e(EK::TypeValue(f.ret.clone()), f.ret.clone())),
    );
    f
}

/// A value parameter's read is generic IR's until link gives it its constant; an instance
/// keeping it, or a constant argument check left as written, is refused.
#[test]
fn a_value_parameters_read_is_gone_after_link() {
    assert!(check(&read(Ty::Param(0)), Stage::Generic, &[], 0).is_empty());
    assert!(!check(&read(CTy::Int), Stage::Instances, &[], 0).is_empty());
    let at = ItemId::file(crate::ids::FileId::new("m"), "m".into()).at(cpl_diag::Span::new(0, 0));
    let mut f = kept(Ty::Int);
    f.locals[0] = Ty::Struct(
        crate::ty::StructId(0),
        vec![Ty::Const(ConstArg::At(at))],
        vec![],
    );
    assert!(!check(&f, Stage::Generic, &[], 0).is_empty());
}
