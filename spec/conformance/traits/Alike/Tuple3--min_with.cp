// expect: ok, output "1 1"
// engines: next
// The values past the first two are a Tuple3; its min_with folds them into the min of the first two.
print(min(8, 3, 9, 1, 5), min(5, 1, 9, 3, 8))
