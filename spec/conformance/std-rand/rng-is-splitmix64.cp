// expect: ok, output "YES"
use std.rand.{Rng}
var a = Rng(0)
var b = Rng(1)
var c = Rng(-1)
let ok = a.next() == 0xE220A8397B1DCDAF and a.next() == 0x6E789E6AA1B965F4 and b.next() == 10451216379200822465 and c.next() == 0xE4D971771B652C20
print(yes(ok))
