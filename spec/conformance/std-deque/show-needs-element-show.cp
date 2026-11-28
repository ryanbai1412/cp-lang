// expect: E0606
// rule: R08.2.2
// engines: next
// has: "does not implement `Show`"
// std's `impl Show for Deque<T: Show>` states the bound its body needs, so a deque whose
// element type has no `Show` does not print.
let q = Deque.of([(x: Int) => x + 1])
print(q) // <- here
