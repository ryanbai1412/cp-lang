// expect: ok, output "0 0 0 0"
let s = Sieve(0)
print(s.phi_table()[0], s.mobius_table()[0], s.div_count_table()[0], s.div_sum_table()[0])
