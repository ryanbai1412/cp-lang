// expect: ok, output "YES YES"
print(yes(splitmix64(0) == 0xE220A8397B1DCDAF), yes(splitmix64(1) != splitmix64(2)))
