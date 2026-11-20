//! `law disjoint`: may two lends of one container be live together?
//!
//! The rules are those of `plan/19-index-and-lends.md`, section 3.6:
//!
//! - two lends that only read never conflict;
//! - lends of different variables, or of different fields of one, are
//!   separate without any law;
//! - two lends through the *same projection* of the same receiver are
//!   allowed exactly when the projection's `law disjoint` is proved for
//!   their arguments from the facts in scope: no check is inserted, so
//!   "probably different" is a conflict;
//! - anything else (two different projections of one receiver, a place
//!   and a part of it) conflicts.

use crate::context::{Context, Counterexample, Verdict};
use crate::laws::Law;
use crate::parse;
use crate::term::{Error, Sort, Term};

/// One lend: `mut a[i]`, `a[l..m]`, `mut g.row(r)`, `mut p.x`.
#[derive(Clone, PartialEq, Debug)]
pub struct Lend {
    /// The place lent from: `a`, `g`, `p.x`.
    pub receiver: String,
    /// The projection applied to it: `at` for `a[i]`, `slice` for
    /// `a[l..m]`, the method name for `g.row(r)`. `None` lends the whole place.
    pub projection: Option<String>,
    pub args: Vec<Term>,
    pub mutable: bool,
}

impl Lend {
    pub fn parse(src: &str) -> Result<Lend, Error> {
        let src = src.trim();
        let (mutable, rest) = match src.strip_prefix("mut ") {
            Some(rest) => (true, rest),
            None => (false, src),
        };
        let (receiver, projection, args) = match parse::term(rest)? {
            Term::Index(base, args) => {
                let is_range = |arg: &Term| {
                    matches!(arg, Term::Tuple(items)
                        if items.len() == 2 && items[0].0.as_deref() == Some("lo") && items[1].0.as_deref() == Some("hi"))
                };
                let projection = if args.iter().any(is_range) {
                    "slice"
                } else {
                    "at"
                };
                (place(&base)?, Some(projection.to_string()), args)
            }
            Term::Call(callee, args) => match *callee {
                Term::Field(base, method) => (place(&base)?, Some(method), args),
                _ => {
                    return Err(Error::new(format!(
                        "`{rest}` is not a projection of a place"
                    )))
                }
            },
            other => (place(&other)?, None, Vec::new()),
        };
        Ok(Lend {
            receiver,
            projection,
            args,
            mutable,
        })
    }
}

fn place(term: &Term) -> Result<String, Error> {
    match term {
        Term::Var(name) => Ok(name.clone()),
        Term::Field(base, field) => Ok(format!("{}.{field}", place(base)?)),
        other => Err(Error::new(format!("`{other}` is not a place"))),
    }
}

/// Whether two lends may be live at the same time.
#[derive(Clone, PartialEq, Debug)]
pub enum Aliasing {
    /// Neither lend writes.
    Shared,
    /// Different variables or different fields: nothing to prove.
    Separate,
    /// The same projection, with arguments its law proves disjoint.
    Disjoint,
    /// The lends may overlap. When the law is refuted the
    /// counterexample gives arguments for which they do.
    Conflict {
        reason: String,
        counterexample: Option<Counterexample>,
    },
}

impl Aliasing {
    pub fn is_allowed(&self) -> bool {
        !matches!(self, Aliasing::Conflict { .. })
    }
}

impl std::fmt::Display for Aliasing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aliasing::Shared => write!(f, "allowed: neither lend writes"),
            Aliasing::Separate => write!(f, "allowed: different places"),
            Aliasing::Disjoint => write!(f, "allowed: the law is proved"),
            Aliasing::Conflict {
                reason,
                counterexample: Some(counterexample),
            } => {
                write!(f, "conflict: {reason} for {counterexample}")
            }
            Aliasing::Conflict {
                reason,
                counterexample: None,
            } => write!(f, "conflict: {reason}"),
        }
    }
}

impl Context {
    /// Decide whether lends `a` and `b` may be live together. `law` is
    /// the `law disjoint` of their projection, if it declares one.
    pub fn check_lends(
        &mut self,
        a: &Lend,
        b: &Lend,
        law: Option<&Law>,
    ) -> Result<Aliasing, Error> {
        let conflict = |reason: String| {
            Ok(Aliasing::Conflict {
                reason,
                counterexample: None,
            })
        };
        if !a.mutable && !b.mutable {
            return Ok(Aliasing::Shared);
        }
        if a.receiver != b.receiver {
            let inside = |outer: &str, inner: &str| {
                inner
                    .strip_prefix(outer)
                    .is_some_and(|rest| rest.starts_with('.'))
            };
            return if inside(&a.receiver, &b.receiver) || inside(&b.receiver, &a.receiver) {
                conflict(format!(
                    "`{}` and `{}` are a place and a part of it",
                    a.receiver, b.receiver
                ))
            } else {
                Ok(Aliasing::Separate)
            };
        }
        let receiver = &a.receiver;
        let projection = match (&a.projection, &b.projection) {
            (Some(p), Some(q)) if p == q => p,
            (Some(p), Some(q)) => {
                return conflict(format!(
                    "`{p}` and `{q}` are different projections of `{receiver}`: a law only relates two uses of one projection"
                ));
            }
            _ => {
                return conflict(format!(
                    "`{receiver}` is lent as a whole while a lend of it is live"
                ))
            }
        };
        let Some(law) = law else {
            return conflict(format!("`{projection}` declares no `law disjoint`"));
        };
        if law.params.len() != 2 {
            return Err(Error::new(format!(
                "`{law}` must take the arguments of two lends"
            )));
        }
        let pack = |args: &[Term]| match args {
            [one] => one.clone(),
            many => Term::Tuple(many.iter().map(|arg| (None, arg.clone())).collect()),
        };
        let params = law.params.iter().map(|(name, _)| name.clone()).collect();
        let instance = Term::Call(
            Box::new(Term::Lambda(params, Box::new(law.body.clone()))),
            vec![pack(&a.args), pack(&b.args)],
        );
        Ok(match self.prove_term(&instance)? {
            Verdict::Proved => Aliasing::Disjoint,
            Verdict::Refuted(counterexample) => Aliasing::Conflict {
                reason: format!("`{}` does not hold", law.body),
                counterexample: Some(counterexample),
            },
            Verdict::Unknown(reason) => Aliasing::Conflict {
                reason: format!("`{}` is not proved: {reason}", law.body),
                counterexample: None,
            },
        })
    }

    /// Is a `law disjoint` sound: do lends it accepts never share a cell?
    ///
    /// `covers` says which cells a lend touches, as a law of the lend's
    /// arguments and a cell: `covers(i, k) = k == i` for an element,
    /// `covers(s, k) = s.lo <= k and k < s.hi` for a slice. A refutation
    /// names two argument values the law accepts and a cell both touch.
    pub fn check_disjoint_law(&self, law: &Law, covers: &Law) -> Result<Verdict, Error> {
        let ([(x, x_sort), (y, y_sort)], [(_, _), (cell, cell_sort)]) =
            (law.params.as_slice(), covers.params.as_slice())
        else {
            return Err(Error::new(
                "a `law disjoint` takes two lends, and `covers` a lend and a cell",
            ));
        };
        if cell == x || cell == y {
            return Err(Error::new(format!(
                "the cell `{cell}` needs a name of its own"
            )));
        }
        let mut ctx = self.clone();
        ctx.declare(x, x_sort.as_ref().unwrap_or(&Sort::Int))?;
        ctx.declare(y, y_sort.as_ref().unwrap_or(&Sort::Int))?;
        ctx.declare(cell, cell_sort.as_ref().unwrap_or(&Sort::Int))?;
        let lambda = |law: &Law| {
            let params = law.params.iter().map(|(name, _)| name.clone()).collect();
            Box::new(Term::Lambda(params, Box::new(law.body.clone())))
        };
        let accepted = Term::Call(lambda(law), vec![Term::var(x), Term::var(y)]);
        let touches =
            |lend: &str| Term::Call(lambda(covers), vec![Term::var(lend), Term::var(cell)]);
        let overlap = touches(x).and(touches(y));
        ctx.prove_term(&Term::binary(
            crate::term::BinOp::Or,
            accepted.negate(),
            overlap.negate(),
        ))
    }
}
