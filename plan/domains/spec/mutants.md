# Mutation score

How many one-line compiler bugs the conformance suite catches
([conformance/R9](../../../handbook/guidelines/conformance-checklist.md)).
`ci-slow.yml` reruns 12 fresh mutants on every push to main, without blocking
(artifact `mutation-score`). This file records the first full run and what
came of each mutant it planted.

## Run 1: seed 1, 20 mutants, at `d711d1a27`

5 of 20 caught (25%) at that commit. After triage:

| state | mutants |
|---|---|
| caught at `d711d1a27` | 5 |
| caught by a case added since | 3 |
| case landed, passes once the interpreter's trap code is fixed | 1 |
| equivalent: no checked program behaves differently | 6 |
| open: asked compiler for a program or an equivalence proof | 5 |

| site | state | evidence |
|---|---|---|
| `cpl-syntax/src/parser/stmt.rs:1659` | caught since | `core-01/binder-after-comma-needs-let` (`d338798b4`, line fixed in `0e7e4375a`); the mutant gives E0103 |
| `cpl-check/src/check/expr.rs:127` | caught since | `core-07/in-on-a-list-fixes-to-contains` (`48b54102c`): only a lone `x in xs` gets the `xs.contains(x)` fix |
| `cpl-interp/src/vm/compile/mod.rs:966` | caught since | `core-04/trap-in-a-small-fn-names-its-frame` (`103ed37e6`): the mutant inlines `f` and drops its call-stack note. Writing it showed that trap cases ignored `// has:`, which that commit fixes |
| `cpl-interp/src/mutation.rs:125` | case landed | `core-07/swap-with-one-index-past-the-end-traps` (`5402b3000`) wants R0101; the interpreter says R0001 today |
| `cpl-codegen/src/expr.rs:97` | equivalent (unproven) | `narrow: true` on a cast whose operand is already wide emits the same C++ |
| `cpl-interp/src/lib.rs:1750` | equivalent | a tree-walker fast path; the general `resolve` arm gives the same cell (compiler built it and compared stdout and traces) |
| `cpl-ir/src/lib.rs:382` | equivalent | the checker gives both sides one type before lowering, so the arm sees no differing shapes (compiler built both variants and compared them) |
| `cpl-interp/src/vm/compile/stmt.rs:995` | equivalent | `==` only turns the fed-loop fast path off; the slow path prints the same |
| `cpl-interp/src/vm/compile/call_mix.rs:72` | equivalent | `false` only sends a list cell's `mut` argument down the slow path; `core-05/mut-cell-of-2d-array-as-argument` (`e107dbf2b`) covers the 2-D side |
| `cpl-codegen/src/ranges.rs:520` | equivalent (unproven) | `depth - 1` only cuts the proof's recursion short, which keeps a check the proof would drop; output is the same |
| `cpl-check/src/lends.rs:1875` (the hold for a named view) | caught | 3 cases; compiler finds the flag otherwise unobservable and logged removing it |
| `cpl-check/src/check/expr.rs:3389` (Tuple arm of `fits`) | open | checking against the named-record cases (`be27b372a`) |
| `cpl-interp/src/vm/exec/calls.rs:508` | open | asked compiler |
| `cpl-interp/src/vm/compile/expr_i.rs:479` | open | asked compiler |
| `cpl-codegen/src/stmt.rs:493` | open | asked compiler |
| `cpl-ir/src/lower/expr.rs:3655` | open | asked compiler |

### The generated report

| site | was | mutant | result | cases that fail |
|---|---|---|---|---|
| `crates/cpl-check/src/check/lends.rs:1875` | `is_capture: false,` | `is_capture: true,` | caught | 3 |
| `crates/cpl-codegen/src/expr.rs:97` | `narrow: false,` | `narrow: true,` | missed | 0 |
| `crates/cpl-syntax/src/parser/stmt.rs:1659` | `return i + 1;` | `return i - 1;` | missed | 0 |
| `crates/cpl-ir/src/lower/expr.rs:424` | `if indices.len() == 1 && Self::keyed_index(&base_expr.ty) {` | `if indices.len() != 1 && Self::keyed_index(&base_expr.ty) {` | caught | 29 |
| `crates/cpl-check/src/check/expr.rs:3389` | `(Ty::Tuple(a), Ty::Tuple(b)) if a.len() == b.len() => {` | `(Ty::Tuple(a), Ty::Tuple(b)) if a.len() != b.len() => {` | missed | 0 |
| `crates/cpl-interp/src/vm/exec/calls.rs:508` | `if j < 0 \|\| j >= d1 {` | `if j <= 0 \|\| j >= d1 {` | missed | 0 |
| `crates/cpl-interp/src/vm/compile/expr_i.rs:479` | `if self.in_dom.iter().any(\|f\| (f.0, f.1) == (src, dom)) {` | `if self.in_dom.iter().any(\|f\| (f.0, f.1) != (src, dom)) {` | missed | 0 |
| `crates/cpl-interp/src/vm/compile/mod.rs:966` | `&& simple_expr(e, f, program)` | `\|\| simple_expr(e, f, program)` | missed | 0 |
| `crates/cpl-codegen/src/stmt.rs:493` | `hi_code.at(REL + 1)` | `hi_code.at(REL - 1)` | missed | 0 |
| `crates/cpl-interp/src/lib.rs:1750` | `Value::Array(array) if array.dims.len() == 2 && n == 2 => {` | `Value::Array(array) if array.dims.len() != 2 && n == 2 => {` | missed | 0 |
| `crates/cpl-ir/src/lib.rs:382` | `(Ty::Tuple(x), Ty::Tuple(y)) => x.len() == y.len() && x != y,` | `(Ty::Tuple(x), Ty::Tuple(y)) => x.len() != y.len() && x != y,` | missed | 0 |
| `crates/cpl-check/src/check/expr.rs:127` | `self.list_in(*op, left_expr, right_expr, &right, rest.len() == 1);` | `self.list_in(*op, left_expr, right_expr, &right, rest.len() != 1);` | missed | 0 |
| `crates/cpl-interp/src/vm/compile/stmt.rs:995` | `\|\| args.len() != f.params.len()` | `\|\| args.len() == f.params.len()` | missed | 0 |
| `crates/cpl-syntax/src/parser/expr.rs:729` | `let name = if self.at(T::Ident) && self.peek_n(1) == T::Eq {` | `let name = if self.at(T::Ident) \|\| self.peek_n(1) == T::Eq {` | caught | 2638 |
| `crates/cpl-interp/src/mutation.rs:125` | `(Some(i), Some(j)) if i < len && j < len => v.swap(i, j),` | `(Some(i), Some(j)) if i < len \|\| j < len => v.swap(i, j),` | missed | 0 |
| `crates/cpl-interp/src/vm/compile/assign.rs:135` | `&& (indices.len() == 1 \|\| indices.len() == 2) =>` | `&& (indices.len() != 1 \|\| indices.len() == 2) =>` | caught | 3 |
| `crates/cpl-codegen/src/ranges.rs:520` | `.is_some_and(\|item\| self.below_max_at(home, item, depth + 1)))` | `.is_some_and(\|item\| self.below_max_at(home, item, depth - 1)))` | missed | 0 |
| `crates/cpl-syntax/src/items.rs:60` | `&& tokens[i - 1].kind == T::Dedent` | `\|\| tokens[i - 1].kind == T::Dedent` | caught | 2869 |
| `crates/cpl-interp/src/vm/compile/call_mix.rs:72` | `Ty::List(_) => true,` | `Ty::List(_) => false,` | missed | 0 |
| `crates/cpl-ir/src/lower/expr.rs:3655` | `closed: false,` | `closed: true,` | missed | 0 |
