// expect: ok, output "7"
// rule: 08.1.1
// Uncalled methods on a generic struct do not fail instantiation if their
// signatures reference trait or deferred types not needed by called methods.
trait Extra:
    fn tag(self) -> Int

struct Holder<T>:
    x: T

    pub fn get(self) -> T = x
    pub fn unused(self, e: Extra) -> Int = e.tag()

let h = Holder<Int>(7)
print(h.get())
