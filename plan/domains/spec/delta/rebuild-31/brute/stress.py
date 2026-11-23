from harness import *
import p01,p02,p03,p04,p05,p06,p07,p08,p09,p10
mods={'1898E':p01,'1858D':p02,'1866D':p03,'1881G':p04,'1881F':p05,'1879D':p06,'1866H':p07,'1878F':p08,'1898C':p09,'1891F':p10}
pat=sys.argv[1] if len(sys.argv)>1 else ''
for f in sorted(glob.glob(D+'/'+pat+'*.cp')):
    name=os.path.basename(f);pid=name.split('-')[1].split('.')[0];rev='rev' in name;m=mods[pid]
    exe,err=build(f,['-std=c++17','-O2'],'.o2')
    if not exe: print(name,'BUILD FAIL',err[:500],flush=True);continue
    r=random.Random(31);bad=0;first=None
    for it in range(200):
        inp=m.gen(r,True) if pid=='1891F' and rev else m.gen(r)
        o,e,rc=interp(f,inp);o2,e2,rc2=native(f,inp,exe)
        if pid=='1898C': ok=m.check(inp,o)
        elif pid=='1879D': ok=o.split()==m.solve(inp,rev).split()
        else: ok=o.split()==m.solve(inp).split()
        agree=o.split()==o2.split() and rc==rc2
        if not(ok and agree):
            bad+=1
            if first is None: first=(inp,o[:300],e[:300],o2[:300],ok,agree)
    print(name,'200 random: fails',bad,first if first else '',flush=True)
