//! `cpl fix`, the one fix path behind the command, the editor's fix-all
//! and the playground: each error's checked fix, round after round, then,
//! once the file checks clean, the lint fixes and codemods.

use cpl_check::Session;
use cpl_diag::Diagnostic;

/// `text` fixed in `db` under `key`: the fixed text, how many fixes were
/// applied, and the errors left.
pub fn document(
    db: &mut Session,
    key: &str,
    text: &str,
    config: &cpl_lint::Config,
) -> (String, usize, Vec<Diagnostic>) {
    let (fixed, applied, left) = cpl_ir::fix::apply_error_fixes_in(db, key, text);
    if !left.is_empty() {
        return (fixed, applied, left);
    }
    let (linted, n) = cpl_lint::fix_in(db, key, &fixed, config);
    (linted, applied + n, left)
}
