// error: expected Int, got ?Int
// Synthesis #2: this spelling stays rejected (research/language-architecture-synthesis-2.md, rejected list).
var o: ?Int = 3
o += 2
let k: Int = o
print(k)
