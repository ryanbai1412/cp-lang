//! The interpreter's sorts: a stable merge sort whose runs of at most
//! `SORT_RUN` insertion-sort first, making the comparisons the C++
//! `cpl::merge_sort` makes, so `cpl profile` counts agree natively.

use super::*;

/// The longest run a sort orders by insertion before it merges.
const SORT_RUN: usize = 7;

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    pub(crate) fn merge_sort_ty(
        &mut self,
        mut xs: Vec<Value>,
        ty: &Ty,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
        if xs.len() <= SORT_RUN {
            for i in 1..xs.len() {
                for j in (1..=i).rev() {
                    let (a, b) = [(&xs[j], &xs[j - 1]), (&xs[j - 1], &xs[j])][desc as usize];
                    if !self.less_ty_val(ty, a, b, span)? {
                        break;
                    }
                    xs.swap(j, j - 1);
                }
            }
            return Ok(xs);
        }
        let mid = xs.len() / 2;
        let right = self.merge_sort_ty(xs.split_off(mid), ty, desc, span)?;
        let left = self.merge_sort_ty(xs, ty, desc, span)?;
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            // One `<` per step, as in the C++: the right goes first when smaller (`desc`: larger).
            let (a, b) = [(&right[j], &left[i]), (&left[i], &right[j])][desc as usize];
            if self.less_ty_val(ty, a, b, span)? {
                out.push(right[j].clone());
                j += 1;
            } else {
                out.push(left[i].clone());
                i += 1;
            }
        }
        out.extend_from_slice(&left[i..]);
        out.extend_from_slice(&right[j..]);
        Ok(out)
    }

    pub(crate) fn merge_sort_keyed(
        &mut self,
        mut xs: Vec<(Value, Value)>,
        key_ty: &Ty,
        desc: bool,
        span: Span,
    ) -> Res<Vec<(Value, Value)>> {
        if xs.len() <= SORT_RUN {
            for i in 1..xs.len() {
                for j in (1..=i).rev() {
                    let (a, b) =
                        [(&xs[j].0, &xs[j - 1].0), (&xs[j - 1].0, &xs[j].0)][desc as usize];
                    if !self.less_ty_val(key_ty, a, b, span)? {
                        break;
                    }
                    xs.swap(j, j - 1);
                }
            }
            return Ok(xs);
        }
        let mid = xs.len() / 2;
        let right = self.merge_sort_keyed(xs.split_off(mid), key_ty, desc, span)?;
        let left = self.merge_sort_keyed(xs, key_ty, desc, span)?;
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            let (a, b) = [(&right[j].0, &left[i].0), (&left[i].0, &right[j].0)][desc as usize];
            if self.less_ty_val(key_ty, a, b, span)? {
                out.push(right[j].clone());
                j += 1;
            } else {
                out.push(left[i].clone());
                i += 1;
            }
        }
        out.extend_from_slice(&left[i..]);
        out.extend_from_slice(&right[j..]);
        Ok(out)
    }

    pub(crate) fn checked_merge_sort_by(
        &mut self,
        xs: Vec<Value>,
        cmp: &Value,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
        for x in &xs {
            let pair = vec![x.clone(), x.clone()];
            if self
                .quietly(|run| run.call_value(cmp.clone(), pair, span))?
                .bool()
            {
                return trap(
                    span,
                    "sort `cmp:` comparator is not a strict weak order: cmp(x, x) is true",
                );
            }
        }
        let out = self.merge_sort_by(xs, cmp, desc, span)?;
        for i in 1..out.len() {
            let checks = [
                (&out[i], &out[i - 1]),
                (&out[i], &out[0]),
                (&out[out.len() - 1], &out[i - 1]),
            ];
            for (lhs, rhs) in checks {
                let pair = if desc {
                    vec![rhs.clone(), lhs.clone()]
                } else {
                    vec![lhs.clone(), rhs.clone()]
                };
                if self
                    .quietly(|run| run.call_value(cmp.clone(), pair, span))?
                    .bool()
                {
                    return trap(
                        span,
                        "sort `cmp:` comparator is not a strict weak order: result is not ordered",
                    );
                }
            }
        }
        Ok(out)
    }

    pub(crate) fn merge_sort_by(
        &mut self,
        mut xs: Vec<Value>,
        cmp: &Value,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
        if xs.len() <= SORT_RUN {
            for i in 1..xs.len() {
                for j in (1..=i).rev() {
                    if !self.sort_by_less(&xs[j], &xs[j - 1], cmp, desc, span)? {
                        break;
                    }
                    xs.swap(j, j - 1);
                }
            }
            return Ok(xs);
        }
        let mid = xs.len() / 2;
        let right = xs.split_off(mid);
        let left = self.merge_sort_by(xs, cmp, desc, span)?;
        let right = self.merge_sort_by(right, cmp, desc, span)?;
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            if self.sort_by_less(&right[j], &left[i], cmp, desc, span)? {
                out.push(right[j].clone());
                j += 1;
            } else {
                out.push(left[i].clone());
                i += 1;
            }
        }
        out.extend_from_slice(&left[i..]);
        out.extend_from_slice(&right[j..]);
        Ok(out)
    }

    /// Whether `later` goes before `earlier` under a sort's `cmp:` (`desc`: after),
    /// trapping when `cmp` holds both ways.
    fn sort_by_less(
        &mut self,
        later: &Value,
        earlier: &Value,
        cmp: &Value,
        desc: bool,
        span: Span,
    ) -> Res<bool> {
        let (first, second) = if desc {
            (earlier, later)
        } else {
            (later, earlier)
        };
        let lt = self
            .call_value(cmp.clone(), vec![first.clone(), second.clone()], span)?
            .bool();
        if lt {
            let pair = vec![second.clone(), first.clone()];
            if self
                .quietly(|run| run.call_value(cmp.clone(), pair, span))?
                .bool()
            {
                return trap(
                    span,
                    "sort `cmp:` comparator is not a strict weak order: both cmp(a, b) and cmp(b, a) are true",
                );
            }
        }
        Ok(lt)
    }
}
