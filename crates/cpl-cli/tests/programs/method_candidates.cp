// R11.3.4: `x.f(..)` takes the `fn f(self: ..)` whose `self` its receiver
// fits, from whichever module declares it: `count` and `split` are in both
// std.strings (on `Str`) and std.seq (on any sequence).
domain R = 0..3
domain C = 0..3
let p = (R(1), C(1))
print(p.step(Dir4.up)!)
let s = "a,b,ca"
print(s.count('a'), [1, 2, 3].count(x => x > 1))
print(s.split(',').len, [1, 0, 2, 0, 3].split(x => x == 0).count())
