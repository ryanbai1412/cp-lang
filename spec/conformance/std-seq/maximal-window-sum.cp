// expect: ok, output "1 2 1 2"
// The spec's own `maximal(Window(+), ok:)` example: for each right end, the longest piece with sum at most 6 (rebuild round 33 F4, and rounds 29 to 32).
print(List([1, 2, 5, 1].maximal(Window(+), ok: w => (w.all ?? 0) <= 6).map(p => p.len)))
