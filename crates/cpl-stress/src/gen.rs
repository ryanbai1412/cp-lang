//! Cases: the values of one input, made from an [`InputSpec`].

use crate::rng::Rng;
use crate::spec::{eval, Env, InputSpec, Shape};

/// One value of an input.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Int(i128),
    Seq(Vec<Value>),
    Tuple(Vec<Value>),
}

/// One input: a value per `read()`, in order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Case {
    pub values: Vec<Value>,
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

fn build(shape: &Shape, env: &Env, is_size: bool, pick: &mut dyn Pick) -> Option<Value> {
    match shape {
        Shape::Int { lo, hi } => {
            let (lo, hi) = (eval(lo, env)?, eval(hi, env)?);
            (lo <= hi).then(|| Value::Int(pick.pick(lo, hi, is_size)))
        }
        Shape::Array { len, elem } => {
            let len = eval(len, env)?.max(0);
            (len <= MAX_LEN).then_some(())?;
            (0..len)
                .map(|_| build(elem, env, false, pick))
                .collect::<Option<_>>()
                .map(Value::Seq)
        }
        Shape::Tuple(parts) => parts
            .iter()
            .map(|part| build(part, env, false, pick))
            .collect::<Option<_>>()
            .map(Value::Tuple),
    }
}

/// The integers of `value`, in the order a pattern names them.
fn leaves(value: &Value, out: &mut Vec<Option<i128>>) {
    match value {
        Value::Int(v) => out.push(Some(*v)),
        Value::Tuple(parts) => parts.iter().for_each(|part| leaves(part, out)),
        Value::Seq(_) => out.push(None),
    }
}

/// Adds the names a field binds to `env`.
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
    let mut env: Env = spec.consts.clone();
    let mut values = Vec::new();
    for field in &spec.fields {
        let value = match (&field.shape, field.names.as_slice()) {
            // `let (n, q): (.., ..) = read()`: each part can be a size.
            (Shape::Tuple(parts), names) if names.len() == parts.len() => {
                let mut built = Vec::new();
                for (part, name) in parts.iter().zip(names) {
                    let is_size = name.as_ref().is_some_and(|n| spec.sizes.contains(n));
                    let value = build(part, &env, is_size, pick)?;
                    bind(std::slice::from_ref(name), &value, &mut env);
                    built.push(value);
                }
                Value::Tuple(built)
            }
            (shape, names) => {
                let is_size = matches!(names, [Some(name)] if spec.sizes.contains(name));
                build(shape, &env, is_size, pick)?
            }
        };
        bind(&field.names, &value, &mut env);
        values.push(value);
    }
    Some(Case { values })
}

/// A random case. `None` when a bound cannot be evaluated or a range is
/// empty (`1..=n - 1` with `n = 1`).
pub fn generate(spec: &InputSpec, rng: &mut Rng, size: Size) -> Option<Case> {
    make(spec, &mut Random { rng, size })
}

/// One line when `value` is a number or a row of numbers.
fn flat(value: &Value) -> Option<String> {
    let number = |value: &Value| match value {
        Value::Int(v) => Some(v.to_string()),
        _ => None,
    };
    match value {
        Value::Int(v) => Some(v.to_string()),
        Value::Seq(items) | Value::Tuple(items) => items
            .iter()
            .map(number)
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join(" ")),
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
        (None, Value::Int(_)) => unreachable!("a number is always flat"),
    }
}

impl Case {
    /// The case as a judge would give it: a number or a row of numbers
    /// per line.
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.values.iter().for_each(|value| write(value, &mut out));
        out
    }

    /// How much there is to read: the count of numbers, then their sum
    /// of magnitudes. A smaller case is one a person finds the bug in
    /// sooner.
    pub fn weight(&self) -> (usize, u128) {
        fn walk(value: &Value, count: &mut usize, sum: &mut u128) {
            match value {
                Value::Int(v) => {
                    *count += 1;
                    *sum = sum.saturating_add(v.unsigned_abs());
                }
                Value::Seq(items) | Value::Tuple(items) => {
                    items.iter().for_each(|item| walk(item, count, sum))
                }
            }
        }
        let (mut count, mut sum) = (0, 0);
        self.values
            .iter()
            .for_each(|value| walk(value, &mut count, &mut sum));
        (count, sum)
    }

    /// `n = 3  a = [1, 5, 2]`: the case by the names the program gives it.
    pub fn describe(&self, spec: &InputSpec) -> String {
        fn show(value: &Value) -> String {
            match value {
                Value::Int(v) => v.to_string(),
                Value::Seq(items) => {
                    format!(
                        "[{}]",
                        items.iter().map(show).collect::<Vec<_>>().join(", ")
                    )
                }
                Value::Tuple(items) => {
                    format!(
                        "({})",
                        items.iter().map(show).collect::<Vec<_>>().join(", ")
                    )
                }
            }
        }
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
        parts.join("  ")
    }
}
