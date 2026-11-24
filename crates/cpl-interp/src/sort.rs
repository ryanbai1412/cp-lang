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
/// by insertion, then halves merged right half first with the left half moved
/// to a buffer of `n / 2`, as `cpl::merge_sort` does. One `dyn` comparator
/// serves every sort, so cpl.wasm carries one merge sort.
pub(crate) fn stable_order(
    n: usize,
    less: &mut dyn FnMut(usize, usize) -> Res<bool>,
) -> Res<Vec<u32>> {
    fn sort(
        xs: &mut [u32],
        buf: &mut [u32],
        less: &mut dyn FnMut(usize, usize) -> Res<bool>,
    ) -> Res<()> {
        let n = xs.len();
        if n <= SORT_RUN {
            for i in 1..n {
                for j in (1..=i).rev() {
                    if !less(xs[j] as usize, xs[j - 1] as usize)? {
                        break;
                    }
                    xs.swap(j, j - 1);
                }
            }
            return Ok(());
        }
        let m = n / 2;
        sort(&mut xs[m..], buf, less)?;
        sort(&mut xs[..m], buf, less)?;
        buf[..m].copy_from_slice(&xs[..m]);
        let (mut i, mut j, mut k) = (0, m, 0);
        while i < m && j < n {
            // One `<` per step, as in the C++: the right goes first when smaller.
            if less(xs[j] as usize, buf[i] as usize)? {
                xs[k] = xs[j];
                j += 1;
            } else {
                xs[k] = buf[i];
                i += 1;
            }
            k += 1;
        }
        xs[k..k + m - i].copy_from_slice(&buf[i..m]);
        Ok(())
    }
    let mut order: Vec<u32> = (0..n as u32).collect();
    let mut buf = vec![0; n / 2];
    sort(&mut order, &mut buf, less)?;
    Ok(order)
}

/// Rearranges `order.len()` cells in place, through `swap`, so that the cell
/// `order[i]` comes `i`-th: each cycle of the permutation is walked once and
/// marked done in `order`, so no second copy of the cells is made.
pub(crate) fn apply_order(order: &mut [u32], swap: &mut dyn FnMut(usize, usize)) {
    const DONE: u32 = u32::MAX;
    for s in 0..order.len() {
        let mut cur = s;
        while order[cur] != DONE {
            let next = order[cur] as usize;
            order[cur] = DONE;
            if next == s {
                break;
            }
            swap(cur, next);
            cur = next;
        }
    }
}

/// Sorts `xs` by `Value`'s order through `stable_order`, so cpl.wasm carries no
/// second sort of `Value`.
pub(crate) fn sort_values(xs: &mut Vec<Value>) {
    let Ok(order) = stable_order(xs.len(), &mut |a, b| Ok(xs[a] < xs[b])) else {
        return;
    };
    *xs = permuted(std::mem::take(xs), order);
}

/// `xs` rearranged in place so that `order[i]` comes `i`-th.
pub(crate) fn permuted(mut xs: Vec<Value>, mut order: Vec<u32>) -> Vec<Value> {
    apply_order(&mut order, &mut |a, b| xs.swap(a, b));
    xs
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
        Ok(permuted(xs, order))
    }

    /// `0..n` in the stable order of the keys `key(i)` of type `key_ty` (`desc`:
    /// larger first), compared as `<` on that type, user `Ord` included.
    pub(crate) fn keyed_order(
        &mut self,
        n: usize,
        key: &dyn Fn(usize) -> Value,
        key_ty: &Ty,
        desc: bool,
        span: Span,
    ) -> Res<Vec<u32>> {
        stable_order(n, &mut |a, b| {
            let (a, b) = if desc { (b, a) } else { (a, b) };
            self.less_ty_val(key_ty, &key(a), &key(b), span)
        })
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
        Ok(permuted(xs, order))
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
