// error: bind it first
// Synthesis #2: this spelling stays rejected (research/language-architecture-synthesis-2.md, rejected list).
var o: ?(Int, Int) = (1, 2)
print(o?.0 ?? 0)
