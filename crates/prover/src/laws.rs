//! Algebraic laws: the equations a `Monoid`, a `Group`, an `Action` or
//! an `Order` promises, proved for every value or refuted with one.
//!
//! The laws are the ones written in `stdlib/algebra.cp`, in the same
//! words. A structure is checked by binding its type parameters and its
//! fields:
//!
//! ```
//! use cpl_prover::laws::Structure;
//! let max = Structure::monoid().check(&[("T", "Int")], &[("id", "Int.MIN"), ("op", "(a, b) => max(a, b)")]);
//! assert!(max.unwrap().holds());
//!
//! let sub = Structure::monoid().check(&[("T", "Int")], &[("id", "0"), ("op", "(a, b) => a - b")]).unwrap();
//! assert_eq!(
//!     sub.to_string(),
//!     "Monoid breaks law assoc\n  a = 0, b = 0, c = 1\n  op(op(a, b), c) = -1   op(a, op(b, c)) = 1\n\
//!      Monoid breaks law ident\n  a = 1\n  op(id, a) = -1   a = 1"
//! );
//! ```

use std::collections::HashMap;
use std::fmt;

use crate::context::{Context, Verdict};
use crate::parse;
use crate::term::{BinOp, Error, Sort, Term, Value};

/// `law name(params) = body`: a Bool that must hold for all parameters.
#[derive(Clone, PartialEq, Debug)]
pub struct Law {
    pub name: String,
    /// The parameters and their types; a missing type means `Int`.
    pub params: Vec<(String, Option<Sort>)>,
    pub body: Term,
}

impl Law {
    /// `law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))`.
    pub fn parse(src: &str) -> Result<Law, Error> {
        parse::law(src)
    }
}

impl fmt::Display for Law {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "law {}(", self.name)?;
        for (i, (name, sort)) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            match sort {
                Some(sort) => write!(f, "{name}: {sort}")?,
                None => write!(f, "{name}")?,
            }
        }
        write!(f, ") = {}", self.body)
    }
}

/// A struct with laws, such as `Monoid<T>(id, op)`.
#[derive(Clone, PartialEq, Debug)]
pub struct Structure {
    pub name: String,
    pub laws: Vec<Law>,
}

const ASSOC: &str = "law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))";
const IDENT: &str = "law ident(a: T) = op(id, a) == a and op(a, id) == a";
const INVERSE: &str = "law inverse(a: T) = op(a, inv(a)) == id and op(inv(a), a) == id";

impl Structure {
    pub fn new(name: &str, laws: &[&str]) -> Result<Structure, Error> {
        let laws = laws
            .iter()
            .map(|law| Law::parse(law))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Structure {
            name: name.to_string(),
            laws,
        })
    }

    fn builtin(name: &str, laws: &[&str]) -> Structure {
        Structure::new(name, laws).expect("the laws of the standard library parse")
    }

    /// `Semigroup<T>(op)`: `assoc`.
    pub fn semigroup() -> Structure {
        Structure::builtin("Semigroup", &[ASSOC])
    }

    /// `Monoid<T>(id, op)`: `assoc`, `ident`.
    pub fn monoid() -> Structure {
        Structure::builtin("Monoid", &[ASSOC, IDENT])
    }

    /// `Group<T>(id, op, inv)`: `assoc`, `ident`, `inverse`.
    pub fn group() -> Structure {
        Structure::builtin("Group", &[ASSOC, IDENT, INVERSE])
    }

    /// `Action<T, F>(id, compose, apply)`: `ident`, `composes`.
    pub fn action() -> Structure {
        Structure::builtin(
            "Action",
            &[
                "law ident(a: T) = apply(id, a) == a",
                "law composes(f: F, g: F, a: T) = apply(compose(f, g), a) == apply(f, apply(g, a))",
            ],
        )
    }

    /// `Order<T>(lt)`: `irreflexive`, `transitive`.
    pub fn order() -> Structure {
        Structure::builtin(
            "Order",
            &[
                "law irreflexive(a: T) = not lt(a, a)",
                "law transitive(a: T, b: T, c: T) = not (lt(a, b) and lt(b, c)) or lt(a, c)",
            ],
        )
    }

    /// Add a law.
    pub fn with_law(mut self, law: &str) -> Result<Structure, Error> {
        self.laws.push(Law::parse(law)?);
        Ok(self)
    }

    /// Also require `op(a, b) == op(b, a)`.
    pub fn commutative(self) -> Structure {
        self.with_law("law commutes(a: T, b: T) = op(a, b) == op(b, a)")
            .unwrap()
    }

    /// Also require `op(a, a) == a`, as a sparse table does.
    pub fn idempotent(self) -> Structure {
        self.with_law("law idempotent(a: T) = op(a, a) == a")
            .unwrap()
    }

    /// Also require that `apply` distributes over `op`, as a lazy segment
    /// tree does: updating a merged node equals merging updated children.
    pub fn distributive(self) -> Structure {
        self.with_law("law distributes(f: F, a: T, b: T) = apply(f, op(a, b)) == op(apply(f, a), apply(f, b))")
            .unwrap()
    }

    /// Check every law for the given type parameters and fields.
    pub fn check(&self, types: &[(&str, &str)], fields: &[(&str, &str)]) -> Result<Report, Error> {
        self.check_in(&Context::new(), types, fields)
    }

    /// Like [`Structure::check`], inside a context: the fields may use
    /// its variables (a runtime modulus `m`, say) and rely on its facts.
    pub fn check_in(
        &self,
        ctx: &Context,
        types: &[(&str, &str)],
        fields: &[(&str, &str)],
    ) -> Result<Report, Error> {
        let mut bound: HashMap<String, Sort> = HashMap::new();
        for (name, sort) in types {
            bound.insert(name.to_string(), parse::sort(sort)?);
        }
        let mut base = ctx.clone();
        for (name, term) in fields {
            base.define(name, term)?;
        }
        let mut results = Vec::with_capacity(self.laws.len());
        for law in &self.laws {
            let mut ctx = base.clone();
            for (param, sort) in &law.params {
                let sort = sort
                    .as_ref()
                    .map_or(Sort::Int, |sort| instantiate(sort, &bound));
                ctx.declare(param, &sort)?;
            }
            let verdict = ctx.prove_term(&law.body)?;
            let broken = broken_equation(&ctx, &law.body, &verdict);
            results.push(LawResult {
                law: law.clone(),
                verdict,
                broken,
            });
        }
        Ok(Report {
            structure: self.name.clone(),
            results,
        })
    }
}

/// Checks the `@law`s a `.cp` file states at its top level.
///
/// A one-line fn (`fn add(a: Int, b: Int) -> Int = a + b`) and a `const`
/// are what a law can be proved from; a law about anything else comes
/// back as not proved, and `cpl test` is what tries it on sample values.
///
/// ```
/// let report = cpl_prover::laws::check_source(
///     "fn add(a: Int, b: Int) -> Int = a + b\n\
///      @law comm(a: Int, b: Int) = add(a, b) == add(b, a)\n\
///      @law no_ident(a: Int) = add(a, 1) == a\n",
/// );
/// assert!(report.verdict("comm").is_proved());
/// assert_eq!(report.verdict("no_ident").to_string(), "fails for a = 0");
/// ```
pub fn check_source(src: &str) -> Report {
    let mut base = Context::new();
    let mut laws = Vec::new();
    for line in src.lines() {
        // Only what starts a line is at the top level.
        let line = match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        };
        let line = line.strip_prefix("pub ").unwrap_or(line).trim_end();
        if let Some(law) = line.strip_prefix("@law ") {
            laws.push((law.to_string(), Law::parse(&format!("law {law}"))));
        } else if let Some(def) = line.strip_prefix("fn ") {
            if let Some((name, lambda)) = one_line_fn(def) {
                // A fn the prover can't read leaves its laws not proved.
                let _ = base.define(name, &lambda);
            }
        } else if let Some(def) = line.strip_prefix("const ") {
            if let Some((head, value)) = def.split_once('=') {
                let name = head.split(':').next().unwrap_or(head).trim();
                let _ = base.define(name, value.trim());
            }
        }
    }
    let mut results = Vec::with_capacity(laws.len());
    for (text, law) in laws {
        let law = match law {
            Ok(law) => law,
            Err(error) => {
                let name = text.split('(').next().unwrap_or(&text).trim().to_string();
                results.push(LawResult {
                    law: Law {
                        name,
                        params: Vec::new(),
                        body: Term::Bool(true),
                    },
                    verdict: Verdict::Unknown(error.to_string()),
                    broken: None,
                });
                continue;
            }
        };
        let mut ctx = base.clone();
        let verdict = law
            .params
            .iter()
            .try_for_each(|(param, sort)| ctx.declare(param, sort.as_ref().unwrap_or(&Sort::Int)))
            .and_then(|()| ctx.prove_term(&law.body))
            .unwrap_or_else(|error| Verdict::Unknown(error.to_string()));
        let broken = broken_equation(&ctx, &law.body, &verdict);
        results.push(LawResult {
            law,
            verdict,
            broken,
        });
    }
    Report {
        structure: String::new(),
        results,
    }
}

/// `add(a: Int, b: Int) -> Int = a + b` as `add` and `(a, b) => a + b`.
fn one_line_fn(def: &str) -> Option<(&str, String)> {
    let (name, rest) = def.split_once('(')?;
    let (params, rest) = rest.split_once(')')?;
    let (_, body) = rest.split_once('=')?;
    // `->` and `==` are not the `=` of a body.
    if rest.trim_start().starts_with("->") && !rest.contains(" = ") {
        return None;
    }
    let body = match rest.split_once(" = ") {
        Some((_, body)) => body,
        None => body,
    };
    let names: Vec<&str> = params
        .split(',')
        .map(|param| param.split(':').next().unwrap_or(param).trim())
        .filter(|param| !param.is_empty())
        .collect();
    Some((name.trim(), format!("({}) => {}", names.join(", "), body.trim())))
}

/// The equation of `body` that the counterexample of `verdict` breaks.
fn broken_equation(ctx: &Context, body: &Term, verdict: &Verdict) -> Option<Broken> {
    let counterexample = verdict.counterexample()?;
    equations(body).into_iter().find_map(|(lhs, rhs)| {
        let (left, right) = (
            ctx.eval_at(counterexample, lhs)?,
            ctx.eval_at(counterexample, rhs)?,
        );
        (left != right).then(|| Broken {
            lhs: lhs.to_string(),
            left,
            rhs: rhs.to_string(),
            right,
        })
    })
}

fn instantiate(sort: &Sort, bound: &HashMap<String, Sort>) -> Sort {
    match sort {
        Sort::Named(name) => bound.get(name).cloned().unwrap_or_else(|| sort.clone()),
        Sort::Tuple(items) => Sort::Tuple(
            items
                .iter()
                .map(|(name, sort)| (name.clone(), instantiate(sort, bound)))
                .collect(),
        ),
        Sort::Opt(inner) => Sort::Opt(Box::new(instantiate(inner, bound))),
        other => other.clone(),
    }
}

/// The equations a law body is a conjunction of.
fn equations(body: &Term) -> Vec<(&Term, &Term)> {
    match body {
        Term::Binary(BinOp::And, lhs, rhs) => {
            let mut out = equations(lhs);
            out.extend(equations(rhs));
            out
        }
        Term::Binary(BinOp::Eq, lhs, rhs) => vec![(lhs, rhs)],
        _ => Vec::new(),
    }
}

/// The equation of a law that fails, with the value of each side.
#[derive(Clone, PartialEq, Debug)]
pub struct Broken {
    pub lhs: String,
    pub left: Value,
    pub rhs: String,
    pub right: Value,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LawResult {
    pub law: Law,
    pub verdict: Verdict,
    /// For a refuted law that is an equation: the two sides at the counterexample.
    pub broken: Option<Broken>,
}

/// The outcome of checking a structure: one result per law.
#[derive(Clone, PartialEq, Debug)]
pub struct Report {
    pub structure: String,
    pub results: Vec<LawResult>,
}

impl Report {
    /// Whether every law was proved.
    pub fn holds(&self) -> bool {
        self.results.iter().all(|result| result.verdict.is_proved())
    }

    pub fn law(&self, name: &str) -> Option<&LawResult> {
        self.results.iter().find(|result| result.law.name == name)
    }

    /// The verdict on one law; panics if the structure has no such law.
    pub fn verdict(&self, name: &str) -> &Verdict {
        &self
            .law(name)
            .unwrap_or_else(|| panic!("{} has no law `{name}`", self.structure))
            .verdict
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, result) in self.results.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            let law = &result.law.name;
            // The laws of a file have no structure to name.
            let structure = match self.structure.as_str() {
                "" => "the file",
                name => name,
            };
            match &result.verdict {
                Verdict::Proved => write!(f, "{structure} keeps law {law}")?,
                Verdict::Unknown(reason) => {
                    write!(f, "{structure}: law {law} is not proved: {reason}")?
                }
                Verdict::Refuted(counterexample) => {
                    write!(f, "{structure} breaks law {law}\n  {counterexample}")?;
                    if let Some(Broken {
                        lhs,
                        left,
                        rhs,
                        right,
                    }) = &result.broken
                    {
                        write!(f, "\n  {lhs} = {left}   {rhs} = {right}")?;
                    }
                }
            }
        }
        Ok(())
    }
}
