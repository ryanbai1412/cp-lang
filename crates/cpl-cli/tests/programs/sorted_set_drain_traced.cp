// A traced `drain` on a SortedSet, whose `while:` closure the inner bag calls,
// finishes and prints what an untraced run prints.
var ss = SortedSet<Int>.of([1, 3, 5, 7, 9, 12])
let sd1 = ss.drain(after: 3, while: x => x <= 9)
print(sd1, [x for x in ss])
