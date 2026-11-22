#!/usr/bin/env python3
"""Differential test of the blind-critic-3 ports against the original C++ submissions.

Usage (from anywhere): python3 proto/ports/blind_critic_3/check.py [name ...] [--iters N]
Builds scratch copies under a temp dir, so the tracked ports are never stamped.
"""
import os, random, resource, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
PROTO = os.path.dirname(os.path.dirname(HERE))
CORPUS = os.path.join(os.path.dirname(PROTO), "corpus", "submissions")
ORIGINAL = {'cco16p1': 4583882, 'coci21c2p5': 4143748, 'cheerio1s4': 5339437, 'aac5p5': 4240171, 'ioi09p6': 5106126, 'cmt21r1xym': 3997206, 'noip19p2': 5127828, 'noip22p1': 5213639}

def gen_cco16p1(it):
    n=random.randint(1,12 if it%5 else 60); k=random.randint(1,min(n,6))
    # each vertex degree<=2: build paths and cycles
    vs=list(range(1,n+1)); random.shuffle(vs); es=[]; i=0
    while i<n:
        s=random.randint(1,min(n-i,10)); comp=vs[i:i+s]; i+=s
        for a,b in zip(comp,comp[1:]): es.append((a,b))
        if s>=3 and random.random()<.5: es.append((comp[-1],comp[0]))
    random.shuffle(es)
    return f"{n} {len(es)} {k}\n"+"".join(f"{a} {b}\n" for a,b in es)

def gen_coci21c2p5(it):
    n=random.randint(1,8 if it%5 else 100); C=random.choice([6,20,10**9])
    out=[str(n)]
    for _ in range(n):
        l=random.randint(1,C); r=random.randint(l,min(C,l+random.choice([2,C])))
        out.append(f"{l} {r}")
    q=random.randint(1,10); out.append(str(q))
    for _ in range(q):
        l=random.randint(1,n); out.append(f"{l} {random.randint(l,n)}")
    return '\n'.join(out)+'\n'

def gen_cheerio1s4(it):
    n=random.randint(1,10 if it%5 else 200); m=random.randint(1,min(n,5 if it%3 else 30))
    return f"{n} {m}\n"+' '.join(str(random.randint(1,m)) for _ in range(n))+"\n"

def gen_aac5p5(it):
    n=random.randint(1,6 if it%5 else 25); m=random.randint(1,6 if it%5 else 25); C=random.choice([6,12])
    out=[f"{n} {m}"]
    for _ in range(n):
        a,b=sorted(random.sample(range(1,C+1),2)); out.append(f"{random.randint(1,C)} {a} {b}")
    for _ in range(m):
        a,b=sorted(random.sample(range(1,C+1),2)); out.append(f"{random.randint(1,C)} {a} {b}")
    return '\n'.join(out)+'\n'

def gen_ioi09p6(it):
    n=random.randint(2,6 if it%5 else 14); k=random.randint(1,4)
    g=[[random.choice('GGGGGT') for _ in range(n)] for _ in range(n)]
    cells=random.sample([(i,j) for i in range(n) for j in range(n)],min(n*n,2+random.randint(1,3)))
    g[cells[0][0]][cells[0][1]]='M'; g[cells[1][0]][cells[1][1]]='D'
    for x,y in cells[2:]: g[x][y]='H'
    return f"{n} {k}\n"+'\n'.join(''.join(r) for r in g)+'\n'

def gen_cmt21r1xym(it):
    n=random.randint(1,6 if it%5 else 40); p=random.choice([.2,.5,.8])
    return f"{n}\n"+''.join(''.join('1' if random.random()<p else '0' for _ in range(n))+'\n' for _ in range(n))

def gen_noip19p2(it):
    n=random.randint(1,10 if it%5 else 300)
    s=''.join(random.choice('()') for _ in range(n))
    return f"{n}\n{s}\n"+' '.join(str(random.randint(max(1,v-(3 if it%2 else v)),v-1)) for v in range(2,n+1))+"\n"

def gen_noip22p1(it):
    t=random.randint(1,3); out=[f"{t} 0"]
    for _ in range(t):
        n=random.randint(1,8 if it%5 else 30); m=random.randint(1,8 if it%5 else 30)
        out.append(f"{n} {m} {random.randint(0,1)} {random.randint(0,1)}")
        p=random.choice([.1,.3,.5])
        for i in range(n): out.append(''.join('1' if random.random()<p else '0' for _ in range(m)))
    return '\n'.join(out)+'\n'


def main():
    args = sys.argv[1:]
    iters = 300
    if "--iters" in args:
        i = args.index("--iters")
        iters = int(args[i + 1])
        args = args[:i] + args[i + 2:]
    # the ioi09p6 original keeps 8 MB arrays on the stack
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
    tmp = tempfile.mkdtemp(prefix="bc3_")
    bad = 0
    for name in args or ORIGINAL:
        orig = ORIGINAL[name]
        cp = os.path.join(tmp, name + ".cp")
        shutil.copy(os.path.join(HERE, name + ".cp"), cp)
        r = subprocess.run([sys.executable, os.path.join(PROTO, "cp.py"), "build", cp], capture_output=True, text=True)
        if r.returncode:
            print(r.stderr[-1500:]); bad += 1; continue
        ours, ref = os.path.join(tmp, name + "_bin"), os.path.join(tmp, name + "_ref")
        subprocess.run(["g++", "-std=c++17", "-O2", "-o", ours, os.path.join(tmp, name + ".cpp")], check=True)
        subprocess.run(["g++", "-std=c++17", "-O2", "-o", ref, os.path.join(CORPUS, f"{orig}.cpp")], check=True)
        gen = globals()["gen_" + name]
        for it in range(iters):
            random.seed(it)
            inp = gen(it)
            a = subprocess.run([ours], input=inp, capture_output=True, text=True, timeout=20)
            b = subprocess.run([ref], input=inp, capture_output=True, text=True, timeout=20)
            if a.returncode or a.stdout.split() != b.stdout.split():
                print(f"MISMATCH {name} it={it}\ninput:\n{inp[:600]}\nours:\n{a.stdout[:300]}{a.stderr[:300]}\nref:\n{b.stdout[:300]}")
                bad += 1
                break
        else:
            print(f"OK {name}: {iters} tests agree with {orig}.cpp")
    shutil.rmtree(tmp)
    sys.exit(1 if bad else 0)

if __name__ == "__main__":
    main()
