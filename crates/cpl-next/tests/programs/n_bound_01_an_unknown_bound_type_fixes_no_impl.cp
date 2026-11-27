// expect: ok, output "2\n1\n2\n1\n2 5 5 2"
// rule: R08.1.2
// `Rev<S: Bidir<T>>`'s `T` is fixed by the impl meeting `Bidir` for `S`, not by whichever impl of
// `Indexed` comes first while `S`'s source is still unknown (std.grid's rows read `Row`s).
use std.seq.{Rev}
let w = [1, 2, 3][0..2]
for x in w.rev: print(x)
for x in Rev(w).iter(): print(x)
let r = 2..8 by 3
print(r.len, r[5], r.rev)
