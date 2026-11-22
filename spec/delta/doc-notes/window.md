# Doc notes: `std.window`

- `Window.pop`: chose to trap on an empty window (nothing to remove), like
  `List.pop` on an empty list would if it returned nothing.
- `Window.all`: chose to fold oldest first (push order), which matters for a
  non-commutative op such as matrix product.
- Contradiction: lib/sequence-patterns.md says `Window(op)` implements
  `Track<T>`, whose `pop` takes the leaving element (`pop(mut self, x: T)`);
  `Window.pop` takes none. Left the signature; one side should change.
- Contradiction: lib/ops-ranges.md §16 still shows the old API
  (`w.push(i, a[i])`, `w.drop_before(i - k)`, `w.fold`) and §10 a `drop` method;
  the stub has `push(x)`, `pop()` and `all`. Documented the stub. (resolved: kept stub; ops-ranges §10 and §16 fixed)
- `Tracked.pop(x)`: passes `x` to `remove` as given; it does not check that `x`
  is the oldest element (it keeps no queue).
- `Undoing.pop(x)`: chose that the element removed is the stored front and `x`
  is not compared (that would need `T: Eq`); popping an empty `Undoing` traps.
- `Sweep`: chose that it feeds the state the positions of `D` (state is a
  `Track<D>`); the spec never says what element type a `Sweep` pushes.
- `Sweep.move`: chose the order: grow the right end, grow the left end, shrink
  the right end, shrink the left end ("expands before shrinking"). Growing on
  the left and shrinking on the right push and pop at the "wrong" end of the
  `Track`, so like Mo it needs a state whose answer ignores order.
- `Sweep.new`: an empty sweep sits at the first cut of `D` (`range` is empty
  there); with `start`, it pushes each position of `start` in increasing order.
- Contradiction: `Sweep.new` says O(1), but starting at a non-empty `start`
  must push its `start.len` positions; documented O(1) plus those pushes. (resolved: kept the documented cost)
- `undoing(state, add)`: chose that `state` as passed is the base the window
  never undoes past (its own `mark` is taken at creation).
- `Window.all`, `Tracked.state` examples use an `Int` state; `track` closures
  are written `(mut s, x) => s += x`, as `(mut d, e) => ...` in core.
