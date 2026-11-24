// expect: ok, output "YES YES"
print(yes(knuth(0, (i, j) => j - i) == [[0]]), yes(knuth(1, (i, j) => j - i) == [[0, 0], [0, 0]]))
