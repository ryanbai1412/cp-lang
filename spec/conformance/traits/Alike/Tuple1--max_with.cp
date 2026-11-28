// expect: ok, output "9 9"
// engines: next
// The values past the first two are a Tuple1; its max_with folds them into the max of the first two.
print(max(4, 9, 2), max(2, 9, 4))
