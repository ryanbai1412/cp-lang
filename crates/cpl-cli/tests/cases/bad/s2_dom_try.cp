// error: a `domain D` has no method `try`
// Synthesis #2: this spelling stays rejected (research/archive/architecture/language-architecture-synthesis-2.md, rejected list).
domain D = 1..=5
print(D.try(2) ?? D(1))
