// expect: ok, output "3"
// rule: 06.4.13
// `@laws(sample: n)` sets how many values stress samples for a type's laws; a run ignores it.
@laws(sample: 50)
struct P(a: Int)
print(P(3).a)
