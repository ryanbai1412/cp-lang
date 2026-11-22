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

    /// An empty window over `op`. O(1).
    pub fn new(op: M) -> Window<T, M> = todo

    /// Push `x` at the back. O(1) amortized.
    pub fn push(mut self, x: T) = todo

    /// Pop the front. O(1) amortized.
    pub fn pop(mut self) = todo

    /// Fold of the current contents, `none` when empty. O(1).
    pub @get fn all(self) -> ?T = todo

    /// Number of elements. O(1).
    pub @get fn len(self) -> Int = todo

/// State a window can add to at the back and remove from at the front. `Window(op)`,
/// `Counter<K>`, `Bag<T>` and `Undoing` implement it.
pub trait Track<T>:
    fn push(mut self, x: T)
    fn pop(mut self, x: T)

/// A `Track` over any structure: `add(mut state, x)` applies an element at the back and
/// `remove(mut state, x)` takes the front one back out, so `ok` reads `w.state`. The same shape
/// as `undoing`, for structures that can remove directly. O(1).
pub fn track<S, T>(state: S, add: (mut S, T) -> (), remove: (mut S, T) -> ()) -> Tracked<S, T> = todo

/// A `Track` built from a structure and two closures.
pub struct Tracked<S, T>(state: S, add: (mut S, T) -> (), remove: (mut S, T) -> ()):
    pub fn push(mut self, x: T) = todo
    pub fn pop(mut self, x: T) = todo

    /// The structure with exactly the window's elements applied. O(1).
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
    /// Push `x` at the back. O(1) applications amortized.
    pub fn push(mut self, x: T) = todo

    /// Drop the front element, which must be `x`. O(log n) applications amortized.
    pub fn pop(mut self, x: T) = todo

    /// The structure with exactly the window's elements applied. O(1).
    pub @get fn state(self) -> S = todo

/// O(1). `Undoing` over `state`; see there.
pub fn undoing<S: Undo, T>(state: S, add: (mut S, T) -> ()) -> Undoing<S, T> = todo

/// A movable interval over `D` feeding a `Track`, for Mo and divide-and-conquer DP. Moving
/// expands before shrinking. O(|change|) pushes and pops per move.
pub struct Sweep<D, S>(state: S):
    lo: Int
    hi: Int

    /// A sweep over `state`, starting empty (or at `start`). O(1).
    pub fn new(state: S, start: ?Range<D> = none) -> Sweep<D, S> = todo

    /// Slide to `to`. O(|dl| + |dr|).
    pub fn move(mut self, to: Range<D>) = todo

    /// The current interval. O(1).
    pub @get fn range(self) -> Range<D> = todo
