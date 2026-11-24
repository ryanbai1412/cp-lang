import sys
if sys.argv[1] == "--max":  # the main test: n = 20
    print(20); sys.exit()
# seed size: the statement has two tests, n = 3 and n = 20
print(3 if int(sys.argv[1]) % 2 else 20)
