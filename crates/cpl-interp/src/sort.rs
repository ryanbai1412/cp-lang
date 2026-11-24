//! The interpreter's sorts: a stable merge sort whose runs of at most
//! `SORT_RUN` insertion-sort first, making the comparisons the C++
//! `cpl::merge_sort` makes, so `cpl profile` counts agree natively.

use super::*;

/// The longest run a sort orders by insertion before it merges.
const SORT_RUN: usize = 7;

/// `cells` sorted, when they are packed scalars whose type orders them as numbers: no
/// cell is boxed and equal cells cannot be told apart, so the order is the stable sort's.
pub(crate) fn sorted_packed(cells: &value::Cells, ty: &Ty, desc: bool) -> Option<value::Cells> {
    let custom = matches!(
        ty,
        Ty::Struct(_) | Ty::Tuple(_) | Ty::Record(_) | Ty::List(_) | Ty::Opt(_)
    );
    let scalar = matches!(
        cells,
        value::Cells::Int(_) | value::Cells::Bool(_) | value::Cells::Char(_)
    );
    if custom || !scalar {
        return None;
    }
    let mut sorted = cells.clone();
    sorted.sort_unstable();
    if desc {
        sorted.reverse();
    }
    Some(sorted)
}

/// `0..n` in the stable order of `less(later, earlier)`: runs of at most `SORT_RUN`
/// by insertion, then halves merged right half first, as `cpl::merge_sort` does. One
/// `dyn` comparator serves every sort, so cpl.wasm carries one merge sort.
pub(crate) fn stable_order(
    n: usize,
    less: &mut dyn FnMut(usize, usize) -> Res<bool>,
) -> Res<Vec<usize>> {
    /// Sorts into `dst`, with `src` holding the same items as scratch.
    fn sort(
        src: &mut [usize],
        dst: &mut [usize],
        less: &mut dyn FnMut(usize, usize) -> Res<bool>,
    ) -> Res<()> {
        let n = dst.len();
        if n <= SORT_RUN {
            for i in 1..n {
                for j in (1..=i).rev() {
                    if !less(dst[j], dst[j - 1])? {
                        break;
                    }
                    dst.swap(j, j - 1);
                }
            }
            return Ok(());
        }
        let m = n / 2;
        sort(&mut dst[m..], &mut src[m..], less)?;
        sort(&mut dst[..m], &mut src[..m], less)?;
        let (mut i, mut j) = (0, m);
        for slot in dst.iter_mut() {
            // One `<` per step, as in the C++: the right goes first when smaller.
            let right = i == m || (j < n && less(src[j], src[i])?);
            *slot = if right { src[j] } else { src[i] };
            if right {
                j += 1;
            } else {
                i += 1;
            }
        }
        Ok(())
    }
    let mut order: Vec<usize> = (0..n).collect();
    let mut scratch = order.clone();
    sort(&mut scratch, &mut order, less)?;
    Ok(order)
}

/// Sorts `xs` by `Value`'s order through `stable_order`, so cpl.wasm carries no
/// second sort of `Value`.
pub(crate) fn sort_values(xs: &mut Vec<Value>) {
    let Ok(order) = stable_order(xs.len(), &mut |a, b| Ok(xs[a] < xs[b])) else {
        return;
    };
    *xs = permuted(std::mem::take(xs), &order);
}

/// `xs` rearranged so that `order[i]` comes `i`-th.
fn permuted<T>(xs: Vec<T>, order: &[usize]) -> Vec<T> {
    let mut slots: Vec<Option<T>> = xs.into_iter().map(Some).collect();
    order.iter().filter_map(|&i| slots[i].take()).collect()
}

impl<'p, 'o, O: Observer> Interp<'p, 'o, O> {
    pub(crate) fn merge_sort_ty(
        &mut self,
        xs: Vec<Value>,
        ty: &Ty,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
        let order = stable_order(xs.len(), &mut |a, b| {
            let (a, b) = if desc {
                (&xs[b], &xs[a])
            } else {
                (&xs[a], &xs[b])
            };
            self.less_ty_val(ty, a, b, span)
        })?;
        Ok(permuted(xs, &order))
    }

    pub(crate) fn merge_sort_keyed(
        &mut self,
        xs: Vec<(Value, Value)>,
        key_ty: &Ty,
        desc: bool,
        span: Span,
    ) -> Res<Vec<(Value, Value)>> {
        let order = stable_order(xs.len(), &mut |a, b| {
            let (a, b) = if desc {
                (&xs[b], &xs[a])
            } else {
                (&xs[a], &xs[b])
            };
            self.less_ty_val(key_ty, &a.0, &b.0, span)
        })?;
        Ok(permuted(xs, &order))
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
                    cpl_diag::msgs::interp::SORT_CMP_COMPARATOR_NOT_STRICT.at(&[]),
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
                        cpl_diag::msgs::interp::SORT_CMP_COMPARATOR_NOT_STRICT_2.at(&[]),
                    );
                }
            }
        }
        Ok(out)
    }

    pub(crate) fn merge_sort_by(
        &mut self,
        xs: Vec<Value>,
        cmp: &Value,
        desc: bool,
        span: Span,
    ) -> Res<Vec<Value>> {
        let order = stable_order(xs.len(), &mut |a, b| {
            self.sort_by_less(&xs[a], &xs[b], cmp, desc, span)
        })?;
        Ok(permuted(xs, &order))
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
                    cpl_diag::msgs::interp::SORT_CMP_COMPARATOR_NOT_STRICT_3.at(&[]),
                );
            }
        }
        Ok(lt)
    }
}
