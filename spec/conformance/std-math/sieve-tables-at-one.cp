// expect: ok, output "1 1 1 1"
let s = Sieve(1)
print(s.phi_table()[1], s.mobius_table()[1], s.div_count_table()[1], s.div_sum_table()[1])
