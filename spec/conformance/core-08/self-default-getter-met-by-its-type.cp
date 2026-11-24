// expect: ok, output "3 4"
// rule: 08.1.1
// A trait whose getter defaults to `= self` is met by the type the getter returns, as std's
// `Target<V>` is met by a vertex `V` itself.
trait Tagged<T>:
    @get fn tag(self) -> T = self
fn tag_of<A: Tagged<Int>>(a: A) -> Int = a.tag
print(tag_of(3), tag_of((tag: 4)))
