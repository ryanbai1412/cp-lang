// error: `o` is a `let`, so it can't be assigned
// Synthesis #2: this spelling stays rejected (research/archive/architecture/language-architecture-synthesis-2.md, rejected list).
let o: ?Int = 3
o! += 1
print(o!)
