//! Reads whose branch a tag read above them chooses.

use super::*;

impl Builder<'_> {
    /// An `if` or `match` on a tag read above it, each branch reading
    /// its own values: `if op == 1: let (l, r) = read() else: let i = read()`.
    /// The tag is a number read alone or first in a tuple; what is read
    /// between it and the branch is read by every branch, before its own.
    pub(super) fn choice(
        &mut self,
        expr: &Expr,
        inside: &mut HashSet<String>,
        parts: &mut Vec<Shape>,
        binds: &mut Vec<Vec<Option<String>>>,
    ) -> Result<bool, SpecError> {
        let scalar = |shape: &Shape| {
            matches!(
                shape,
                Shape::Int { .. } | Shape::Char { .. } | Shape::Bool | Shape::Str { .. }
            )
        };
        for at in (0..parts.len()).rev() {
            let Some(Some(name)) = binds[at].first().cloned() else {
                continue;
            };
            let (tag, rest) = match &parts[at] {
                shape if scalar(shape) && binds[at].len() == 1 => (shape.clone(), None),
                Shape::Tuple {
                    parts: inner,
                    binds: inner_binds,
                } if inner.len() > 1 && inner.len() == binds[at].len() && scalar(&inner[0]) => {
                    let rest = Shape::Tuple {
                        parts: inner[1..].to_vec(),
                        binds: inner_binds.get(1..).unwrap_or_default().to_vec(),
                    };
                    (inner[0].clone(), Some((rest, binds[at][1..].to_vec())))
                }
                _ => continue,
            };
            let rebound = binds[at][1..]
                .iter()
                .chain(binds[at + 1..].iter().flatten())
                .any(|n| n.as_deref() == Some(name.as_str()));
            if rebound {
                return Ok(false);
            }
            let Some((tag, mut arms)) = self.arms_on(expr, inside, &name, tag)? else {
                continue;
            };
            let mut between: Vec<(Shape, Vec<Option<String>>)> = rest.into_iter().collect();
            between.extend(parts.drain(at + 1..).zip(binds.drain(at + 1..)));
            parts.pop();
            binds.pop();
            if !between.is_empty() && arms.iter().all(|arm| arm.tags.is_some()) {
                arms.push(Arm {
                    tags: None,
                    parts: Vec::new(),
                    binds: Vec::new(),
                });
            }
            for arm in &mut arms {
                let (shapes, names): (Vec<_>, Vec<_>) = between.iter().cloned().unzip();
                arm.parts.splice(0..0, shapes);
                arm.binds.splice(0..0, names);
            }
            inside.insert(name.clone());
            parts.push(Shape::Choice {
                tag: Box::new(tag),
                name,
                arms,
            });
            binds.push(Vec::new());
            return Ok(true);
        }
        Ok(false)
    }

    /// The top-level fields from a tag on, made one choice by the branch
    /// `expr` on that tag; false when `expr` is not one.
    pub(super) fn top_choice(&mut self, expr: &Expr) -> Result<bool, SpecError> {
        if !matches!(expr.kind, ExprKind::If { .. } | ExprKind::Match { .. }) {
            return Ok(false);
        }
        // A tag named twice is ambiguous: which one the branch tests
        // depends on scope the fields no longer carry.
        let mut named = HashSet::new();
        let names = self
            .spec
            .fields
            .iter()
            .flat_map(|field| field.names.iter().flatten());
        if !names.into_iter().all(|name| named.insert(name)) {
            return Ok(false);
        }
        let fields = std::mem::take(&mut self.spec.fields);
        let (mut parts, mut binds): (Vec<_>, Vec<_>) = fields
            .iter()
            .map(|field| (field.shape.clone(), field.names.clone()))
            .unzip();
        let mut inside = HashSet::new();
        let chose = self.choice(expr, &mut inside, &mut parts, &mut binds);
        match chose {
            Ok(true) => {
                self.spec.fields = fields;
                let Some(shape) = parts.pop() else {
                    return Ok(false);
                };
                let kept = parts.len();
                let line = self.spec.fields[kept].line;
                self.spec.fields.truncate(kept);
                self.spec.fields.push(Field {
                    names: Vec::new(),
                    shape,
                    line,
                });
                Ok(true)
            }
            other => {
                self.spec.fields = fields;
                other
            }
        }
    }

    /// The arms of `expr` as a choice on the tag `name`, and the tag's
    /// shape; `None` when `expr` is not a choice on it.
    pub(super) fn arms_on(
        &mut self,
        expr: &Expr,
        inside: &HashSet<String>,
        name: &str,
        tag: Shape,
    ) -> Result<Option<(Shape, Vec<Arm>)>, SpecError> {
        let consts = self.spec.consts.clone();
        // A `Str` tag is compared with words: each is its position in
        // the order the arms name them.
        let words = std::cell::RefCell::new(Vec::<String>::new());
        let word = matches!(tag, Shape::Str { .. });
        let literal = |expr: &Expr| match &expr.kind {
            ExprKind::Str(pieces) if word => {
                let [StrPiece::Text(text)] = pieces.as_slice() else {
                    return None;
                };
                let mut words = words.borrow_mut();
                let at = words.iter().position(|w| w == text).unwrap_or_else(|| {
                    words.push(text.clone());
                    words.len() - 1
                });
                Some(at as i128)
            }
            _ if word => None,
            _ => eval(expr, &consts),
        };
        // The tags `x == 1 or x == 2` takes, with `x` the tag.
        fn tags(
            cond: &Expr,
            name: &str,
            literal: &dyn Fn(&Expr) -> Option<i128>,
        ) -> Option<(Vec<i128>, bool)> {
            match &cond.kind {
                ExprKind::Paren(inner) => tags(inner, name, literal),
                ExprKind::Binary {
                    op: BinOp::Or,
                    lhs,
                    rhs,
                    ..
                } => {
                    let (mut a, false) = tags(lhs, name, literal)? else {
                        return None;
                    };
                    let (b, false) = tags(rhs, name, literal)? else {
                        return None;
                    };
                    a.extend(b);
                    Some((a, false))
                }
                ExprKind::Compare { first, rest } => {
                    let [(op @ (CmpOp::Eq | CmpOp::Ne), other)] = rest.as_slice() else {
                        return None;
                    };
                    let is_tag = |e: &Expr| matches!(&e.kind, ExprKind::Name(n) if n == name);
                    let value = match (is_tag(first), is_tag(other)) {
                        (true, false) => literal(other)?,
                        (false, true) => literal(first)?,
                        _ => return None,
                    };
                    Some((vec![value], *op == CmpOp::Ne))
                }
                _ => None,
            }
        }
        let mut ways: Vec<(Option<Vec<i128>>, Option<&Block>)> = Vec::new();
        match &expr.kind {
            ExprKind::If {
                branches,
                otherwise,
            } => {
                for (at, (cond, block)) in branches.iter().enumerate() {
                    let Cond::Expr(cond) = cond else {
                        return Ok(None);
                    };
                    match tags(cond, name, &literal) {
                        Some((values, false)) => ways.push((Some(values), Some(block))),
                        // `if x != 1: A else: B` is `B` for 1, else `A`.
                        Some((values, true)) if at == 0 && branches.len() == 1 => {
                            ways.push((Some(values), otherwise.as_ref()));
                            ways.push((None, Some(block)));
                        }
                        _ => return Ok(None),
                    }
                }
                if !matches!(ways.last(), Some((None, _))) {
                    ways.push((None, otherwise.as_ref()));
                }
            }
            ExprKind::Match { scrutinee, arms } if matches!(&scrutinee.kind, ExprKind::Name(n) if n == name) =>
            {
                fn values(
                    pat: &Pattern,
                    literal: &dyn Fn(&Expr) -> Option<i128>,
                ) -> Option<Option<Vec<i128>>> {
                    match &pat.kind {
                        PatternKind::Wildcard | PatternKind::Name(_) => Some(None),
                        PatternKind::Literal(expr) => Some(Some(vec![literal(expr)?])),
                        PatternKind::Or(pats) => {
                            let mut all = Vec::new();
                            for pat in pats {
                                all.extend(values(pat, literal)??);
                            }
                            Some(Some(all))
                        }
                        _ => None,
                    }
                }
                for arm in arms {
                    if arm.guard.is_some() {
                        return Ok(None);
                    }
                    let Some(tags) = values(&arm.pat, &literal) else {
                        return Ok(None);
                    };
                    ways.push((tags, Some(&arm.body)));
                }
            }
            _ => return Ok(None),
        }
        let mut arms = Vec::new();
        for (tags, block) in ways {
            let mut arm = Arm {
                tags,
                parts: Vec::new(),
                binds: Vec::new(),
            };
            if let Some(block) = block {
                let mut inside = inside.clone();
                if !self.rows(&block.stmts, &mut inside, &mut arm.parts, &mut arm.binds)? {
                    return Ok(None);
                }
            }
            arms.push(arm);
        }
        // An `Int` tag read with no range takes the values its arms test,
        // and one past them for an arm that takes the rest: the statement
        // numbers its queries, and a far-off tag only reads as noise.
        let tag = match tag {
            Shape::Int { lo, hi }
                if eval(&lo, &consts) == Some(DEFAULT_INT.0)
                    && eval(&hi, &consts) == Some(DEFAULT_INT.1) =>
            {
                let named: Vec<i128> = arms
                    .iter()
                    .flat_map(|arm| arm.tags.iter().flatten().copied())
                    .collect();
                let rest = arms.iter().any(|arm| arm.tags.is_none());
                match (named.iter().min(), named.iter().max()) {
                    (Some(&min), Some(&max)) => Shape::Int {
                        lo: int(min),
                        hi: int(max + i128::from(rest)),
                    },
                    _ => Shape::Int { lo, hi },
                }
            }
            Shape::Str { .. } => {
                let mut words = words.into_inner();
                if words.is_empty()
                    || words
                        .iter()
                        .any(|w| w.is_empty() || w.contains(char::is_whitespace))
                {
                    return Ok(None);
                }
                // One more word for an arm that takes every other word.
                if arms.iter().any(|arm| arm.tags.is_none()) {
                    let mut other = String::from("x");
                    while words.contains(&other) {
                        other.push('x');
                    }
                    words.push(other);
                }
                Shape::Words(words.into())
            }
            tag => tag,
        };
        Ok(Some((tag, arms)))
    }
}
