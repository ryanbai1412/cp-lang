// expect: E0409
// rule: R06.1.1
// A nested `fn` that gives a value gives one on every path, as a top-level one does: this
// `for` can end, and then `inner` has no value.
fn outer(k: Int) -> Int:
    fn inner(x: Int) -> Int:
        for i in 0..x: // <- here
            if i > 2: return i
    inner(k)
print(outer(5))
