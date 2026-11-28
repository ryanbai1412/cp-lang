// expect: E0505
// rule: R02.5.5
// engines: next
// `@on_destructure` says why a pattern can't take apart what a fn gives, so it sits on a fn.
@on_destructure("a pair is not {n} names") // <- here
struct P(a: Int, b: Int)
print(P(1, 2).a)
