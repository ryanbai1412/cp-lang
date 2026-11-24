// expect: ok, output "2 3 5 3 4 3 3"
// rule: std.seq.Sized
// `S: Sized` holds for a list, a string, a range, an array, a domain and the views over them
// through std's `impl Sized` declarations, and `s.len` reads each one's length.
fn n<S: Sized>(s: S) -> Int = s.len
domain R = 0..4
let a = [1, 2, 3]
print(n([1, 2]), n("abc"), n(0..5), n(a), n(R), n(a.rev), n(a.items))
