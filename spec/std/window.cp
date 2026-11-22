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

    /// An empty window over `op`. The strategy follows `op`'s laws: a `Group` adds and
    /// subtracts, a `Selective` op keeps a monotone deque, anything else uses two stacks.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var w = Window.new(min); w.push(4); w.all == Some(4)`
    pub fn new(op: M) -> Window<T, M> = todo

    /// Pushes `x` at the back.
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `var w = Window.new(+); w.push(1); w.push(2); w.all == Some(3)`
    pub fn push(mut self, x: T) = todo

    /// Removes the front element (the oldest pushed).
    ///
    /// cost: O(1) amortized
    /// traps: when the window is empty
    /// example: `var w = Window.new(min); w.push(1); w.push(3); w.push(2); w.pop(); w.all == Some(2)`
    pub fn pop(mut self) = todo

    /// The fold of the contents front to back, `None` when empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var w = Window.new(max); w.push(1); w.push(3); w.all == Some(3)`
    /// example: `Window.new(+).all == None`
    pub @get fn all(self) -> ?T = todo

    /// The number of elements in the window.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var w = Window.new(+); w.push(1); w.push(2); w.pop(); w.len == 1`
    pub @get fn len(self) -> Int = todo

/// State a window can add to at the back and remove from at the front. `Window(op)`,
/// `Counter<K>`, `Bag<T>` and `Undoing` implement it.
pub trait Track<T>:
    fn push(mut self, x: T)
    fn pop(mut self, x: T)

/// A `Track` over any structure: `add(mut state, x)` applies an element at the back and
/// `remove(mut state, x)` takes the front one back out, so `ok` reads `w.state`. The same shape
/// as `undoing`, for structures that can remove directly.
///
/// cost: O(1)
/// traps: never
/// example: `var t = track(0, add: (mut s, x) => s += x, remove: (mut s, x) => s -= x); t.push(3); t.push(4); t.pop(3); t.state == 4`
pub fn track<S, T>(state: S, add: (mut S, T) -> (), remove: (mut S, T) -> ()) -> Tracked<S, T> = todo

/// A `Track` built from a structure and two closures.
pub struct Tracked<S, T>(state: S, add: (mut S, T) -> (), remove: (mut S, T) -> ()):
    /// Applies `x` at the back: calls `add(mut state, x)`.
    ///
    /// cost: O(1) plus one call of `add`
    /// traps: never
    /// example: `var t = track(0, add: (mut s, x) => s += x, remove: (mut s, x) => s -= x); t.push(5); t.state == 5`
    pub fn push(mut self, x: T) = todo
    /// Takes the front element `x` back out: calls `remove(mut state, x)`. `x` is not checked
    /// against the front; passing it correctly is the caller's job.
    ///
    /// cost: O(1) plus one call of `remove`
    /// traps: never
    /// example: `var t = track(0, add: (mut s, x) => s += x, remove: (mut s, x) => s -= x); t.push(5); t.push(2); t.pop(5); t.state == 2`
    pub fn pop(mut self, x: T) = todo

    /// The structure with exactly the window's elements applied.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `track(7, add: (mut s, x) => s += x, remove: (mut s, x) => s -= x).state == 7`
    pub @get fn state(self) -> S = todo

/// A structure that can return to an earlier state, newest change first: `Dsu`, `ParityDsu`,
/// `WeightedDsu`, `DsuWith`, `RollbackArray`, and any user struct with these two methods.
pub trait Undo:
    fn mark(mut self) -> Mark
    fn undo_to(mut self, snap: Mark)

/// The queue-undo trick: a `Track` over an `Undo` structure, so a window that adds at the back
/// and drops from the front runs on a structure that can only undo its newest change (a
/// rollback DSU in a two-pointer pass). `add(mut state, x)` applies one element; `state`
/// always holds exactly the window's elements. Each element is applied O(log n) times
/// amortized, so a pass of n pushes and pops costs O(n log n) applications.
pub struct Undoing<S, T>(state: S, add: (mut S, T) -> ()):
    /// Pushes `x` at the back and applies it.
    ///
    /// cost: O(1) applications amortized
    /// traps: never
    /// example: `var w = undoing(RollbackArray<0..3, Int>(init: i => 0), add: (mut a, x) => a.set(x, a.get(x) + 1)); w.push(1); w.state.get(1) == 1`
    pub fn push(mut self, x: T) = todo

    /// Drops the front element, which must be `x`: undoes back past it and reapplies the
    /// others, regrouped so each element is reapplied O(log n) times amortized.
    ///
    /// cost: O(log n) applications amortized
    /// traps: when the window is empty or its front is not `x`
    /// example: `var w = undoing(RollbackArray<0..3, Int>(init: i => 0), add: (mut a, x) => a.set(x, a.get(x) + 1)); w.push(1); w.push(1); w.push(2); w.pop(1); w.state.get(1) == 1 and w.state.get(2) == 1`
    pub fn pop(mut self, x: T) = todo

    /// The structure with exactly the window's elements applied.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var w = undoing(RollbackArray<0..3, Int>(init: i => 0), add: (mut a, x) => a.set(x, a.get(x) + 1)); w.push(2); w.push(2); w.state.get(2) == 2`
    pub @get fn state(self) -> S = todo

/// The `Undoing` window over `state`; see there.
///
/// cost: O(1)
/// traps: never
/// example: `undoing(RollbackArray<0..3, Int>(init: i => 0), add: (mut a, x) => a.set(x, a.get(x) + 1)).state.get(0) == 0`
pub fn undoing<S: Undo, T>(state: S, add: (mut S, T) -> ()) -> Undoing<S, T> = todo

/// A movable interval over `D` feeding a `Track`, for Mo and divide-and-conquer DP. Moving
/// expands before shrinking. O(|change|) pushes and pops per move.
pub struct Sweep<D, S>(state: S):
    lo: Int
    hi: Int

    /// A sweep over `state`, starting at the empty range before `D.first`, or at `start` (its
    /// positions pushed in increasing order).
    ///
    /// cost: O(1), plus O(|start|) pushes
    /// empty: `start: None` gives an empty range at the first cut
    /// traps: never
    /// example: `let t = track(0, add: (mut s, i) => s += Int(i), remove: (mut s, i) => s -= Int(i)); var w: Sweep<0..4, Tracked<Int, 0..4>> = Sweep.new(t, start: Some(1..3)); w.range == 1..3`
    pub fn new(state: S, start: ?Range<D> = None) -> Sweep<D, S> = todo

    /// Slides the range to `to`, expanding before shrinking: first push the positions added on
    /// the right (increasing), then those added on the left (decreasing), then pop those
    /// removed on the left (increasing), then those removed on the right (decreasing).
    ///
    /// cost: O(|dl| + |dr|) pushes and pops
    /// empty: moving to an empty range pops everything
    /// traps: never
    /// example: `let t = track(0, add: (mut s, i) => s += Int(i), remove: (mut s, i) => s -= Int(i)); var w: Sweep<0..4, Tracked<Int, 0..4>> = Sweep.new(t); w.move(1..3); w.move(2..4); w.range == 2..4`
    pub fn move(mut self, to: Range<D>) = todo

    /// The current range.
    ///
    /// cost: O(1)
    /// order: increasing positions
    /// traps: never
    /// example: `let t = track(0, add: (mut s, i) => s += Int(i), remove: (mut s, i) => s -= Int(i)); var w: Sweep<0..4, Tracked<Int, 0..4>> = Sweep.new(t); w.range == 0..0`
    pub @get fn range(self) -> Range<D> = todo
