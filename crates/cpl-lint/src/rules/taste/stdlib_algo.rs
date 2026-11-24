//! The plain one-index read the taste rules take apart.

use super::*;

/// Unwraps a 1-D index `arr[ix]` and returns `(arr_name, ix)`.
pub(super) fn plain_index(e: &Expr) -> Option<(&str, &Expr)> {
    let ExprKind::Index { base, indices } = &walk::unparen(e).kind else {
        return None;
    };
    let [ix] = indices.as_slice() else {
        return None;
    };
    Some((shapes::name(base)?, ix))
}
