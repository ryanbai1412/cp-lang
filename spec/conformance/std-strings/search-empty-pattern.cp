// expect: ok, output "0 1 -1"
print("ab".search("") ?? -1, "ab".rsearch("") ?? -1, "".search("a") ?? -1)
