// expect: E0602
// input: "\n"
// Pushing to a list while walking it.
var log: List<Int> = [1, 2, 3]
var s = 0
for v in log:
    if log.len < 10: log.push(v)  // <- here
    s += v
print(s)
