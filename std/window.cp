//! Sliding-window folds and stateful windows (round 7, `lib/sequence-patterns.md`).
//!
//! start: `window_fold`, `Window`, `track`, `undoing`, `Sweep`
//! example: `[3, 1, 4, 1, 5].window_fold(2, max).collect() == [3, 4, 4, 5]`
//!
//! `a.window_fold(k, op)` and `Window(op)` keep two stacks for every op today; the law-picked
//! strategies of `lib/sequence-patterns.md` (a `Group` adds and subtracts, a `Selective` op keeps
//! a monotone deque) wait on law-bound dispatch over op values. State that is not a fold implements `Track` (`push`/`pop`); two pointers, Mo
//! and divide-and-conquer DP all accept a `Track`.
use std.algebra.{Op}
use std.dsu.{Mark, Dsu, DsuOf, ParityDsu, PotentialDsu}
use std.hashmap.{Counter}
use std.seq.{Track, Indexed, collect}
use std.domains.{Domain}
use std.list.{List}
use std.ops.{Eq}
use std.optional.{Some}
use std.ranges.{Range}
use std.show.{Show}

/// One fold of `op` per length-`k` window of `self`, left to right, kept the way `Window(op)`
/// keeps it (two stacks).
///
/// cost: \(O(n)\) amortized calls of `op`
/// empty: `[List<Int>().window_fold(1, max).collect(), [1, 2].window_fold(3, max).collect()] == [[], []]`
/// order: by window start, left to right
/// traps: when `k < 1`
/// example: `[3, 1, 4, 1, 5].window_fold(2, max).collect() == [3, 4, 4, 5]`
/// example: `[1, 2, 3].window_fold(2, +).collect() == [3, 5]`
pub fn window_fold<S: Indexed<D, T>, D: Domain, T: Eq, M: Op<T>>(self: S, k: 1.., op: M) -> yields T:
    assert k >= 1, "a window holds at least one element"
    var w = Window<T, M>.new(op)
    for i in 0..self.len:
        w.push(self.nth(i))
        if i >= k: w.pop(self.nth(i - k))
        if i >= k - 1: yield w.all!

/// A sliding fold queue over `op`. `push` and `pop` are \(O(1)\) amortized; `all` is \(O(1)\).
/// `all` is `None` when empty for every op today. `SegTree` already folds an empty bounded
/// `max`/`min` to `T.MIN`/`T.MAX` (decision 602); `Window` takes the same rule when `all` becomes
/// `M.Fold`, and `Window(max.opt())` is the fold that stays `None` when empty.
pub struct Window<T, M: Op<T>>(op: M):
    var q: List<T>
    front: List<T>
    var olds: List<T>
    var head: Int
    var back: ?T
    var keys: List<Int>
    var oldkeys: List<Int>

    /// An empty window over `op`, kept as two stacks: the back folds as it grows, and the front
    /// holds suffix folds of the older elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var w = Window(+); w.all == None`
    pub fn new(op: M) -> Window<T, M>:
        Window(op, [], [], [], 0, None, [], [])

    /// Push `x` at the back, as the newest element. With `at: place`, `x` is keyed by its place,
    /// so `expire(before:)` removes it without being told its value. A keyed window keys every
    /// push, and places never decrease.
    ///
    /// cost: \(O(1)\) amortized calls of `op`
    /// traps: when `at:` is given to a window that has unkeyed elements or the other way round, or
    /// when `at` is below the previous place
    /// example: `var w = Window(max); w.push(3); w.push(5); w.all == Some(5)`
    /// example: `var w = Window(max); w.push(4, at: 0); w.push(1, at: 2); w.expire(before: 1); w.all == Some(1)`
    /// trap: `var w = Window(max); w.push(4, at: 5); w.push(6, at: 7); w.expire(before: 6); w.push(1, at: 3)`
    pub fn push(mut self, x: T, at: ?Int = None):
        if let k = at:
            assert keys.len == q.len and oldkeys.len == olds.len, "`push(x, at:)` on a window with unkeyed elements"
            if keys.len > 0: assert k >= keys.last!, "`push(x, at:)` places never decrease"
            elif head < oldkeys.len: assert k >= oldkeys.last!, "`push(x, at:)` places never decrease"
            keys.push(k)
        else:
            assert keys.len == 0 and oldkeys.len == 0, "`push(x)` on a keyed window needs `at:`"
        self.push_back(x)

    /// Remove every element pushed `at:` a place below `before`, oldest first. The fold of what
    /// is left is `all`.
    ///
    /// cost: \(O(1)\) amortized calls of `op` per removed element
    /// empty: `var w = Window(min); w.expire(before: 5); w.all == None`
    /// traps: when the window has unkeyed elements
    /// example: `var w = Window(+); w.push(1, at: 0); w.push(2, at: 1); w.push(3, at: 2); w.expire(before: 2); w.all == Some(3)`
    /// example: `var w = Window(min); w.push(1, at: 0); w.push(2, at: 3); w.expire(before: 1); w.expire(before: 1); w.all == Some(2)`
    pub fn expire(mut self, before: Int):
        assert keys.len == q.len and oldkeys.len == olds.len, "`expire` on a window with unkeyed elements"
        while true:
            if head == olds.len:
                if q.len == 0 or keys[0] >= before: return
                self.flip()
            if oldkeys[head] >= before: return
            head += 1
            front.pop()

    /// Pop the front: remove the oldest element, `x`. Passing the leaving element is the same
    /// protocol as `Track.pop(x)`, so a `Window` is a `Track` with no second spelling.
    ///
    /// cost: \(O(1)\) amortized calls of `op`
    /// traps: on an empty window, or in a checked build when `x` is not the oldest element
    /// example: `var w = Window(min); w.push(3); w.push(1); w.push(2); w.pop(3); w.all == Some(1)`
    /// example: `var w = Window(min); w.push(1); w.push(2); w.pop(1); w.all == Some(2)`
    pub fn pop(mut self, x: T) where T: Eq:
        self.drop_front(x)

    /// The fold of the current contents, oldest first (`op(op(x1, x2), x3)` for pushes `x1`, `x2`,
    /// `x3`), so a non-commutative op folds in push order; `None` when empty.
    ///
    /// cost: \(O(1)\)
    /// empty: `Window<Int, (Int, Int) -> Int>(+).all == None`
    /// traps: never
    /// example: `var w = Window(+); w.push(2); w.push(3); w.all == Some(5)`
    /// example: `var w = Window(+); w.push(2); w.pop(2); w.all == None`
    @get pub fn all(self) -> ?T:
        if front.len == 0: return back
        let f = front.last!
        if let b = back: return Some(op.op(f, b))
        Some(f)

    /// The number of elements in the window: pushes minus pops.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var w = Window(+); w.push(2); w.push(3); w.pop(2); w.len == 1`
    @get pub fn len(self) -> Int:
        front.len + q.len

    /// Whether the window holds no elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var w = Window(+); w.push(2); w.pop(2); w.is_empty`
    @get pub fn is_empty(self) -> Bool = self.len == 0

    /// The elements in the window, oldest first.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) per step
    /// empty: `var w = Window(+); w.push(2); w.pop(2); w.items.collect().len == 0`
    /// order: push order, oldest first
    /// traps: never
    /// example: `var w = Window(+); w.push(1); w.push(2); w.push(3); w.pop(1); w.push(4); w.items.collect() == [2, 3, 4]`
    @get pub fn items(self) -> yields T:
        for i in head..olds.len: yield olds[i]
        for x in q: yield x

    fn push_back(mut self, x: T):
        q.push(x)
        if let b = back: back = Some(op.op(b, x))
        else: back = Some(x)

    fn flip(mut self):
        var k = q.len - 1
        front.push(q[k])
        while k > 0:
            k -= 1
            let acc = op.op(q[k], front.last!)
            front.push(acc)
        olds = q
        q = []
        oldkeys = keys
        keys = []
        head = 0
        back = None

    fn drop_front(mut self, x: T) where T: Eq:
        if front.len == 0:
            assert q.len > 0, "`pop` on an empty window"
            self.flip()
        assert olds[head] == x, "`pop(x)` is passed the oldest element, and `x` is not it"
        head += 1
        front.pop()

impl<T, M: Op<T>> Show for Window<T, M>:
    fn show(w) = "{w.len} elements"

/// `push(x)` and `pop(x)` are `Window`'s own `push(x)` and `pop(x)`.
impl<T: Eq, M: Op<T>> Track<T> for Window<T, M>:
    fn push(mut self, x: T):
        self.push_back(x)
    fn pop(mut self, x: T):
        self.drop_front(x)

/// `push(x)` and `pop(x)` are the `Tracked`'s own `push(x)` and `pop(x)`.
impl Track<T> for Tracked<S, T>:
    fn push(mut self, x: T) = self.push(x)
    fn pop(mut self, x: T) = self.pop(x)

/// `push(x)` and `pop(x)` are the `Undoing`'s own `push(x)` and `pop(x)`.
impl<S: Undo, T> Track<T> for Undoing<S, T>:
    fn push(mut self, x: T) = self.push(x)
    fn pop(mut self, x: T) = self.pop(x)

/// A `Track` over any structure: `add(mut state, x)` applies an element at the back and
/// `remove(mut state, x)` takes the front one back out, so `ok` reads `w.state`. The same shape
/// as `undoing`, for structures that can remove directly.
///
/// `push(x)` calls `add(state, x)` and `pop(x)` calls `remove(state, x)`; the `Tracked` keeps no
/// queue, so the caller passes the leaving element to `pop`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); t.push(3); t.push(4); t.pop(3); t.state == 4`
pub fn track<S, T>(
    state: S,
    add: (mut S, T) -> (),
    remove: (mut S, T) -> (),
) -> Tracked<S, T> = Tracked(state, add, remove)

/// A `Track` built from a structure and two closures.
pub struct Tracked<S, T>(pub state: S, add: (mut S, T) -> (), remove: (mut S, T) -> ()):
    /// Apply `x` at the back: one call of `add(state, x)`.
    ///
    /// cost: \(O(1)\) plus one call of `add`
    /// traps: never (beyond what `add` traps on)
    /// example: `var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); t.push(3); t.state == 3`
    pub fn push(mut self, x: T):
        add(mut state, x)
    /// Take the front element `x` back out: one call of `remove(state, x)`. `x` is passed through
    /// as given; nothing checks that it is the oldest element.
    ///
    /// cost: \(O(1)\) plus one call of `remove`
    /// traps: never (beyond what `remove` traps on)
    /// example: `var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x); t.push(3); t.push(4); t.pop(3); t.state == 4`
    pub fn pop(mut self, x: T):
        remove(mut state, x)

    /// The structure with exactly the window's elements applied.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `track(7, (mut s, x) => s += x, (mut s, x) => s -= x).state == 7`
    @get pub fn state(self) -> S = state

/// A structure that can return to an earlier state, newest change first: `Dsu`, `ParityDsu`,
/// `WeightedDsu`, `DsuWith`, `RollbackArray`, and any user struct with these two methods.
pub trait Undo:
    fn mark(mut self) -> Mark
    fn undo_to(mut self, snap: Mark)

impl Undo for Dsu<V>:
    fn mark(mut self) -> Mark = self.mark()
    fn undo_to(mut self, snap: Mark) = self.undo_to(snap)

impl Undo for DsuOf<V, T>:
    fn mark(mut self) -> Mark = self.mark()
    fn undo_to(mut self, snap: Mark) = self.undo_to(snap)

impl Undo for ParityDsu<V>:
    fn mark(mut self) -> Mark = self.mark()
    fn undo_to(mut self, snap: Mark) = self.undo_to(snap)

impl Undo for PotentialDsu<V>:
    fn mark(mut self) -> Mark = self.mark()
    fn undo_to(mut self, snap: Mark) = self.undo_to(snap)

/// The queue-undo trick: a `Track` over an `Undo` structure, so a window that adds at the back
/// and drops from the front runs on a structure that can only undo its newest change (a
/// rollback DSU in a two-pointer pass). `add(mut state, x)` applies one element; `state`
/// always holds exactly the window's elements. Each element is applied \(O(\log n)\) times
/// amortized, so a pass of n pushes and pops costs \(O(n \log n)\) applications.
pub struct Undoing<S: Undo, T>(pub state: S, add: (mut S, T) -> ()):
    items: List<T> = []
    is_b: List<Bool> = []
    marks: List<Mark> = []
    var a_cnt: Int = 0

    fn push_entry(mut self, x: T, b: Bool):
        marks.push(state.mark())
        add(mut state, x)
        items.push(x)
        is_b.push(b)
        if not b: a_cnt += 1

    /// Push `x` at the back: one `add` on top of the current state.
    ///
    /// cost: \(O(1)\) applications of `add` amortized
    /// traps: never (beyond what `add` traps on)
    /// example: `domain V = 0..3; var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1)); u.push((V(0), V(1))); u.state.same(V(0), V(1))`
    pub fn push(mut self, x: T):
        self.push_entry(x, true)

    /// Drop the front element, which must be `x`. The element removed is the stored oldest one;
    /// `x` names it for the `Track` protocol and is not compared. Undoes back past the oldest
    /// change and re-applies the rest (the queue-undo trick).
    ///
    /// cost: \(O(\log n)\) applications of `add` amortized
    /// traps: on an empty `Undoing`
    /// example: `domain V = 0..3; var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1)); u.push((V(0), V(1))); u.push((V(1), V(2))); u.pop((V(0), V(1))); u.state.same(V(1), V(2)) and not u.state.same(V(0), V(1))`
    /// trap: `domain V = 0..3; var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1)); u.pop((V(0), V(1)))`
    pub fn pop(mut self, x: T):
        assert not items.is_empty, "pop on empty Undoing"
        if a_cnt == 0:
            let base = marks[0]
            var rev_items: List<T> = []
            while not items.is_empty:
                rev_items.push(items.pop()!)
                is_b.pop()
                marks.pop()
            state.undo_to(base)
            for i in 0..rev_items.len - 1:
                self.push_entry(rev_items[i], false)
            return
        if not is_b[items.len - 1]:
            let m = marks.pop()!
            items.pop()
            is_b.pop()
            a_cnt -= 1
            state.undo_to(m)
            return
        var b_buf: List<T> = []
        var a_buf: List<T> = []
        var base = marks.last!
        while let item = items.pop():
            let b = is_b.pop()!
            base = marks.pop()!
            if b:
                b_buf.push(item)
            else:
                a_buf.push(item)
                a_cnt -= 1
                if a_buf.len == b_buf.len or a_cnt == 0:
                    break
        state.undo_to(base)
        for i in (0..b_buf.len).rev:
            self.push_entry(b_buf[i], true)
        for i in (1..a_buf.len).rev:
            self.push_entry(a_buf[i], false)

    /// The structure with exactly the window's elements applied.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain V = 0..3; let u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1)); not u.state.same(V(0), V(1))`
    @get pub fn state(self) -> S = state

/// `Undoing` over `state`, holding no elements; see there. `state` is taken as the base: the
/// window never undoes past it.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain V = 0..3; var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1)); u.push((V(0), V(2))); u.state.same(V(2), V(0))`
pub fn undoing<S: Undo, T>(state: S, add: (mut S, T) -> ()) -> Undoing<S, T> = Undoing(state, add)

/// A movable range over `D` feeding a `Track`, for Mo and divide-and-conquer DP. Moving
/// expands before shrinking. \(O(\lvert \mathit{change} \rvert)\) pushes and pops per move.
pub struct Sweep<D: Domain, S: Track<D>>(pub state: S):
    var lo: Int
    var hi: Int

    /// A sweep over `state`, starting empty (or at `start`). Empty means the range is empty at the
    /// first cut of `D`; with `start`, each position of `start` is pushed in increasing order.
    ///
    /// cost: \(O(1)\), plus `start.len` pushes when `start` is given
    /// empty: `domain D = 0..5; let s = Sweep<D, Counter<D, Int>>(Counter<D, Int>(), start: Some(3..3)); s.range == D(3)..D(3)`
    /// traps: never (beyond what `state.push` traps on)
    /// example: `domain D = 0..5; let s = Sweep<D, Counter<D, Int>>(Counter<D, Int>()); s.range.len == 0`
    /// example: `domain D = 0..5; let s = Sweep<D, Counter<D, Int>>(Counter<D, Int>(), start: Some(1..3)); s.range == 1..3`
    pub fn new(state: S, start: ?Range<D> = None) -> Sweep<D, S>:
        var s = Sweep<D, S>(state, lo: 0, hi: 0)
        if let r = start: s.move(r)
        s

    /// Slide to `to`, feeding `state` the positions that enter and leave.
    ///
    /// Moving expands before shrinking, in this order: grow the right end (push each new position,
    /// increasing), grow the left end (push, decreasing), shrink the right end (pop, decreasing),
    /// shrink the left end (pop, increasing). The state never sees a negative-length range, but
    /// growing on the left and shrinking on the right touch the "wrong" end of the `Track`, so as
    /// with Mo the state's answer must ignore order.
    ///
    /// cost: \(O(\lvert \mathit{dl} \rvert + \lvert \mathit{dr} \rvert)\) pushes and pops, where `dl` and `dr` are the moves of the two ends
    /// empty: `domain D = 0..5; var s = Sweep<D, Counter<D, Int>>(Counter<D, Int>()); s.move(1..4); s.move(2..2); (s.range.len, s.state.total()) == (0, 0)`
    /// traps: never (beyond what the state traps on)
    /// example: `domain D = 0..5; var s = Sweep<D, Counter<D, Int>>(Counter<D, Int>()); s.move(1..4); s.move(2..5); s.range == 2..5`
    pub fn move(mut self, to: Range<D>):
        if to.is_empty:
            while hi > lo:
                hi -= 1
                state.pop(D.nth(hi))
            lo = Int(to.l) - Int(D.cuts.nth(0))
            hi = lo
            return
        let l = Int(to.first!) - Int(D.nth(0))
        let r = Int(to.last!) - Int(D.nth(0)) + 1
        while hi < r:
            state.push(D.nth(hi))
            hi += 1
        while lo > l:
            lo -= 1
            state.push(D.nth(lo))
        while hi > r:
            hi -= 1
            state.pop(D.nth(hi))
        while lo < l:
            state.pop(D.nth(lo))
            lo += 1

    /// The current range: the positions whose pushes the state holds.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// order: increasing positions of `D`
    /// example: `domain D = 0..5; var s = Sweep<D, Counter<D, Int>>(Counter<D, Int>()); s.move(0..2); s.range.len == 2`
    @get pub fn range(self) -> Range<D>:
        if lo < hi: D.nth(lo)..=D.nth(hi - 1) else: D.cuts.nth(lo)..D.cuts.nth(lo)
