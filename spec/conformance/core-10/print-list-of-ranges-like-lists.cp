// expect: ok, output "4 5\n7 8\n4 5\n7 8"
// rule: 10.3.1
// A range inside a list prints its elements, one inner sequence per line, exactly as the list of lists with the same elements does.
print([4..6, 7..9])
print([[4, 5], [7, 8]])
