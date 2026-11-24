// expect: ok, output "1 0 1\n3 YES\n2"
// rule: std.seq.split std.seq.Sep
// One `split(sep:)` takes a value or a predicate through `Sep<T>` (decision 456): a list splits on an equal element, a `Str` on a `Char`, and each `Str` piece compares as a string.
print([p.len for p in [1, 0, 0, 2].split(0)])
let parts = "a,,b".split(',')
print(parts.count(), yes([Str(p) for p in parts] == ["a", "", "b"]))
print("x y".split(sep: ' ').count())
