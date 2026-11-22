// expect: ok, output "YES"
domain B = 0..3
let o: ?Subset<B> = None
let s: ?Subset<B> = Subset<B>.full
print(yes(o == None and s! == Subset<B>.full))
