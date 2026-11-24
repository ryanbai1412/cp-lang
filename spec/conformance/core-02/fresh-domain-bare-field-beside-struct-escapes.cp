// expect: E0313
// rule: 02.9.2
// A struct in a returned record packs its domain, but a bare `[P]` field beside it still escapes.
struct S<V, P>:
    at: [P]V
fn mk<V>(r: V, k: Int) -> (s: S<V, _>, xs: List<Int>):
    domain P = 0..k
    (s: S([P] _ => r), xs: [P] p => p.ord) // <- here
