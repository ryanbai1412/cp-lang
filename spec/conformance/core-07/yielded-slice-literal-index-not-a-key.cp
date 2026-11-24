// expect: E0305
// rule: 07.4.5
// A slice any fn yields keeps its parent's keys, as a run does: `h[0]` is not its first.
fn halves<S: Indexed>(self: S) -> yields Slice<S>:
    let m = self.len / 2
    yield self[self.keys.nth(0)..=self.keys.nth(m - 1)]
    yield self[self.keys.nth(m)..=self.keys.nth(self.len - 1)]
let xs = [5, 6, 7, 8]
for h in xs.halves():
    print(h[0]) // <- here
