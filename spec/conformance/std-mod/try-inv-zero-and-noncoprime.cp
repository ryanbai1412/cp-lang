// expect: ok, output "NO YES"
print(yes(Mod<7>(0).try_inv() != None), yes(Mod<7>(3).try_inv() != None))
