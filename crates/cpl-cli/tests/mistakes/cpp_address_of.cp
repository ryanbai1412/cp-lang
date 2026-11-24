// here: 4
// C's `&x` passed to a fn (self-run mistakes): a bare "expected a value after `(`".
var (a, b) = (1, 2)
swap(&a, &b)
print(a, b)
