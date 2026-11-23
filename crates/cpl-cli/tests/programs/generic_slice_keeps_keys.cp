// A slice given for a generic `[K]T` keeps its source's keys, and so does
// that `[K]T` passed on to another generic fn (R04b.3.1).
fn top<K>(a: [K]Int) -> K = a.argmax()!
fn first_key<K>(a: [K]Int) -> K = a.keys.first!
fn best<K>(a: [K]Int) -> ?K:
    var at: ?K = None
    var high: ?Int = None
    for (k, x) in a.items:
        if high == None or high! < x:
            at = Some(k)
            high = Some(x)
    at
fn via<K>(a: [K]Int) -> K = best(a)!
let s = [1, 2, 3, 4, 9]
print(Int(top(s[2..])), Int(first_key(s[2..])), Int(best(s[1..4])!), Int(via(s[2..])))
print(Int(top(s)), Int(via(s)))
