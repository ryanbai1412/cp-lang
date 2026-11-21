//! Offline segment tree over time (divide-and-conquer on life intervals).
//!
//! Turn any rollback-capable data structure (e.g. `Dsu` with `.mark()` / `.undo_to(snap)`)
//! into a dynamic structure with insertions and deletions over `[0, q)` in O(q log q)
//! apply/undo steps.
use std.dsu.{Dsu, Mark}

/// One step in a `TimeTree` DFS traversal:
/// - `Apply(id)`: apply event `tt.event(id)` on entry to a segment-tree node.
/// - `Query(t)`: answer time slot `t` at a leaf.
/// - `Undo`: revert the most recently applied event in LIFO order on exit.
pub enum TimeStep:
    Apply(id: Int)
    Query(t: Int)
    Undo

/// Offline segment tree over time `[0, q)` holding events of type `E` active on half-open
/// intervals `[lo, hi)`.
pub struct TimeTree<E>:
    q: Int
    size: Int
    head: List<-1..=2_000_000_000>
    next: List<-1..=2_000_000_000>
    ev_val: List<E>

    /// Create a timeline for `q` time slots `0..q`. O(q).
    pub fn new(q: Int) -> TimeTree<E> = todo

    fn link(mut self, u: Int, ev: E) = todo

    /// Schedule event `ev` to be active across the half-open time interval `[lo, hi)`. O(log q).
    pub fn add_interval(mut self, lo: Int, hi: Int, ev: E) = todo

    /// Retrieve the event referenced by `Apply(id)`. O(1).
    pub fn event(self, id: Int) -> E = todo

    fn emit_dfs(self, u: Int, nl: Int, nr: Int, mut out: List<TimeStep>) = todo

    /// Return the full DFS traversal of `Apply`, `Query`, and `Undo` steps over `0..q`.
    /// Total length is O(q + m log q) for `m` added intervals.
    pub fn steps(self) -> List<TimeStep> = todo

    fn run_node<M, R>(
        self,
        u: Int,
        nl: Int,
        nr: Int,
        mark: () -> M,
        apply: E -> (),
        answer: Int -> R,
        undo: M -> (),
        mut out: List<R>,
    ) = todo

    /// Walk the segment tree over `0..q` with zero step-list allocation, returning
    /// `[answer(0), ..., answer(q - 1)]`. Calls `mark()` once on entering each non-empty
    /// node, `apply(ev)` for each event active on that node, `answer(t)` at every leaf
    /// `t in 0..q` in ascending order, and `undo(snap)` on exit. O(q + m log q).
    pub fn run<M, R>(
        self,
        mark: () -> M,
        apply: E -> (),
        answer: Int -> R,
        undo: M -> (),
    ) -> List<R> = todo

    /// O(log q). Schedule event `ev` to be active across the half-open time range `r`.
    pub fn add_range(mut self, r: Range<Int>, ev: E) = todo

    /// O(q + m log q). Zero-allocation generator yielding `Apply`, `Query`, and `Undo` steps
    /// in DFS order across `0..q` (generator counterpart to `steps`).
    pub fn walk(self) -> yields TimeStep = todo

/// O(n log n) calls to `on_cross`. General CDQ divide-and-conquer driver over an ordered
/// sequence of events: recursively solves left and right halves, then invokes `on_cross(left, right)`
/// for contributions from the left half onto the right half.
pub fn cdq<E>(events: List<E>, on_cross: (List<E>, List<E>) -> ()) = todo

/// Offline divide-and-conquer over time `0..q` for `items` active on half-open
/// intervals `by(ev) = (lo, hi)`. Builds a `TimeTree<E>`, walks it with
/// `mark`/`apply`/`answer`/`undo` callbacks (zero step-list allocation), and returns
/// `[answer(0), ..., answer(q - 1)]`. O(q + m log q).
pub fn offline_over<E, M, R>(
    q: Int,
    items: List<E>,
    by: E -> (Int, Int),
    mark: () -> M,
    apply: E -> (),
    answer: Int -> R,
    undo: M -> (),
) -> List<R> = todo

@test
fn timetree_dynamic_connectivity_with_rollback_dsu() = todo

@test
fn offline_over_matches_timetree() = todo
