// A search given a predicate that flips the wrong way stops with its
// message on both engines; the message is a constant, built only then.
let which: Int = read()
let n: Int = read()
if which == 1: print((0..n).first_true(x => x < 3) ?? -1)
if which == 2: print((0..n).bsearch(x => x >= 3, last: true).r)
if which == 3: print((0..n).bsearch(x => x < 3).r)
print("fine")
