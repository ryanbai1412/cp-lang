// Synthesis #2 rule 7: a list literal fills a [D]T one value per position, and `x!` on a var ?T is a place.
domain Small = 0..3
let primes: [Small]Int = [2, 3, 5]
print(primes[Small(0)], primes[Small(1)], primes[Small(2)], sum(primes))

struct St:
    h: [Small]Int
    k: Int
var state: ?St = None
state = St([Small] _ => 0, 1)
state!.h[Small(1)] = 9             // writes the state in place, not a copy of it
state!.k += 4
print(state!.h[Small(1)], state!.k, sum(state!.h))

var o: ?Int = 3
o! += 2
o! min= 9
print(o!)
o! = 11
var best: ?(Int, Int) = (1, 2)
best!.1 = 7
print(o!, best!.0, best!.1)

var ls: ?List<Int> = [1]
ls!.push(4)                        // a mutator reaches the list itself
ls![0] = 8
print(ls!.len, ls![0], ls![1])
