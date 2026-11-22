//! Sliding-window folds and stateful windows (round 7, `lib/sequence-patterns.md`).
//!
//! `a.windows(k).fold(op)` and `Window(op)` pick their strategy from the op's laws:
//! a `Group` adds and subtracts, a `Selective` op keeps a monotone deque, anything else uses
//! two stacks. State that is not a fold implements `Track` (`push`/`pop`); two pointers, Mo
//! and divide-and-conquer DP all accept a `Track`.
use std.algebra.{Op}
use std.dsu.{Mark}

/// A sliding fold queue over `op`. `push` and `pop` are O(1) amortized; `all` is O(1).
pub struct Window<T, M>(op: M):
    q: List<T>
    front: List<T>
    back: ?T

    /// An empty window over `op`. The strategy comes from `op`'s laws: a `Group` keeps a running
    /// total and subtracts on `pop`, a `Selective` op (`min`, `max`, `min(by:)`) keeps a monotone
    /// deque, and anything else keeps two stacks.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var w = Window(+); w.all == None`
    pub fn new(op: M) -> Window<T, M>:
        Window(op, [], [], None)

    /// Push `x` at the back, as the newest element.
    ///
    /// cost: O(1) amortized calls of `op`
    /// traps: never (beyond what `op` traps on)
    /// example: `var w = Window(max); w.push(3); w.push(5); w.all == Some(5)`
    pub fn push(mut self, x: T):
        self.push_back(x)

    /// Pop the front: remove the oldest element.
    ///
    /// cost: O(1) amortized calls of `op`
    /// traps: on an empty window
    /// example: `var w = Window(min); w.push(3); w.push(1); w.push(2); w.pop(); w.all == Some(1)`
    /// example: `var w = Window(min); w.push(1); w.push(2); w.pop(); w.all == Some(2)`
    pub fn pop(mut self):
        self.drop_front()

    /// The fold of the current contents, oldest first (`op(op(x1, x2), x3)` for pushes `x1`, `x2`,
    /// `x3`), so a non-commutative op folds in push order; `None` when empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `var w = Window(+); w.push(2); w.push(3); w.all == Some(5)`
    /// example: `var w = Window(+); w.push(2); w.pop(); w.all == None`
    @get pub fn all(self) -> ?T:
        if front.len == 0: return back
        let f = front[front.len - 1]
        if let b = back: return Some(op(f, b))
        Some(f)

    /// The number of elements in the window: pushes minus pops.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var w = Window(+); w.push(2); w.push(3); w.pop(); w.len == 1`
    @get pub fn len(self) -> Int:
        front.len + q.len

    fn push_back(mut self, x: T):
        q.push(x)
        if let b = back: back = Some(op(b, x))
        else: back = Some(x)

    fn drop_front(mut self):
        if front.len == 0:
            assert q.len > 0, "`pop` on an empty window"
            var k = q.len - 1
            var acc = q[k]
            front.push(acc)
            while k > 0:
                k -= 1
                acc = op(q[k], acc)
                front.push(acc)
            q.clear()
            back = None
        front.pop()

/// State a window can add to at the back and remove from at the front. `pop(x)` is passed the
/// element leaving the front. `Window(op)`, `Counter<K, Int>`, `Tracked` and `Undoing` implement it.
pub trait Track<T>:
    fn push(mut self, x: T)
    fn pop(mut self, x: T)

/// `push(x)` adds 1 to `x`'s count and `pop(x)` takes 1 away, so `len` is the number of distinct
/// values in the window.
impl Track<K> for Counter<K, Int>:
    fn push(mut self, x: K):
        self.add(x, 1)
    fn pop(mut self, x: K):
        self.add(x, -1)

/// `push(x)` is `Window.push(x)`; `pop(x)` is `Window.pop()`, which already knows the front.
impl Track<T> for Window<T, M>:
    fn push(mut self, x: T):
        self.push_back(x)
    fn pop(mut self, x: T):
        self.drop_front()

/// A `Track` over any structure: `add(mut state, x)` applies an element at the back and
/// `remove(mut state, x)` takes the front one back out, so `ok` reads `w.state`. The same shape
/// as `undoing`, for structures that can remove directly.
///
/// `push(x)` calls `add(state, x)` and `pop(x)` calls `remove(state, x)`; the `Tracked` keeps no
/// queue, so the caller passes the leaving element to `pop`.
///
/// cost: O(1)
/// traps: never
/// example: `var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); t.push(3); t.push(4); t.pop(3); t.state == 4`
pub fn track<S, T>(state: S, add: (mut S, T) -> (), remove: (mut S, T) -> ()) -> Tracked<S, T> = Tracked(state, add, remove)

/// A `Track` built from a structure and two closures.
pub struct Tracked<S, T>(state: S, add: (mut S, T) -> (), remove: (mut S, T) -> ()):
    /// Apply `x` at the back: one call of `add(state, x)`.
    ///
    /// cost: one call of `add`
    /// traps: never (beyond what `add` traps on)
    /// example: `var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); t.push(3); t.state == 3`
    pub fn push(mut self, x: T) = todo
    /// Take the front element `x` back out: one call of `remove(state, x)`. `x` is passed through
    /// as given; nothing checks that it is the oldest element.
    ///
    /// cost: one call of `remove`
    /// traps: never (beyond what `remove` traps on)
    /// example: `var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); t.push(3); t.push(4); t.pop(3); t.state == 4`
    pub fn pop(mut self, x: T) = todo

    /// The structure with exactly the window's elements applied.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `track(7, (mut s, x) => s += x, (mut s, x) => s -= x).state == 7`
    @get pub fn state(self) -> S = state

/// A structure that can return to an earlier state, newest change first: `Dsu`, `ParityDsu`,
/// `WeightedDsu`, `DsuWith`, `RollbackArray`, and any user struct with these two methods.
pub trait Undo:
    fn mark(mut self) -> Mark
    fn undo_to(mut self, snap: Mark)

// blocked: `(mut S, T) -> ()` fn types don't parse (GAPS.md window)
/// The queue-undo trick: a `Track` over an `Undo` structure, so a window that adds at the back
/// and drops from the front runs on a structure that can only undo its newest change (a
/// rollback DSU in a two-pointer pass). `add(mut state, x)` applies one element; `state`
/// always holds exactly the window's elements. Each element is applied O(log n) times
/// amortized, so a pass of n pushes and pops costs O(n log n) applications.
pub struct Undoing<S, T>(state: S, add: (mut S, T) -> ()):
    /// Push `x` at the back: one `add` on top of the current state.
    ///
    /// cost: O(1) applications of `add` amortized
    /// traps: never (beyond what `add` traps on)
    /// example: `domain V = 0..3; var u = undoing(Dsu<V>(rollback: true), (mut d, e) => d.unite(e.0, e.1)); u.push((V(0), V(1))); u.state.same(V(0), V(1))`
    pub fn push(mut self, x: T) = todo

    /// Drop the front element, which must be `x`. The element removed is the stored oldest one;
    /// `x` names it for the `Track` protocol and is not compared. Undoes back past the oldest
    /// change and re-applies the rest (the queue-undo trick).
    ///
    /// cost: O(log n) applications of `add` amortized
    /// traps: on an empty `Undoing`
    /// example: `domain V = 0..3; var u = undoing(Dsu<V>(rollback: true), (mut d, e) => d.unite(e.0, e.1)); u.push((V(0), V(1))); u.push((V(1), V(2))); u.pop((V(0), V(1))); u.state.same(V(1), V(2)) and not u.state.same(V(0), V(1))`
    pub fn pop(mut self, x: T) = todo

    /// The structure with exactly the window's elements applied.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..3; let u = undoing(Dsu<V>(rollback: true), (mut d, e) => d.unite(e.0, e.1)); not u.state.same(V(0), V(1))`
    @get pub fn state(self) -> S = todo

// blocked: `(mut S, T) -> ()` fn types don't parse (GAPS.md window)
/// `Undoing` over `state`, holding no elements; see there. `state` is taken as the base: the
/// window never undoes past it.
///
/// cost: O(1)
/// traps: never
/// example: `domain V = 0..3; var u = undoing(Dsu<V>(rollback: true), (mut d, e) => d.unite(e.0, e.1)); u.push((V(0), V(2))); u.state.same(V(2), V(0))`
pub fn undoing<S: Undo, T>(state: S, add: (mut S, T) -> ()) -> Undoing<S, T> = todo

/// A movable interval over `D` feeding a `Track`, for Mo and divide-and-conquer DP. Moving
/// expands before shrinking. O(|change|) pushes and pops per move.
pub struct Sweep<D, S>(state: S):
    lo: Int
    hi: Int

    /// A sweep over `state`, starting empty (or at `start`). Empty means the range is empty at the
    /// first cut of `D`; with `start`, each position of `start` is pushed in increasing order.
    ///
    /// cost: O(1), plus `start.len` pushes when `start` is given
    /// empty: an empty `start` is the same as None, but sits at `start`'s cut
    /// traps: never (beyond what `state.push` traps on)
    /// example: `domain D = 0..5; let s = Sweep<D, Counter<D, Int>>(Counter<D, Int>(0)); s.range.len == 0`
    /// example: `domain D = 0..5; let s = Sweep<D, Counter<D, Int>>(Counter<D, Int>(0), start: Some(1..3)); s.range == 1..3`
    pub fn new(state: S, start: ?Range<D> = None) -> Sweep<D, S>:
        var s = Sweep<D, S>(state, lo: 0, hi: 0)
        if let r = start: s.move(r)
        s

    /// Slide to `to`, feeding `state` the positions that enter and leave. Moving expands before
    /// shrinking, in this order: grow the right end (push each new position, increasing), grow the
    /// left end (push, decreasing), shrink the right end (pop, decreasing), shrink the left end
    /// (pop, increasing). The state never sees a negative-length interval, but growing on the left
    /// and shrinking on the right touch the "wrong" end of the `Track`, so as with Mo the state's
    /// answer must ignore order.
    ///
    /// cost: O(|dl| + |dr|) pushes and pops, where `dl` and `dr` are the moves of the two ends
    /// empty: moving to an empty `to` pops everything
    /// traps: never (beyond what the state traps on)
    /// example: `domain D = 0..5; var s = Sweep<D, Counter<D, Int>>(Counter<D, Int>(0)); s.move(1..4); s.move(2..5); s.range == 2..5`
    pub fn move(mut self, to: Range<D>):
        let keys = [d for d in D]
        var (l, r) = (0, 0)
        if not to.is_empty:
            l = Int(to.first!) - Int(keys[0])
            r = Int(to.last!) - Int(keys[0]) + 1
        while hi < r:
            state.push(keys[hi])
            hi += 1
        while lo > l:
            lo -= 1
            state.push(keys[lo])
        while hi > r:
            hi -= 1
            state.pop(keys[hi])
        while lo < l:
            state.pop(keys[lo])
            lo += 1

    /// The current interval: the positions whose pushes the state holds.
    ///
    /// cost: O(1)
    /// traps: never
    /// order: increasing positions of `D`
    /// example: `domain D = 0..5; var s = Sweep<D, Counter<D, Int>>(Counter<D, Int>(0)); s.move(0..2); s.range.len == 2`
    @get pub fn range(self) -> Range<D>:
        let keys = [d for d in D]
        if lo < hi: keys[lo]..=keys[hi - 1] else: keys[0]..keys[0]
