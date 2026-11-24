// expect: ok, output "4 1 5 3 7 YES 1 b 2 a"
// Acceptance for seq/U32 (min and max in one pass), handbook/product/std/sequences-usecases.md: the method folds the sequence; a plain call with two values orders the pair.
let (lo, hi) = [5, 1, 4].minmax()!
let e: List<Int> = []
let (a, b) = [(2, 'a'), (1, 'b'), (2, 'c')].minmax(by: p => p.0)!
print(hi - lo, lo, hi, minmax(7, 3), yes(e.minmax() == None), a, b)
