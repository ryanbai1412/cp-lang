// expect: ok, output "2 2"
// engines: next
// The values past the first two are a Tuple1; its min_with folds them into the min of the first two.
print(min(4, 9, 2), min(2, 9, 4))
