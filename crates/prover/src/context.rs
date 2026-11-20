//! A proof context: the variables in scope, what is known about them,
//! and the questions that can be asked.

use std::collections::{BTreeSet, HashMap};
use std::fmt;

use crate::core::{Formula, Res};
use crate::eval::{Corner, Env, Rng, Table, Trap};
use crate::interval::Interval;
use crate::lower::{Globals, Hazard, Lowerer, SVal, Store};
use crate::parse;
use crate::solve::{self, Limits, Model, Outcome, Search};
use crate::term::{Error, Sort, Term, Value, INT_MAX, INT_MIN};

/// The answer to "does this always hold?".
#[derive(Clone, PartialEq, Debug)]
pub enum Verdict {
    /// It holds for every value of the variables that satisfies the facts.
    Proved,
    /// It fails for these values. The values were run through the
    /// evaluator: the facts hold for them and the goal does not.
    Refuted(Counterexample),
    /// The prover could not decide, and why. Sampling found no failure.
    Unknown(String),
}

impl Verdict {
    pub fn is_proved(&self) -> bool {
        matches!(self, Verdict::Proved)
    }

    pub fn counterexample(&self) -> Option<&Counterexample> {
        match self {
            Verdict::Refuted(counterexample) => Some(counterexample),
            _ => None,
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Proved => write!(f, "proved"),
            Verdict::Refuted(counterexample) => write!(f, "fails for {counterexample}"),
            Verdict::Unknown(reason) => write!(f, "not proved: {reason}"),
        }
    }
}

/// Values that break a goal.
#[derive(Clone, PartialEq, Debug)]
pub struct Counterexample {
    /// The variables the goal depends on, in declaration order.
    pub bindings: Vec<(String, Value)>,
    /// The array elements and function results that were read, as
    /// (`a[3]`, value).
    pub reads: Vec<(String, Value)>,
    /// Every value of the run, to evaluate further terms at.
    state: State,
}

/// The values behind a counterexample. Two counterexamples are compared
/// by what they show, so this takes no part in equality.
#[derive(Clone, Default)]
struct State {
    vars: HashMap<String, Value>,
    tables: std::collections::BTreeMap<String, Table>,
}

impl PartialEq for State {
    fn eq(&self, _: &State) -> bool {
        true
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "..")
    }
}

impl Counterexample {
    /// The value of a variable or of an element such as `a[3]`. A
    /// variable the goal does not depend on is not among the bindings,
    /// but it has a value all the same.
    pub fn get(&self, name: &str) -> Option<&Value> {
        let shown = self
            .bindings
            .iter()
            .chain(&self.reads)
            .find(|(n, _)| n == name)
            .map(|(_, v)| v);
        shown.or_else(|| self.state.vars.get(name))
    }

    /// The value of an integer variable or element.
    pub fn int(&self, name: &str) -> Option<i128> {
        match self.get(name) {
            Some(Value::Int(v)) => Some(*v),
            _ => None,
        }
    }
}

impl fmt::Display for Counterexample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bindings.is_empty() && self.reads.is_empty() {
            return write!(f, "every input");
        }
        for (i, (name, value)) in self.bindings.iter().chain(&self.reads).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name} = {value}")?;
        }
        Ok(())
    }
}

/// What can go wrong when an expression runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trouble {
    /// An arithmetic result outside `Int.MIN..=Int.MAX`.
    Overflow,
    DivisionByZero,
    /// `x!` on `none`.
    UnwrapNone,
}

/// One operation of an expression and whether it is safe.
#[derive(Clone, PartialEq, Debug)]
pub struct Finding {
    pub trouble: Trouble,
    /// The operation, as written.
    pub expr: String,
    /// For overflow: the range the result can take, as far as intervals
    /// can tell. An open end means "no bound known".
    pub range: Option<Interval>,
    /// `Proved` means the operation is safe.
    pub verdict: Verdict,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.trouble {
            Trouble::Overflow => "overflow Int",
            Trouble::DivisionByZero => "divide by zero",
            Trouble::UnwrapNone => "unwrap none",
        };
        match &self.verdict {
            Verdict::Proved => write!(f, "`{}` cannot {what}", self.expr),
            Verdict::Refuted(counterexample) => {
                write!(f, "`{}` can {what}: {counterexample}", self.expr)
            }
            Verdict::Unknown(reason) => write!(f, "`{}` may {what}: {reason}", self.expr),
        }
    }
}

#[derive(Clone)]
enum Kind {
    /// An unknown of the given type.
    Input { sort: Sort, value: SVal },
    /// The result of an assignment: shown in counterexamples.
    Assigned,
    /// A named term (`op`, `id`): not shown.
    Defined,
}

#[derive(Clone)]
struct Decl {
    /// The name as the user wrote it.
    name: String,
    /// The name of this version: `j`, then `j@1` after a reassignment.
    internal: String,
    kind: Kind,
}

#[derive(Clone)]
struct Fact {
    /// The fact as a term, for the evaluator; the facts a type gives have none.
    term: Option<Term>,
    formula: Formula,
}

/// What a candidate counterexample has to do.
enum Target<'a> {
    /// Make the term false.
    Falsify(&'a Term),
    /// Make the term trap at the named operation.
    Trap(&'a Term, Trouble, &'a str),
}

/// The variables in scope and what is known about them.
///
/// ```
/// use cpl_prover::Context;
/// let mut ctx = Context::new();
/// ctx.var("n", "1..=2e5").unwrap();
/// ctx.var("i", "0..n").unwrap();
/// ctx.var("j", "0..n").unwrap();
/// assert!(ctx.prove("i + 1 <= n").unwrap().is_proved());
/// ctx.assume("i < j").unwrap();
/// assert!(ctx.prove("i != j and i + 1 < n").unwrap().is_proved());
/// let failed = ctx.prove("j + 1 < n").unwrap();
/// assert_eq!(failed.to_string(), "fails for n = 2, i = 0, j = 1");
/// ```
#[derive(Clone, Default)]
pub struct Context {
    store: Store,
    globals: Globals,
    decls: Vec<Decl>,
    /// The current version of each name.
    current: HashMap<String, String>,
    versions: HashMap<String, usize>,
    facts: Vec<Fact>,
    /// For each assignment so far: its arithmetic stayed inside `Int`.
    safety: Vec<Formula>,
    /// How much work a query may take.
    pub limits: Limits,
}

impl Context {
    pub fn new() -> Context {
        Context::default()
    }

    /// Declare a variable of a type: `Int`, `Bool`, `0..n`, `1..=2e5`,
    /// `(Int, Int)`, `(lo: 0..n, hi: 0..=n)`, `?Int`.
    pub fn var(&mut self, name: &str, sort: &str) -> Res<()> {
        self.declare(name, &parse::sort(sort)?)
    }

    pub fn declare(&mut self, name: &str, sort: &Sort) -> Res<()> {
        let sort = self.resolve_sort(sort);
        let internal = self.next_version(name);
        let (value, facts) =
            Lowerer::new(&mut self.store, &self.globals).fresh(&internal, &sort)?;
        self.globals.vars.insert(internal.clone(), value.clone());
        self.facts.push(Fact {
            term: None,
            formula: facts,
        });
        self.bind(name, internal, Kind::Input { sort, value });
        Ok(())
    }

    /// Name a term: a constant, or a function such as `(a, b) => a + b`.
    pub fn define(&mut self, name: &str, term: &str) -> Res<()> {
        self.define_term(name, &parse::term(term)?, Kind::Defined)
    }

    /// `name = term`: from here on the name means the new value. What was
    /// known about the old value stays true of the old value only.
    pub fn assign(&mut self, name: &str, term: &str) -> Res<()> {
        self.assign_term(name, &parse::term(term)?)
    }

    pub fn assign_term(&mut self, name: &str, term: &Term) -> Res<()> {
        self.define_term(name, term, Kind::Assigned)
    }

    fn define_term(&mut self, name: &str, term: &Term, kind: Kind) -> Res<()> {
        let term = self.resolve(term);
        let mut lowerer = Lowerer::new(&mut self.store, &self.globals);
        lowerer.obligations = Some(Vec::new());
        let value = lowerer.term(&term)?;
        let obligations = lowerer.obligations.take().unwrap();
        self.safety.extend(no_overflow(&obligations)?);
        let internal = self.next_version(name);
        self.globals.vars.insert(internal.clone(), value);
        self.globals.defs.insert(internal.clone(), term);
        self.bind(name, internal, kind);
        Ok(())
    }

    /// Forget the value of a variable: it was reassigned to something
    /// unknown, or a call may have changed it. Its type is all that stays.
    pub fn havoc(&mut self, name: &str) -> Res<()> {
        let internal = self
            .current
            .get(name)
            .ok_or_else(|| Error::new(format!("`{name}` is not declared")))?;
        let decl = self
            .decls
            .iter()
            .find(|decl| decl.internal == *internal)
            .unwrap();
        let sort = match &decl.kind {
            Kind::Input { sort, .. } => sort.clone(),
            _ => Sort::Int,
        };
        self.declare(name, &sort)
    }

    /// Declare an array by the type of its elements: `a[i]` is then a
    /// value of that type, and equal indices give equal elements.
    pub fn array(&mut self, name: &str, element: &str) -> Res<()> {
        let sort = self.resolve_sort(&parse::sort(element)?);
        self.globals.symbols.insert(name.to_string(), sort);
        Ok(())
    }

    /// Declare a function the prover knows nothing about except the type
    /// of its results: `f(x)` equals `f(y)` whenever `x == y`.
    pub fn function(&mut self, name: &str, result: &str) -> Res<()> {
        self.array(name, result)
    }

    /// Add a fact: a dominating `if` or `while` condition, an `assert`,
    /// a `requires`, an `invariant`.
    pub fn assume(&mut self, fact: &str) -> Res<()> {
        self.assume_term(&parse::term(fact)?)
    }

    pub fn assume_term(&mut self, fact: &Term) -> Res<()> {
        let term = self.resolve(fact);
        let formula = Lowerer::new(&mut self.store, &self.globals).formula(&term)?;
        self.facts.push(Fact {
            term: Some(term),
            formula,
        });
        Ok(())
    }

    /// Does `goal` hold for every value of the variables that satisfies
    /// the facts? An `Err` means the goal is not a well-formed Bool term.
    ///
    /// A run that traps is not a failure of the goal: `a + 1 > a` is
    /// proved, because the one `a` that would break it overflows first,
    /// and [`Context::check_overflow`] is what reports that.
    pub fn prove(&mut self, goal: &str) -> Res<Verdict> {
        self.prove_term(&parse::term(goal)?)
    }

    pub fn prove_term(&mut self, goal: &Term) -> Res<Verdict> {
        let goal = self.resolve(goal);
        let mut lowerer = Lowerer::new(&mut self.store, &self.globals);
        lowerer.obligations = Some(Vec::new());
        let formula = lowerer.formula(&goal)?;
        let obligations = lowerer.obligations.take().unwrap();
        let safety = no_overflow(&obligations)?;
        // Each conjunct of the goal is refuted on its own: its search
        // only wakes the definitions that conjunct depends on.
        let conjuncts = match formula {
            Formula::And(parts) => parts,
            other => vec![other],
        };
        let cases = conjuncts
            .iter()
            .map(Formula::negate)
            .collect::<Res<Vec<_>>>()?;
        Ok(self.decide(&cases, &Target::Falsify(&goal), &safety))
    }

    /// `index in range`, with the range written as `0..n` or `lo..=hi`.
    pub fn check_index(&mut self, index: &str, range: &str) -> Res<Verdict> {
        let Sort::Range { lo, hi, inclusive } = parse::sort(range)? else {
            return Err(Error::new(format!("`{range}` is not a range")));
        };
        self.prove_term(&Term::InRange(
            Box::new(parse::term(index)?),
            lo,
            hi,
            inclusive,
        ))
    }

    /// Every operation of `expr` that can trap, and whether it does: an
    /// arithmetic result that leaves `Int`, a division by zero, an
    /// unwrapped `none`. Operations behind an `if`, an `and` or an `or`
    /// are checked under the condition that guards them.
    pub fn check_traps(&mut self, expr: &str) -> Res<Vec<Finding>> {
        self.check_traps_term(&parse::term(expr)?)
    }

    pub fn check_traps_term(&mut self, expr: &Term) -> Res<Vec<Finding>> {
        let term = self.resolve(expr);
        let mut lowerer = Lowerer::new(&mut self.store, &self.globals);
        lowerer.obligations = Some(Vec::new());
        lowerer.term(&term)?;
        let obligations = lowerer.obligations.take().unwrap();
        let root: Vec<Formula> = self.facts.iter().map(|fact| fact.formula.clone()).collect();
        let mut findings: Vec<Finding> = Vec::new();
        for obligation in obligations {
            let trouble = match obligation.hazard {
                Hazard::Overflow => Trouble::Overflow,
                Hazard::DivByZero => Trouble::DivisionByZero,
                Hazard::Unwrap => Trouble::UnwrapNone,
            };
            if findings
                .iter()
                .any(|f| f.trouble == trouble && f.expr == obligation.text)
            {
                continue;
            }
            let range = obligation.value.as_ref().map(|value| {
                let mut known = root.clone();
                known.push(obligation.path.clone());
                solve::range(&known, &self.store.lazies, value)
            });
            let verdict = if range.is_some_and(|r| r.within(&Interval::new(INT_MIN, INT_MAX))) {
                Verdict::Proved
            } else {
                let unsafe_here =
                    Formula::and([obligation.path.clone(), obligation.safe.negate()?]);
                self.decide(
                    &[unsafe_here],
                    &Target::Trap(&term, trouble, &obligation.text),
                    &[],
                )
            };
            findings.push(Finding {
                trouble,
                expr: obligation.text,
                range,
                verdict,
            });
        }
        Ok(findings)
    }

    /// The arithmetic of `expr` that can leave `Int`.
    pub fn check_overflow(&mut self, expr: &str) -> Res<Vec<Finding>> {
        let mut findings = self.check_traps(expr)?;
        findings.retain(|finding| finding.trouble == Trouble::Overflow);
        Ok(findings)
    }

    /// Whether a sum of `count` values of `step` stays an `Int`: the
    /// accumulator of a loop that runs `count` times. A counterexample
    /// gives a count and a value of `step` whose product leaves `Int`.
    pub fn check_sum(&mut self, step: &str, count: &str) -> Res<Finding> {
        let step_term = self.resolve(&parse::term(step)?);
        let step_value = Lowerer::new(&mut self.store, &self.globals).poly(&step_term)?;
        let root: Vec<Formula> = self.facts.iter().map(|fact| fact.formula.clone()).collect();
        let step_range = solve::range(&root, &self.store.lazies, &step_value);
        let mut scratch = self.clone();
        let name = parse::term(step)?.to_string();
        let sort = Sort::Range {
            lo: Box::new(Term::Int(step_range.lo.unwrap_or(INT_MIN))),
            hi: Box::new(Term::Int(step_range.hi.unwrap_or(INT_MAX))),
            inclusive: true,
        };
        scratch.declare(&name, &sort)?;
        let total = Term::binary(
            crate::term::BinOp::Mul,
            parse::term(count)?,
            Term::var(&name),
        );
        let mut findings = scratch.check_traps_term(&total)?;
        let expr = format!("the sum of {} values of {name}", parse::term(count)?);
        Ok(match findings.pop() {
            Some(finding) => Finding { expr, ..finding },
            None => Finding {
                trouble: Trouble::Overflow,
                expr,
                range: None,
                verdict: Verdict::Proved,
            },
        })
    }

    /// The range of an integer expression, as far as intervals can tell.
    pub fn range_of(&mut self, expr: &str) -> Res<Interval> {
        let term = self.resolve(&parse::term(expr)?);
        let value = Lowerer::new(&mut self.store, &self.globals).poly(&term)?;
        let root: Vec<Formula> = self.facts.iter().map(|fact| fact.formula.clone()).collect();
        Ok(solve::range(&root, &self.store.lazies, &value))
    }

    /// The query "facts and not goal" as an SMT-LIB 2 script, to
    /// cross-check a verdict with an external solver: `unsat` means the
    /// goal is proved.
    pub fn to_smtlib(&mut self, goal: &str) -> Res<String> {
        let goal = self.resolve(&parse::term(goal)?);
        let formula = Lowerer::new(&mut self.store, &self.globals).formula(&goal)?;
        let mut asserts: Vec<Formula> =
            self.facts.iter().map(|fact| fact.formula.clone()).collect();
        asserts.extend(self.store.lazies.iter().map(|lazy| lazy.formula.clone()));
        asserts.push(formula.negate()?);
        Ok(crate::smtlib::script(&self.store.names, &asserts))
    }

    fn next_version(&mut self, name: &str) -> String {
        let version = self.versions.entry(name.to_string()).or_insert(0);
        let internal = if *version == 0 {
            name.to_string()
        } else {
            format!("{name}@{version}")
        };
        *version += 1;
        internal
    }

    fn bind(&mut self, name: &str, internal: String, kind: Kind) {
        self.current.insert(name.to_string(), internal.clone());
        self.decls.push(Decl {
            name: name.to_string(),
            internal,
            kind,
        });
    }

    /// Replace every name by its current version.
    fn resolve(&self, term: &Term) -> Term {
        let mut bound = Vec::new();
        self.resolve_in(term, &mut bound)
    }

    fn resolve_sort(&self, sort: &Sort) -> Sort {
        match sort {
            Sort::Range { lo, hi, inclusive } => Sort::Range {
                lo: Box::new(self.resolve(lo)),
                hi: Box::new(self.resolve(hi)),
                inclusive: *inclusive,
            },
            Sort::Tuple(items) => Sort::Tuple(
                items
                    .iter()
                    .map(|(name, sort)| (name.clone(), self.resolve_sort(sort)))
                    .collect(),
            ),
            Sort::Opt(inner) => Sort::Opt(Box::new(self.resolve_sort(inner))),
            other => other.clone(),
        }
    }

    fn resolve_in(&self, term: &Term, bound: &mut Vec<String>) -> Term {
        let go = |t: &Term, bound: &mut Vec<String>| Box::new(self.resolve_in(t, bound));
        match term {
            Term::Int(_) | Term::Bool(_) | Term::None => term.clone(),
            Term::Var(name) => match self.current.get(name) {
                Some(internal) if !bound.contains(name) => Term::Var(internal.clone()),
                _ => term.clone(),
            },
            Term::Unary(op, x) => Term::Unary(*op, go(x, bound)),
            Term::Binary(op, l, r) => Term::Binary(*op, go(l, bound), go(r, bound)),
            Term::If(c, a, b) => Term::If(go(c, bound), go(a, bound), go(b, bound)),
            Term::Call(callee, args) => Term::Call(
                go(callee, bound),
                args.iter().map(|arg| *go(arg, bound)).collect(),
            ),
            Term::Index(base, args) => Term::Index(
                go(base, bound),
                args.iter().map(|arg| *go(arg, bound)).collect(),
            ),
            Term::Tuple(items) => Term::Tuple(
                items
                    .iter()
                    .map(|(name, item)| (name.clone(), *go(item, bound)))
                    .collect(),
            ),
            Term::Field(base, field) => Term::Field(go(base, bound), field.clone()),
            Term::Lambda(params, body) => {
                let depth = bound.len();
                bound.extend(params.iter().cloned());
                let body = go(body, bound);
                bound.truncate(depth);
                Term::Lambda(params.clone(), body)
            }
            Term::InRange(x, lo, hi, inclusive) => {
                Term::InRange(go(x, bound), go(lo, bound), go(hi, bound), *inclusive)
            }
        }
    }

    /// Look for values that satisfy the facts and one of the cases.
    ///
    /// The first pass reasons over the mathematical integers. If all it
    /// finds are failures that overflow on the way, a second pass adds
    /// `safety` (the goal's own arithmetic stays inside `Int`) and looks
    /// for a failure that a real run would reach.
    fn decide(&self, cases: &[Formula], target: &Target, safety: &[Formula]) -> Verdict {
        let mut env = Env {
            defs: self.globals.defs.clone(),
            symbols: self.globals.symbols.clone(),
            ..Env::default()
        };
        let mut root: Vec<Formula> = self.facts.iter().map(|fact| fact.formula.clone()).collect();
        let vars = self.store.names.len();
        let mut unknown: Option<&'static str> = None;
        for pass in 0..2 {
            let mut overflowed = false;
            unknown = None;
            for case in cases {
                let mut confirm = |model: &Model| {
                    self.load(&mut env, model);
                    if self.hits(&mut env, target, true) {
                        return true;
                    }
                    overflowed |= self.hits(&mut env, target, false);
                    false
                };
                match Search::run(
                    &root,
                    case,
                    &self.store.lazies,
                    vars,
                    self.limits,
                    &mut confirm,
                ) {
                    Outcome::Unsat => {}
                    Outcome::Sat => return Verdict::Refuted(self.report(&mut env, target)),
                    Outcome::Unknown(reason) => unknown = Some(reason),
                }
            }
            if pass == 1 || !overflowed {
                break;
            }
            root.extend(self.safety.iter().chain(safety).cloned());
        }
        let Some(reason) = unknown else {
            return Verdict::Proved;
        };
        // The solver could not decide: try inputs directly.
        let mut rng = Rng::new(0x5EED_CAFE);
        for round in 0..self.limits.samples {
            rng.corner = match round {
                0 => Some(Corner::Highest),
                1 => Some(Corner::Lowest),
                _ => None,
            };
            env.vars.clear();
            env.tables.clear();
            env.reads.clear();
            env.rng = Some(rng.clone());
            // The generator stays in place while the candidate is checked:
            // array elements are drawn when they are first read.
            let hit = self.draw(&mut env) && self.hits(&mut env, target, true);
            rng = env.rng.take().unwrap();
            rng.next_u64();
            if hit {
                return Verdict::Refuted(self.report(&mut env, target));
            }
        }
        let tried = self.limits.samples;
        Verdict::Unknown(format!(
            "{reason} ({tried} sampled inputs found no failure)"
        ))
    }

    /// Fill the environment with the values of a model.
    fn load(&self, env: &mut Env, model: &Model) {
        let value = |v| model.get(&v).copied().unwrap_or(0);
        env.vars.clear();
        env.tables.clear();
        env.reads.clear();
        for decl in &self.decls {
            if let Kind::Input {
                value: symbolic, ..
            } = &decl.kind
            {
                env.vars
                    .insert(decl.internal.clone(), concrete(symbolic, &value));
            }
        }
        for app in &self.store.apps {
            // An element the search never looked at has no value yet; the
            // evaluator picks one of its type if it is read.
            if !app.result.vars().iter().all(|v| model.contains_key(v)) {
                continue;
            }
            let args: Vec<Value> = app.args.iter().map(|arg| concrete(arg, &value)).collect();
            let table: &mut Table = env.tables.entry(app.symbol.clone()).or_default();
            table
                .entry(args)
                .or_insert_with(|| concrete(&app.result, &value));
        }
    }

    /// Fill the environment with pseudo-random values of the right types.
    fn draw(&self, env: &mut Env) -> bool {
        for decl in &self.decls {
            if let Kind::Input { sort, .. } = &decl.kind {
                match env.inhabitant(sort) {
                    Ok(value) => env.vars.insert(decl.internal.clone(), value),
                    Err(_) => return false,
                };
            }
        }
        true
    }

    /// Whether the values in the environment are a counterexample: every
    /// variable is of its type, every fact holds, and the target is hit.
    ///
    /// With `strict`, a goal counts as false only if evaluating it stays
    /// inside `Int`, as it must for a real run to get that far.
    fn hits(&self, env: &mut Env, target: &Target, strict: bool) -> bool {
        env.reads.clear();
        env.strict = false;
        for decl in &self.decls {
            if let Kind::Input { sort, .. } = &decl.kind {
                let Some(value) = env.vars.get(&decl.internal).cloned() else {
                    return false;
                };
                if env.inhabits(&value, sort) != Ok(true) {
                    return false;
                }
            }
        }
        for fact in &self.facts {
            if let Some(term) = &fact.term {
                if env.eval_bool(term) != Ok(true) {
                    return false;
                }
            }
        }
        match target {
            Target::Falsify(goal) => {
                env.strict = strict;
                let value = env.eval_bool(goal);
                env.strict = false;
                value == Ok(false)
            }
            Target::Trap(term, trouble, at) => {
                env.strict = true;
                let trap = env.eval(term).err();
                env.strict = false;
                match (trouble, trap) {
                    (Trouble::Overflow, Some(Trap::Overflow(text))) => text == *at,
                    (Trouble::DivisionByZero, Some(Trap::DivByZero)) => true,
                    (Trouble::UnwrapNone, Some(Trap::UnwrapNone)) => true,
                    _ => false,
                }
            }
        }
    }

    /// Turn the confirmed environment into a counterexample: shrink the
    /// values toward zero while they still fail, then keep the variables
    /// the goal depends on.
    fn report(&self, env: &mut Env, target: &Target) -> Counterexample {
        env.rng = None;
        self.shrink(env, target);
        let confirmed = self.hits(env, target, true);
        debug_assert!(confirmed, "a shrunk counterexample must still fail");
        let goal = match target {
            Target::Falsify(term) | Target::Trap(term, ..) => *term,
        };
        let relevant = self.cone(goal);
        let mut bindings = Vec::new();
        for decl in &self.decls {
            let is_current = self.current.get(&decl.name) == Some(&decl.internal);
            if !is_current || !relevant.contains(&decl.internal) {
                continue;
            }
            let value = match decl.kind {
                Kind::Input { .. } => env.vars.get(&decl.internal).cloned(),
                Kind::Assigned => env.eval(&Term::var(&decl.internal)).ok(),
                Kind::Defined => None,
            };
            if let Some(value) = value {
                bindings.push((decl.name.clone(), value));
            }
        }
        // Read the goal once more so that only its own reads are listed.
        env.reads.clear();
        env.strict = matches!(target, Target::Trap(..));
        let _ = env.eval(goal);
        env.strict = false;
        let mut reads: Vec<(String, Value)> = Vec::new();
        for (symbol, args) in std::mem::take(&mut env.reads) {
            let shown: Vec<String> = args.iter().map(Value::to_string).collect();
            let name = format!("{symbol}[{}]", shown.join(", "));
            if reads.iter().all(|(seen, _)| *seen != name) {
                let value = env.tables[&symbol][&args].clone();
                reads.push((name, value));
            }
        }
        let state = State {
            vars: env.vars.clone(),
            tables: env.tables.clone(),
        };
        Counterexample {
            bindings,
            reads,
            state,
        }
    }

    /// The value of a term at a counterexample this context produced:
    /// the two sides of a broken equation, say. `None` if it traps.
    pub fn eval_at(&self, counterexample: &Counterexample, term: &Term) -> Option<Value> {
        let mut env = Env {
            vars: counterexample.state.vars.clone(),
            tables: counterexample.state.tables.clone(),
            defs: self.globals.defs.clone(),
            symbols: self.globals.symbols.clone(),
            ..Env::default()
        };
        env.eval(&self.resolve(term)).ok()
    }

    fn shrink(&self, env: &mut Env, target: &Target) {
        // Reading a table fills it in, so the tables are pinned first.
        if !self.hits(env, target, true) {
            return;
        }
        let mut budget = 600u32;
        let mut progress = true;
        while progress && budget > 0 {
            progress = false;
            let names: Vec<String> = env.vars.keys().cloned().collect();
            let mut slots: Vec<Slot> = names.into_iter().map(Slot::Var).collect();
            for (symbol, table) in &env.tables {
                slots.extend(
                    table
                        .keys()
                        .map(|args| Slot::Element(symbol.clone(), args.clone())),
                );
            }
            slots.sort();
            for slot in slots {
                let Some(original) = slot.get(env).cloned() else {
                    continue;
                };
                for leaf in 0..leaves(&original) {
                    let current = |env: &Env| slot.get(env).and_then(|value| leaf_at(value, leaf));
                    let mut attempt = |env: &mut Env, smaller: i128| -> bool {
                        let (Some(now), Some(current)) = (slot.get(env).cloned(), current(env))
                        else {
                            return false;
                        };
                        // Smaller means closer to zero, or positive instead of negative.
                        let flipped = smaller == -current && current < 0;
                        if (smaller.abs() >= current.abs() && !flipped) || budget == 0 {
                            return false;
                        }
                        budget -= 1;
                        let saved = env.clone();
                        let mut changed = now;
                        set_leaf(&mut changed, leaf, smaller);
                        slot.set(env, changed);
                        if self.hits(env, target, true) {
                            return true;
                        }
                        *env = saved;
                        false
                    };
                    // Zero if it will do; else halve while that still
                    // fails, then close in on the smallest failing value.
                    if attempt(env, 0) {
                        progress = true;
                        continue;
                    }
                    if current(env).is_some_and(|value| attempt(env, -value)) {
                        progress = true;
                    }
                    while current(env).is_some_and(|value| attempt(env, value / 2)) {
                        progress = true;
                    }
                    let Some(value) = current(env) else { continue };
                    let (mut passes, mut fails) = (value / 2, value);
                    while (fails - passes).abs() > 1 {
                        let middle = passes + (fails - passes) / 2;
                        if attempt(env, middle) {
                            fails = middle;
                            progress = true;
                        } else {
                            passes = middle;
                        }
                    }
                }
            }
        }
    }

    /// The versions of variables the goal depends on: the ones it
    /// mentions, and the ones tied to those by a fact or a type.
    fn cone(&self, goal: &Term) -> BTreeSet<String> {
        let mut cone = BTreeSet::new();
        free_names(goal, &mut Vec::new(), &mut cone);
        loop {
            let before = cone.len();
            for name in cone.clone() {
                if let Some(def) = self.globals.defs.get(&name) {
                    free_names(def, &mut Vec::new(), &mut cone);
                }
                let decl = self.decls.iter().find(|decl| decl.internal == name);
                if let Some(Decl {
                    kind: Kind::Input { sort, .. },
                    ..
                }) = decl
                {
                    sort_names(sort, &mut cone);
                }
            }
            for fact in &self.facts {
                let Some(term) = &fact.term else { continue };
                let mut names = BTreeSet::new();
                free_names(term, &mut Vec::new(), &mut names);
                // Names of functions and arrays tie nothing together.
                names.retain(|name| !self.globals.symbols.contains_key(name));
                if names
                    .iter()
                    .any(|name| cone.contains(name) && !self.is_function(name))
                {
                    cone.extend(names);
                }
            }
            if cone.len() == before {
                return cone;
            }
        }
    }

    fn is_function(&self, name: &str) -> bool {
        matches!(self.globals.defs.get(name), Some(Term::Lambda(..)))
    }
}

/// "Under its path condition, this arithmetic stays inside `Int`", for
/// every arithmetic operation among the obligations.
fn no_overflow(obligations: &[crate::lower::Obligation]) -> Res<Vec<Formula>> {
    let mut out = Vec::new();
    for obligation in obligations.iter().filter(|o| o.hazard == Hazard::Overflow) {
        let safe = Formula::or([obligation.path.negate()?, obligation.safe.clone()]);
        if !out.contains(&safe) {
            out.push(safe);
        }
    }
    Ok(out)
}

/// A place a counterexample holds a value: a variable or an element.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Slot {
    Var(String),
    Element(String, Vec<Value>),
}

impl Slot {
    fn get<'a>(&self, env: &'a Env) -> Option<&'a Value> {
        match self {
            Slot::Var(name) => env.vars.get(name),
            Slot::Element(symbol, args) => env.tables.get(symbol)?.get(args),
        }
    }

    fn set(&self, env: &mut Env, value: Value) {
        match self {
            Slot::Var(name) => {
                env.vars.insert(name.clone(), value);
            }
            Slot::Element(symbol, args) => {
                env.tables
                    .entry(symbol.clone())
                    .or_default()
                    .insert(args.clone(), value);
            }
        }
    }
}

fn leaves(value: &Value) -> usize {
    match value {
        Value::Int(_) => 1,
        Value::Tuple(items) => items.iter().map(|(_, item)| leaves(item)).sum(),
        _ => 0,
    }
}

fn leaf_at(value: &Value, index: usize) -> Option<i128> {
    match value {
        Value::Int(v) => (index == 0).then_some(*v),
        Value::Tuple(items) => {
            let mut index = index;
            for (_, item) in items {
                let n = leaves(item);
                if index < n {
                    return leaf_at(item, index);
                }
                index -= n;
            }
            None
        }
        _ => None,
    }
}

fn set_leaf(value: &mut Value, index: usize, to: i128) {
    match value {
        Value::Int(v) if index == 0 => *v = to,
        Value::Tuple(items) => {
            let mut index = index;
            for (_, item) in items {
                let n = leaves(item);
                if index < n {
                    return set_leaf(item, index, to);
                }
                index -= n;
            }
        }
        _ => {}
    }
}

/// The value a symbolic value takes under an assignment.
fn concrete(symbolic: &SVal, value: &impl Fn(u32) -> i128) -> Value {
    match symbolic {
        SVal::Int(p) => Value::Int(p.eval(value).unwrap_or(0)),
        SVal::Bool(f) => Value::Bool(f.eval(value).unwrap_or(false)),
        SVal::Tuple(items) => Value::Tuple(
            items
                .iter()
                .map(|(name, item)| (name.clone(), concrete(item, value)))
                .collect(),
        ),
        SVal::Opt {
            some,
            value: payload,
        } => match some.eval(value) {
            Some(true) => concrete(payload, value),
            _ => Value::None,
        },
        SVal::NoneLit | SVal::Lambda(_) | SVal::Named(_) => Value::None,
    }
}

fn free_names(term: &Term, bound: &mut Vec<String>, out: &mut BTreeSet<String>) {
    match term {
        Term::Int(_) | Term::Bool(_) | Term::None => {}
        Term::Var(name) => {
            if !bound.contains(name) {
                out.insert(name.clone());
            }
        }
        Term::Unary(_, x) | Term::Field(x, _) => free_names(x, bound, out),
        Term::Binary(_, l, r) => {
            free_names(l, bound, out);
            free_names(r, bound, out);
        }
        Term::If(a, b, c) | Term::InRange(a, b, c, _) => {
            free_names(a, bound, out);
            free_names(b, bound, out);
            free_names(c, bound, out);
        }
        Term::Call(head, args) | Term::Index(head, args) => {
            free_names(head, bound, out);
            args.iter().for_each(|arg| free_names(arg, bound, out));
        }
        Term::Tuple(items) => items
            .iter()
            .for_each(|(_, item)| free_names(item, bound, out)),
        Term::Lambda(params, body) => {
            let depth = bound.len();
            bound.extend(params.iter().cloned());
            free_names(body, bound, out);
            bound.truncate(depth);
        }
    }
}

fn sort_names(sort: &Sort, out: &mut BTreeSet<String>) {
    match sort {
        Sort::Range { lo, hi, .. } => {
            free_names(lo, &mut Vec::new(), out);
            free_names(hi, &mut Vec::new(), out);
        }
        Sort::Tuple(items) => items.iter().for_each(|(_, sort)| sort_names(sort, out)),
        Sort::Opt(inner) => sort_names(inner, out),
        Sort::Int | Sort::Bool | Sort::Named(_) => {}
    }
}
