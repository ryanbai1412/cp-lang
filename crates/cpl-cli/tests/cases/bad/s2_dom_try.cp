// error: domain D has no `try`
// Synthesis #2: this spelling stays rejected (research/language-architecture-synthesis-2.md, rejected list).
domain D = 1..=5
print(D.try(2) ?? D(1))
