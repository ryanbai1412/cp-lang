// expect: ok, output "9 9"
// engines: next
// The values past the first two are a Tuple2; its max_with folds them into the max of the first two.
print(max(4, 2, 9, 7), max(7, 9, 2, 4))
