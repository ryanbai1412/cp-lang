// expect: ok, output "1 1"
// engines: next
// The values past the first two are a Tuple4; its min_with folds them into the min of the first two.
print(min(6, 2, 7, 9, 1, 3), min(3, 1, 9, 7, 2, 6))
