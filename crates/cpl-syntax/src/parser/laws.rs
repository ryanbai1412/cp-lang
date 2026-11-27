//! The `laws` member a struct with `law`s derives (R08.3.5).

use super::*;

/// Appends `@get pub fn laws(self) -> List<(name: Str, sizes: S -> List<Int>, holds: (S, List<Int>) -> Bool)>`
/// to a struct body that declares laws: one entry per law, in declaration order.
///
/// `S` is the samples a caller hands over: `List<T>` when every law parameter has the one
/// type `T`, else a record of one list per parameter type, `(T: List<T>, F: List<F>)`.
/// `sizes(s)` is the length of each parameter's pool, and `holds(s, picks)` evaluates the law
/// with parameter `k` bound to element `picks[k]` of its pool.
pub(super) fn derive_laws(methods: &mut Vec<Stmt>) {
    let laws: Vec<_> = methods
        .iter()
        .filter_map(|m| match &m.kind {
            StmtKind::Law { name, params, body } => Some((m.span, name, params, body)),
            _ => None,
        })
        .collect();
    let declares_laws = |m: &Stmt| matches!(&m.kind, StmtKind::Fn(f) if f.name.text == "laws");
    if laws.is_empty() || methods.iter().any(declares_laws) {
        return;
    }
    let mut pools: Vec<String> = Vec::new();
    for (_, _, params, _) in &laws {
        for p in params.iter() {
            let Some(Type {
                kind: TypeKind::Named { name, args },
                ..
            }) = &p.ty
            else {
                return;
            };
            if !args.is_empty() {
                return;
            }
            if !pools.contains(&name.text) {
                pools.push(name.text.clone());
            }
        }
    }
    let span = laws[0].0;
    let b = Build(span);
    let samples_ty = if let [only] = pools.as_slice() {
        b.list_ty(b.ty(only))
    } else {
        b.ty_of(TypeKind::Record(
            pools
                .iter()
                .map(|t| (b.name(t), b.list_ty(b.ty(t))))
                .collect(),
        ))
    };
    let pool = |p: &Param| {
        let s = b.var(SAMPLES);
        match &p.ty {
            Some(Type {
                kind: TypeKind::Named { name, .. },
                ..
            }) if pools.len() > 1 => b.field(s, &name.text),
            _ => s,
        }
    };
    let entries = laws
        .iter()
        .map(|&(span, name, params, body)| {
            let b = Build(span);
            let sizes = b.lambda(
                vec![(SAMPLES, samples_ty.clone())],
                b.expr(ExprKind::List(
                    params.iter().map(|p| b.field(pool(p), "len")).collect(),
                )),
            );
            let mut stmts: Vec<Stmt> = params
                .iter()
                .enumerate()
                .map(|(k, p)| {
                    let pick = b.index(b.var(PICKS), b.expr(ExprKind::Int(k as i128)));
                    b.stmt(StmtKind::Let {
                        mutable: false,
                        pat: Pattern {
                            kind: PatternKind::Name(p.name.text.clone()),
                            span: p.name.span,
                        },
                        ty: None,
                        init: Some(b.index(pool(p), pick)),
                        otherwise: None,
                        guard: None,
                    })
                })
                .collect();
            stmts.push(b.stmt(StmtKind::Expr(body.clone())));
            let holds = b.lambda(
                vec![
                    (SAMPLES, samples_ty.clone()),
                    (PICKS, b.list_ty(b.ty("Int"))),
                ],
                b.expr(ExprKind::Block(Block { stmts, span })),
            );
            b.expr(ExprKind::Record(vec![
                (
                    b.name("name"),
                    b.expr(ExprKind::Str(vec![StrPiece::Text(name.text.clone())])),
                ),
                (b.name("sizes"), sizes),
                (b.name("holds"), holds),
            ]))
        })
        .collect();
    let entry_ty = b.ty_of(TypeKind::Record(vec![
        (b.name("name"), b.ty("Str")),
        (
            b.name("sizes"),
            b.fn_ty(vec![samples_ty.clone()], b.list_ty(b.ty("Int"))),
        ),
        (
            b.name("holds"),
            b.fn_ty(vec![samples_ty, b.list_ty(b.ty("Int"))], b.ty("Bool")),
        ),
    ]));
    let laws_fn = FnDecl {
        name: b.name("laws"),
        generics: Vec::new(),
        bounds: Vec::new(),
        params: vec![Param {
            mutable: false,
            variadic: false,
            lazy: false,
            named_only: false,
            label: None,
            name: b.name("self"),
            ty: None,
            default: None,
            span,
        }],
        ret: Some(b.list_ty(entry_ty)),
        lends: None,
        uses: Vec::new(),
        body: FnBody::Expr(b.expr(ExprKind::List(entries))),
        sig_span: span,
    };
    methods.push(Stmt {
        kind: StmtKind::Fn(laws_fn),
        span,
        decorators: vec![Decorator {
            directive: false,
            mark: Some(crate::decorators::Mark::Get),
            name: b.name("get"),
            args: Vec::new(),
            span,
        }],
        public: true,
    });
}

/// Names no law body can mean: a law parameter or field named the same shadows nothing
/// before its own `let`.
const SAMPLES: &str = "law_samples";
const PICKS: &str = "law_picks";

/// AST built at one span: the derived member points at the law it came from.
#[derive(Clone, Copy)]
struct Build(Span);

impl Build {
    fn name(self, text: &str) -> Name {
        Name {
            text: text.to_string(),
            span: self.0,
        }
    }

    fn expr(self, kind: ExprKind) -> Expr {
        Expr { kind, span: self.0 }
    }

    fn stmt(self, kind: StmtKind) -> Stmt {
        Stmt {
            kind,
            span: self.0,
            decorators: Vec::new(),
            public: false,
        }
    }

    fn var(self, text: &str) -> Expr {
        self.expr(ExprKind::Name(text.to_string()))
    }

    fn field(self, base: Expr, name: &str) -> Expr {
        self.expr(ExprKind::Field {
            base: Box::new(base),
            name: self.name(name),
        })
    }

    fn index(self, base: Expr, at: Expr) -> Expr {
        self.expr(ExprKind::Index {
            base: Box::new(base),
            indices: vec![at],
        })
    }

    fn lambda(self, params: Vec<(&str, Type)>, body: Expr) -> Expr {
        let (pats, tys) = params
            .into_iter()
            .map(|(n, t)| {
                let pat = Pattern {
                    kind: PatternKind::Name(n.to_string()),
                    span: self.0,
                };
                (pat, Some(t))
            })
            .unzip();
        self.expr(ExprKind::Lambda {
            params: pats,
            param_tys: tys,
            param_mut: Vec::new(),
            ret_ty: None,
            body: Box::new(body),
        })
    }

    fn ty_of(self, kind: TypeKind) -> Type {
        Type { kind, span: self.0 }
    }

    fn ty(self, name: &str) -> Type {
        self.ty_of(TypeKind::Named {
            name: self.name(name),
            args: Vec::new(),
        })
    }

    fn list_ty(self, elem: Type) -> Type {
        self.ty_of(TypeKind::Named {
            name: self.name("List"),
            args: vec![elem],
        })
    }

    fn fn_ty(self, params: Vec<Type>, ret: Type) -> Type {
        self.ty_of(TypeKind::Fn {
            params,
            ret: Box::new(ret),
        })
    }
}
