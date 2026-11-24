//! Cases: the values of one input, made from an [`InputSpec`].

use std::borrow::Cow;
use std::collections::HashSet;

use crate::rng::Rng;
use crate::spec::{eval, Arm, Env, GraphKind, InputSpec, Shape};

/// One value of an input.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Int(i128),
    /// One byte, written as itself.
    Char(i128),
    Seq(Vec<Value>),
    Tuple(Vec<Value>),
    /// The word at `at` of `words`, the tag of a choice the program
    /// makes by comparing a `Str` with literals.
    Word {
        at: i128,
        words: std::sync::Arc<[String]>,
    },
    /// What a `read(gen: f)` fn printed when its `rand` started from
    /// `seed`; another seed is another value.
    Drawn {
        seed: u64,
        text: String,
    },
}

impl Value {
    /// The number a number or a byte holds.
    pub fn number(&self) -> Option<i128> {
        match self {
            Value::Int(v) | Value::Char(v) | Value::Word { at: v, .. } => Some(*v),
            _ => None,
        }
    }

    /// The same kind of value holding `to`.
    pub fn with(&self, to: i128) -> Value {
        match self {
            Value::Char(_) => Value::Char(to),
            Value::Word { words, .. } => Value::Word {
                at: to.clamp(0, words.len() as i128 - 1),
                words: words.clone(),
            },
            _ => Value::Int(to),
        }
    }
}

/// A field's name, its numbers' range and its longest length.
type FieldReach = (String, Option<(i128, i128)>, Option<usize>);

/// How far the cases of a clean run reached, field by field: the
/// smallest and largest number, and the longest list (use case stress/U31).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Reach {
    /// By field, in reading order: its name, its numbers' range, its
    /// longest length when it is a list.
    fields: Vec<FieldReach>,
    /// The most test cases one input had, under `@!multitest`.
    tests: usize,
    /// Over generated inputs, read as text: the most numbers one had and
    /// their range.
    generated: Option<(usize, i128, i128)>,
}

impl Reach {
    /// Takes in one case made from `spec`.
    pub fn add(&mut self, spec: &InputSpec, case: &Case) {
        fn numbers(value: &Value, range: &mut Option<(i128, i128)>) {
            match value {
                Value::Int(v) => {
                    *range = Some(range.map_or((*v, *v), |(lo, hi)| (lo.min(*v), hi.max(*v))));
                }
                Value::Seq(items) | Value::Tuple(items) => {
                    items.iter().for_each(|item| numbers(item, range))
                }
                _ => {}
            }
        }
        for (field, value) in spec.fields.iter().zip(&case.values) {
            let name = (field.names.iter().flatten().next().cloned())
                .unwrap_or_else(|| format!("line {}", field.line));
            let at = match self.fields.iter().position(|(known, ..)| *known == name) {
                Some(at) => at,
                None => {
                    self.fields.push((name, None, None));
                    self.fields.len() - 1
                }
            };
            let (_, range, longest) = &mut self.fields[at];
            numbers(value, range);
            if let Value::Seq(items) = value {
                *longest = Some(longest.unwrap_or(0).max(items.len()));
            }
        }
        if let Some(inner) = &spec.cases {
            self.tests = self.tests.max(case.cases.len());
            case.cases.iter().for_each(|test| self.add(inner, test));
        }
    }

    /// Takes in one input a generator wrote, which has no fields to name.
    pub fn add_text(&mut self, input: &str) {
        let numbers: Vec<i128> = (input.split_ascii_whitespace())
            .filter_map(|token| token.parse().ok())
            .collect();
        let (Some(&lo), Some(&hi)) = (numbers.iter().min(), numbers.iter().max()) else {
            return;
        };
        let (most, low, high) = self.generated.unwrap_or((0, lo, hi));
        self.generated = Some((most.max(numbers.len()), low.min(lo), high.max(hi)));
    }

    /// `n from 1 to 8, a up to 8 long from -3 to 5, up to 4 tests`, or for
    /// generated inputs `inputs of up to 9 numbers from 1 to 40`; nothing
    /// when no case passed.
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = (self.fields.iter())
            .filter_map(|(name, range, longest)| {
                let long = longest.map(|most| format!(" up to {most} long"));
                let range = range.map(|(lo, hi)| match lo == hi {
                    true => format!(" always {lo}"),
                    false => format!(" from {lo} to {hi}"),
                });
                (long.is_some() || range.is_some()).then(|| {
                    format!(
                        "{name}{}{}",
                        long.unwrap_or_default(),
                        range.unwrap_or_default()
                    )
                })
            })
            .collect();
        if self.tests > 0 {
            parts.push(format!("up to {} tests", self.tests));
        }
        if let Some((most, lo, hi)) = self.generated {
            let s = if most == 1 { "" } else { "s" };
            parts.push(format!(
                "inputs of up to {most} number{s} from {lo} to {hi}"
            ));
        }
        parts.join(", ")
    }
}

/// One input: a value per `read()`, in order.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Case {
    pub values: Vec<Value>,
    /// Under `@!multitest`: the test cases, after `values`.
    pub cases: Vec<Case>,
    /// The test cases are written with no count before them.
    pub until_eof: bool,
}

/// How large a case may be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    /// Sizes (`n`, `q`) are at most this; most values stay within this of
    /// their lower bound, so equal values and ties are common. One value
    /// in four uses its whole range, to reach overflow.
    Small(i128),
    /// Every size and value may be anything its type allows.
    Full,
}

/// What the integers of a case are drawn from. The stress generator
/// draws at random; `cpl bench` draws the worst cases.
pub trait Pick {
    /// A value in `lo..=hi`. `is_size` is true for a number that decides
    /// how much input follows (`n`, `q`).
    fn pick(&mut self, lo: i128, hi: i128, is_size: bool) -> i128;

    /// How the next tree hangs together. Hanging each vertex under a
    /// uniformly earlier one alone makes shallow trees, and misses the
    /// deep and the wide ones that break solutions.
    fn tree(&mut self) -> TreeShape {
        match self.pick(0, 5, false) {
            0..=2 => TreeShape::Random,
            3 => TreeShape::Long,
            4 => TreeShape::Path,
            _ => TreeShape::Star,
        }
    }
}

/// The shape of a generated tree, or of the tree a connected graph
/// starts from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeShape {
    /// Each vertex under a uniformly earlier one.
    Random,
    /// Each vertex under one of the three before it: deep and bushy.
    Long,
    /// One chain.
    Path,
    /// Every vertex under the first.
    Star,
}

/// The `(parent, child)` positions of a tree on `n` vertices, each child
/// after its parent.
fn hang(n: i128, pick: &mut dyn Pick) -> Vec<(i128, i128)> {
    let shape = pick.tree();
    (1..n)
        .map(|child| {
            let parent = match shape {
                TreeShape::Random => pick.pick(0, child - 1, false),
                TreeShape::Long => pick.pick((child - 3).max(0), child - 1, false),
                TreeShape::Path => child - 1,
                TreeShape::Star => 0,
            };
            (parent, child)
        })
        .collect()
}

/// The most elements one array of a generated case may have.
pub const MAX_LEN: i128 = 20_000_000;

struct Random<'a> {
    rng: &'a mut Rng,
    size: Size,
}

impl Pick for Random<'_> {
    fn pick(&mut self, lo: i128, hi: i128, is_size: bool) -> i128 {
        let hi = match self.size {
            Size::Full => hi,
            Size::Small(limit) if is_size => hi.min(limit.max(lo)),
            Size::Small(limit) => {
                if self.rng.one_in(4) {
                    hi
                } else {
                    hi.min(lo.saturating_add(limit))
                }
            }
        };
        // The ends of a range are where off-by-ones live.
        match self.rng.next_u64() % 8 {
            0 => lo,
            1 => hi,
            _ => self.rng.range(lo, hi),
        }
    }
}

pub(crate) fn build_parts(
    parts: &[Shape],
    binds: &[Vec<Option<String>>],
    env: &Env,
    sizes: &HashSet<String>,
    pick: &mut dyn Pick,
) -> Option<Vec<Value>> {
    let mut env = Cow::Borrowed(env);
    let mut built = Vec::new();
    for (at, part) in parts.iter().enumerate() {
        let is_size =
            matches!(binds.get(at).map(Vec::as_slice), Some([Some(name)]) if sizes.contains(name));
        let value = build(part, &env, sizes, is_size, pick)?;
        if let Some(names) = binds.get(at).filter(|names| !names.is_empty()) {
            bind(names, &value, env.to_mut());
        }
        built.push(value);
    }
    Some(built)
}

/// A tag that takes an arm picked evenly, so each kind of query comes
/// up as often as the others whatever the tag's type allows. A literal
/// tag is used even outside a `Char`'s usual letters; an `Int` keeps to
/// its declared range.
fn choose_tag(tag: &Shape, arms: &[Arm], env: &Env, pick: &mut dyn Pick) -> Option<Value> {
    let (lo, hi, char) = match tag {
        Shape::Int { lo, hi } => (eval(lo, env)?, eval(hi, env)?, false),
        Shape::Char { lo, hi } => (i128::from(*lo), i128::from(*hi), true),
        Shape::Bool => (0, 1, false),
        Shape::Words(words) => (0, words.len() as i128 - 1, false),
        _ => return None,
    };
    (lo <= hi).then_some(())?;
    let wrap = |v: i128| match tag {
        Shape::Words(words) => Value::Word {
            at: v,
            words: words.clone(),
        },
        _ if char => Value::Char(v),
        _ => Value::Int(v),
    };
    let at = usize::try_from(pick.pick(0, arms.len().saturating_sub(1) as i128, false)).ok()?;
    let arm = arms.get(at)?;
    if let Some(tags) = &arm.tags {
        let fits: Vec<i128> = tags
            .iter()
            .copied()
            .filter(|v| char || (lo..=hi).contains(v))
            .collect();
        if !fits.is_empty() {
            let at = pick.pick(0, fits.len() as i128 - 1, false) as usize;
            return Some(wrap(fits[at]));
        }
    }
    let mut value = pick.pick(lo, hi, false);
    for _ in 0..8 {
        if Arm::taken(arms, value).is_some_and(|taken| std::ptr::eq(taken, arm)) {
            break;
        }
        value = pick.pick(lo, hi, false);
    }
    Some(wrap(value))
}

/// Every number at the low end of its range.
pub(crate) struct Lowest;

impl Pick for Lowest {
    fn pick(&mut self, lo: i128, _hi: i128, _is_size: bool) -> i128 {
        lo
    }
}

fn build(
    shape: &Shape,
    env: &Env,
    sizes: &HashSet<String>,
    is_size: bool,
    pick: &mut dyn Pick,
) -> Option<Value> {
    match shape {
        Shape::Int { lo, hi } => {
            let (lo, hi) = (eval(lo, env)?, eval(hi, env)?);
            (lo <= hi).then(|| Value::Int(pick.pick(lo, hi, is_size)))
        }
        Shape::Array { len, elem, index } => {
            let len = eval(len, env)?.max(0);
            (len <= MAX_LEN).then_some(())?;
            (0..len)
                .map(|at| build(elem, &*indexed(index, at, env)?, sizes, false, pick))
                .collect::<Option<_>>()
                .map(Value::Seq)
        }
        Shape::Tuple { parts, binds } => {
            build_parts(parts, binds, env, sizes, pick).map(Value::Tuple)
        }
        Shape::Choice { tag, name, arms } => {
            let tag = choose_tag(tag, arms, env, pick)?;
            let value = tag.number()?;
            let mut env = env.clone();
            env.insert(name.clone(), value);
            let mut built = vec![tag];
            if let Some(arm) = Arm::taken(arms, value) {
                built.extend(build_parts(&arm.parts, &arm.binds, &env, sizes, pick)?);
            }
            Some(Value::Tuple(built))
        }
        Shape::Char { lo, hi } => Some(Value::Char(pick.pick(
            i128::from(*lo),
            i128::from(*hi),
            false,
        ))),
        Shape::Bool => Some(Value::Int(pick.pick(0, 1, false))),
        Shape::Words(words) => Some(Value::Word {
            at: pick.pick(0, words.len() as i128 - 1, false),
            words: words.clone(),
        }),
        Shape::Maybe { inner, none } => match pick.pick(0, 3, false) {
            0 => Some(Value::Int(*none)),
            _ => build(inner, env, sizes, false, pick),
        },
        // Only a top-level field is drawn: it reads the input above it.
        Shape::Gen { .. } => None,
        Shape::Str { alphabet, len } => {
            let (lo, hi) = match len {
                Some((lo, hi)) => (eval(lo, env)?, eval(hi, env)?),
                None => (1, MAX_WORD),
            };
            (lo <= hi && hi >= 0).then_some(())?;
            let len = pick.pick(lo.max(0), hi, true);
            let letter = |pick: &mut dyn Pick| match alphabet {
                Some(letters) => letters[pick.pick(0, letters.len() as i128 - 1, false) as usize],
                None => pick.pick(i128::from(b'a'), i128::from(b'z'), false) as u8,
            };
            Some(Value::Seq(
                (0..len)
                    .map(|_| Value::Char(i128::from(letter(pick))))
                    .collect(),
            ))
        }
        Shape::Perm { lo, len } => {
            let (lo, len) = (eval(lo, env)?, eval(len, env)?.max(0));
            (len <= MAX_LEN).then_some(())?;
            let mut order: Vec<i128> = (lo..lo + len).collect();
            shuffle(&mut order, pick);
            Some(Value::Seq(order.into_iter().map(Value::Int).collect()))
        }
        Shape::Tree { lo, hi, extra } => {
            let (lo, hi) = (eval(lo, env)?, eval(hi, env)?);
            let n = (hi - lo + 1).max(0);
            (n <= MAX_LEN).then_some(())?;
            // Each vertex hangs under an earlier one; then the names
            // and the order of the edges are mixed.
            let mut names: Vec<i128> = (lo..=hi).collect();
            shuffle(&mut names, pick);
            let mut edges: Vec<(i128, i128)> = hang(n, pick)
                .into_iter()
                .map(|(parent, child)| {
                    let (a, b) = (names[parent as usize], names[child as usize]);
                    if pick.pick(0, 1, false) == 0 {
                        (a, b)
                    } else {
                        (b, a)
                    }
                })
                .collect();
            shuffle(&mut edges, pick);
            edges
                .into_iter()
                .map(|(a, b)| {
                    let mut edge = vec![Value::Int(a), Value::Int(b)];
                    for shape in extra {
                        edge.push(build(shape, env, sizes, false, pick)?);
                    }
                    Some(Value::Tuple(edge))
                })
                .collect::<Option<_>>()
                .map(Value::Seq)
        }
        Shape::Graph {
            lo,
            hi,
            len,
            extra,
            kind,
        } => {
            let (lo, hi, m) = (eval(lo, env)?, eval(hi, env)?, eval(len, env)?.max(0));
            let n = (hi - lo + 1).max(0);
            (n <= MAX_LEN && m <= MAX_LEN).then_some(())?;
            let room = if *kind == GraphKind::Digraph {
                n * (n - 1)
            } else {
                n * (n - 1) / 2
            };
            (m <= room).then_some(())?;
            // Position `i` of `names` is the `i`th vertex in a random
            // order: a DAG's edges run forward in it, and a connected
            // graph starts from a tree that hangs each vertex under an
            // earlier one.
            let mut names: Vec<i128> = (lo..=hi).collect();
            shuffle(&mut names, pick);
            let mut chosen: Vec<(i128, i128)> = Vec::new();
            if *kind == GraphKind::Connected {
                (m >= n - 1).then_some(())?;
                chosen.extend(hang(n, pick));
            }
            let mut seen: HashSet<(i128, i128)> = chosen.iter().copied().collect();
            let wanted = usize::try_from(m).ok()?;
            if room <= 4 * m + 64 {
                // Dense: every pair not yet taken, in a random order.
                let both = *kind == GraphKind::Digraph;
                let mut rest: Vec<(i128, i128)> = (0..n)
                    .flat_map(|b| (0..if both { n } else { b }).map(move |a| (a, b)))
                    .filter(|&(a, b)| a != b && !seen.contains(&(a, b)))
                    .collect();
                shuffle(&mut rest, pick);
                rest.truncate(wanted - chosen.len());
                chosen.extend(rest);
            } else {
                while chosen.len() < wanted {
                    let (a, b) = (pick.pick(0, n - 1, false), pick.pick(0, n - 1, false));
                    let pair = if *kind == GraphKind::Digraph {
                        (a, b)
                    } else {
                        (a.min(b), a.max(b))
                    };
                    if a != b && seen.insert(pair) {
                        chosen.push(pair);
                    }
                }
            }
            let mut edges: Vec<(i128, i128)> = chosen
                .into_iter()
                .map(|(a, b)| {
                    let (a, b) = (names[a as usize], names[b as usize]);
                    if !kind.directed() && pick.pick(0, 1, false) == 1 {
                        (b, a)
                    } else {
                        (a, b)
                    }
                })
                .collect();
            shuffle(&mut edges, pick);
            edges
                .into_iter()
                .map(|(a, b)| {
                    let mut edge = vec![Value::Int(a), Value::Int(b)];
                    for shape in extra {
                        edge.push(build(shape, env, sizes, false, pick)?);
                    }
                    Some(Value::Tuple(edge))
                })
                .collect::<Option<_>>()
                .map(Value::Seq)
        }
    }
}

/// The longest word a generated case may hold.
pub const MAX_WORD: i128 = 200_000;

fn shuffle<T>(items: &mut [T], pick: &mut dyn Pick) {
    for i in 0..items.len() {
        let j = pick.pick(i as i128, items.len() as i128 - 1, false);
        items.swap(i, j.clamp(i as i128, items.len() as i128 - 1) as usize);
    }
}

/// The integers of `value`, in the order a pattern names them.
fn leaves(value: &Value, out: &mut Vec<Option<i128>>) {
    match value {
        Value::Int(v) | Value::Word { at: v, .. } => out.push(Some(*v)),
        Value::Tuple(parts) => parts.iter().for_each(|part| leaves(part, out)),
        Value::Seq(_) | Value::Char(_) | Value::Drawn { .. } => out.push(None),
    }
}

/// Adds the names a field binds to `env`.
/// `env` with an array's index name bound for its element at `at`.
pub(crate) fn indexed<'a>(
    index: &Option<(String, cpl_syntax::ast::Expr)>,
    at: i128,
    env: &'a Env,
) -> Option<Cow<'a, Env>> {
    let Some((name, lo)) = index else {
        return Some(Cow::Borrowed(env));
    };
    let mut env = env.clone();
    env.insert(name.clone(), eval(lo, &env)? + at);
    Some(Cow::Owned(env))
}

pub(crate) fn bind(names: &[Option<String>], value: &Value, env: &mut Env) {
    let mut values = Vec::new();
    leaves(value, &mut values);
    for (name, value) in names.iter().zip(values) {
        if let (Some(name), Some(value)) = (name, value) {
            env.insert(name.clone(), value);
        }
    }
}

/// A case whose every integer is chosen by `pick`. `None` when a bound
/// cannot be evaluated, a range is empty, or an array would be longer
/// than [`MAX_LEN`].
pub fn make(spec: &InputSpec, pick: &mut dyn Pick) -> Option<Case> {
    make_in(spec, &spec.consts, pick)
}

/// The most test cases a generated multitest input holds.
const MAX_TESTS: i128 = 5;

fn make_in(spec: &InputSpec, outer: &Env, pick: &mut dyn Pick) -> Option<Case> {
    let mut env: Env = outer.clone();
    let mut values = Vec::new();
    for field in &spec.fields {
        let value = match (&field.shape, field.names.as_slice()) {
            // `let (n, q): (.., ..) = read()`: each part can be a size.
            (Shape::Tuple { parts, binds }, names) if names.len() == parts.len() => {
                let mut built = Vec::new();
                for (at, (part, name)) in parts.iter().zip(names).enumerate() {
                    let is_size = name.as_ref().is_some_and(|n| spec.sizes.contains(n));
                    let value = build(part, &env, &spec.sizes, is_size, pick)?;
                    bind(std::slice::from_ref(name), &value, &mut env);
                    if let Some(own) = binds.get(at) {
                        bind(own, &value, &mut env);
                    }
                    built.push(value);
                }
                Value::Tuple(built)
            }
            (Shape::Gen { program, .. }, _) => {
                let seed = pick.pick(0, i64::MAX as i128, false) as u64;
                draw(program, &values, seed)?
            }
            (shape, names) => {
                let is_size = matches!(names, [Some(name)] if spec.sizes.contains(name));
                build(shape, &env, &spec.sizes, is_size, pick)?
            }
        };
        bind(&field.names, &value, &mut env);
        values.push(value);
    }
    let mut cases = Vec::new();
    if let Some(inner) = &spec.cases {
        // Each test case starts from what was read above the divider.
        for _ in 0..pick.pick(1, MAX_TESTS, true).clamp(1, MAX_TESTS) {
            cases.push(make_in(inner, &env, pick)?);
        }
    }
    Some(Case {
        values,
        cases,
        until_eof: spec.until_eof,
    })
}

/// What the `read(gen: f)` program prints, reading `above` (the values
/// read before it) with `rand` started from `seed`. `None` when it traps.
pub(crate) fn draw(program: &cpl_ir::Program, above: &[Value], seed: u64) -> Option<Value> {
    let mut input = String::new();
    above.iter().for_each(|value| write(value, &mut input));
    let mut out = Vec::new();
    let io = cpl_interp::Io::fixed(input.as_bytes()).seeded(seed);
    cpl_interp::run_with(program, io, &mut out).ok()?;
    let text = String::from_utf8(out).ok()?;
    Some(Value::Drawn { seed, text })
}

/// A random case. `None` when a bound cannot be evaluated or a range is
/// empty (`1..=n - 1` with `n = 1`).
pub fn generate(spec: &InputSpec, rng: &mut Rng, size: Size) -> Option<Case> {
    make(spec, &mut Random { rng, size })
}

/// One line when `value` is a number, a row of numbers, or a word.
fn flat(value: &Value) -> Option<String> {
    let number = |value: &Value| match value {
        Value::Int(v) => Some(v.to_string()),
        _ => None,
    };
    let letter = |value: &Value| match value {
        Value::Char(c) => Some(*c as u8 as char),
        _ => None,
    };
    match value {
        Value::Int(v) => Some(v.to_string()),
        Value::Char(c) => Some((*c as u8 as char).to_string()),
        // Letters are written together, as one word.
        Value::Seq(items)
            if !items.is_empty() && items.iter().all(|item| letter(item).is_some()) =>
        {
            items.iter().map(letter).collect()
        }
        Value::Seq(items) | Value::Tuple(items) => items
            .iter()
            .map(|item| number(item).or_else(|| letter(item).map(String::from)))
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join(" ")),
        Value::Word { at, words } => words.get(*at as usize).cloned(),
        Value::Drawn { text, .. } => Some(text.trim_end().to_string()),
    }
}

fn write(value: &Value, out: &mut String) {
    match (flat(value), value) {
        (Some(line), _) => {
            out.push_str(&line);
            out.push('\n');
        }
        (None, Value::Seq(items) | Value::Tuple(items)) => {
            items.iter().for_each(|item| write(item, out))
        }
        (None, Value::Int(_) | Value::Char(_) | Value::Word { .. } | Value::Drawn { .. }) => {
            unreachable!("a number and a drawn value are always flat")
        }
    }
}

fn show(value: &Value) -> String {
    let joined = |items: &[Value]| items.iter().map(show).collect::<Vec<_>>().join(", ");
    match value {
        Value::Int(v) => v.to_string(),
        Value::Char(c) => format!("'{}'", *c as u8 as char),
        Value::Seq(items)
            if !items.is_empty() && items.iter().all(|item| matches!(item, Value::Char(_))) =>
        {
            format!("\"{}\"", flat(value).unwrap_or_default())
        }
        Value::Seq(items) => format!("[{}]", joined(items)),
        Value::Tuple(items) => format!("({})", joined(items)),
        Value::Word { .. } => format!("\"{}\"", flat(value).unwrap_or_default()),
        Value::Drawn { text, .. } => text.split_whitespace().collect::<Vec<_>>().join(" "),
    }
}

impl Case {
    /// The values `name` has in this case: one, or one for each test.
    pub fn values_of(&self, spec: &InputSpec, name: &str) -> Vec<i128> {
        let mut env = Env::new();
        for (field, value) in spec.fields.iter().zip(&self.values) {
            bind(&field.names, value, &mut env);
        }
        let mut found: Vec<i128> = env.get(name).copied().into_iter().collect();
        if let Some(inner) = &spec.cases {
            found.extend(
                self.cases
                    .iter()
                    .flat_map(|case| case.values_of(inner, name)),
            );
        }
        found
    }

    /// A case of these values and no test cases.
    pub fn of(values: Vec<Value>) -> Case {
        Case {
            values,
            ..Case::default()
        }
    }

    /// The case as a judge would give it: a number, a row of numbers or
    /// a word per line, and under `@!multitest` the number of test cases
    /// before them.
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.values.iter().for_each(|value| write(value, &mut out));
        if !self.cases.is_empty() {
            if !self.until_eof {
                out.push_str(&format!("{}\n", self.cases.len()));
            }
            for case in &self.cases {
                out.push_str(&case.render());
            }
        }
        out
    }

    /// How much there is to read: the count of numbers, then their sum
    /// of magnitudes. A smaller case is one a person finds the bug in
    /// sooner.
    pub fn weight(&self) -> (usize, u128) {
        fn walk(value: &Value, count: &mut usize, sum: &mut u128) {
            match value {
                Value::Int(v) | Value::Char(v) | Value::Word { at: v, .. } => {
                    *count += 1;
                    *sum = sum.saturating_add(v.unsigned_abs());
                }
                Value::Seq(items) | Value::Tuple(items) => {
                    items.iter().for_each(|item| walk(item, count, sum))
                }
                Value::Drawn { text, .. } => {
                    for token in text.split_whitespace() {
                        *count += 1;
                        let size = token
                            .parse::<i128>()
                            .map_or(token.len() as u128, i128::unsigned_abs);
                        *sum = sum.saturating_add(size);
                    }
                }
            }
        }
        let (mut count, mut sum) = (0, 0);
        self.values
            .iter()
            .for_each(|value| walk(value, &mut count, &mut sum));
        for case in &self.cases {
            let (more, total) = case.weight();
            count += more;
            sum = sum.saturating_add(total);
        }
        (count, sum)
    }

    /// `n = 3  a = [1, 5, 2]`: the case by the names the program gives it.
    pub fn describe(&self, spec: &InputSpec) -> String {
        let mut parts = Vec::new();
        for (index, (field, value)) in spec.fields.iter().zip(&self.values).enumerate() {
            match (value, field.names.as_slice()) {
                (Value::Tuple(items), names) if names.len() == items.len() => {
                    for (name, item) in names.iter().zip(items) {
                        let name = name.clone().unwrap_or(format!("#{}", index + 1));
                        parts.push(format!("{name} = {}", show(item)));
                    }
                }
                (value, names) => {
                    let name = match names {
                        [Some(name)] => name.clone(),
                        _ => format!("line {}", field.line),
                    };
                    parts.push(format!("{name} = {}", show(value)));
                }
            }
        }
        if let Some(inner) = &spec.cases {
            for (index, case) in self.cases.iter().enumerate() {
                let described = case.describe(inner);
                parts.push(match self.cases.len() {
                    1 => described,
                    _ => format!("[test {}] {described}", index + 1),
                });
            }
        }
        parts.join("  ")
    }
}
