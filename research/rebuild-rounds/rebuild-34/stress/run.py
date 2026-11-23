import sys, subprocess, random, os, probs
D=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CPL='cpl'
names=sys.argv[1:] or list(probs.P)
N=int(os.environ.get('N','200'))
def sh(cmd, inp=None, t=60):
    p=subprocess.run(cmd, input=inp, capture_output=True, text=True, timeout=t); return p
for k in names:
    g,b=probs.P[k]; f=f"{D}/{k}.cp"; cpp=f"/tmp/r34-{k}.cpp"
    p=sh([CPL,'build',f,'-o',cpp])
    if p.returncode: print(k,'BUILD FAIL',p.stderr[-800:]); continue
    p=sh(['g++','-std=c++17','-O2','-o',f'/tmp/r34-{k}.o2',cpp],t=300)
    if p.returncode: print(k,'G++ O2 FAIL',p.stderr[-800:]); continue
    p=sh(['g++','-std=c++17','-O1','-g','-fsanitize=address,undefined','-D_GLIBCXX_DEBUG','-o',f'/tmp/r34-{k}.san',cpp],t=300)
    if p.returncode: print(k,'G++ SAN FAIL',p.stderr[-800:]); continue
    bad=0; first=None
    for seed in range(N):
        r=random.Random(seed*7919+len(k)); s=g(r)
        open('/tmp/r34-in','w').write(s)
        oi=sh([CPL,'run',f,'/tmp/r34-in'])
        on=sh([f'/tmp/r34-{k}.o2'],s)
        os_=sh([f'/tmp/r34-{k}.san'],s)
        outs=[oi.stdout,on.stdout,os_.stdout]
        if b: exp=b(s); ok=all(o.split()==exp.split() for o in outs)
        else: ok=all(probs.c03(s,o) for o in outs) and oi.stdout.split()==on.stdout.split()
        ok=ok and oi.returncode==0 and on.returncode==0 and os_.returncode==0 and 'runtime error' not in os_.stderr and 'ERROR' not in os_.stderr
        if not ok:
            bad+=1
            if first is None: first=(s, outs, (b(s) if b else None), oi.stderr[-500:], os_.stderr[-500:])
    print(k, f"{N-bad}/{N} ok")
    if first: print('  FIRST FAIL input:',repr(first[0]),'\n  interp/native/san:',[o[:200] for o in first[1]],'\n  expected:',first[2],'\n  interp err:',first[3],'\n  san err:',first[4])
