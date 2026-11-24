// here: 4
// C's `printf("%d %d\n", a, b);` (self-run mistakes): "no `printf` in scope" with no fix.
let (a, b) = (1, 2)
printf("%d %lld\n", a, b);
