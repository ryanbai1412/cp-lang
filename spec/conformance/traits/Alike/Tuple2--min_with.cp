// expect: ok, output "2 2"
// engines: next
// The values past the first two are a Tuple2; its min_with folds them into the min of the first two.
print(min(4, 2, 9, 7), min(7, 9, 2, 4))
