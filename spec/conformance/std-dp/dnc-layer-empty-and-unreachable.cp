// expect: ok, output "0 YES"
use std.dp.{dnc_layer}
let prev: List<?Int> = []
print(dnc_layer(prev, (j, i) => i - j).len, yes(dnc_layer([None, None], (j, i) => i - j) == [None, None]))
