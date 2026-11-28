// expect: ok, output "9 9"
// engines: next
// The values past the first two are a Tuple4; its max_with folds them into the max of the first two.
print(max(6, 2, 7, 9, 1, 3), max(3, 1, 9, 7, 2, 6))
