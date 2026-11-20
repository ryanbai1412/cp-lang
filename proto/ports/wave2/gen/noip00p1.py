import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
print(r.randint(-32768, 32767) if r.random() < .7 else r.randint(-30, 30), -r.randint(2, 20))
