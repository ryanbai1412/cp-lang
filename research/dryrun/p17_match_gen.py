import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2]); al = "ab"[:random.randint(1, 2)] + "c" * (random.random() < .2)
t = "".join(random.choice(al) for _ in range(random.randint(1, s))); print(t)
print("".join(random.choice(al) for _ in range(random.randint(1, min(4, s)))))
