# Friction report: coci25c3p5

Rethought lightly. The v0.2 port read n, a and b as untyped `Int`s and summed adjacent pairs with `i.prev!` behind an `i != Fan.first` guard. The v0.3 port types all three from the statement and takes each pair with `for r in Fan if let l = r.prev`, so there is no unwrap.

No language gap.

## Measurements

gen.py --max: port 0.08 s, accepted C++ 0.04 s, same output.
