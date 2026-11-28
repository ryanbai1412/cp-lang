// expect: ok, output "9 9"
// engines: next
// The values past the first two are a Tuple3; its max_with folds them into the max of the first two.
print(max(8, 3, 9, 1, 5), max(5, 1, 9, 3, 8))
