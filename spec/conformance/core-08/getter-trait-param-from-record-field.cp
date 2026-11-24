// expect: ok, output "7\nab"
// rule: 08.1.7
// A record meets a user trait's bodiless `@get fn` with a field, and that field's type binds the trait's parameter.
trait Scored<S>:
    @get fn score(self) -> S
fn first_score<A: Scored<S>, S>(xs: List<A>) -> S:
    xs[0].score
print(first_score([(name: "x", score: 7)]))
print(first_score([(score: "ab")]))
