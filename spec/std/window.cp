//! Sliding-window folds and stateful windows (round 7, `lib/sequence-patterns.md`).
//!
//! `a.windows(k).fold(op)` and `Window(op)` pick their strategy from the op's laws:
//! a `Group` adds and subtracts, a `Selective` op keeps a monotone deque, anything else uses
//! two stacks. State that is not a fold implements `Track` (`push`/`pop`); two pointers, Mo
//! and divide-and-conquer DP all accept a `Track`.
use std.algebra.{Op}

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
/// `Counter<K>` and `Bag<T>` implement it.
pub trait Track<T>:
    fn push(mut self, x: T)
    fn pop(mut self, x: T)

/// A `Track` from two closures. O(1).
pub fn track<T>(add: T -> (), remove: T -> ()) -> Tracked<T> = todo

/// A `Track` built from closures.
pub struct Tracked<T>(add: T -> (), remove: T -> ())

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
