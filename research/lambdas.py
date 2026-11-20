import re,glob,collections
files=glob.glob('../corpus/**/*.cpp',recursive=True)
c=collections.Counter(); ex=collections.defaultdict(list)
for f in files:
    s=open(f,errors='ignore').read()
    for m in re.finditer(r'auto\s+(\w+)\s*=\s*\[&\]\s*\(([^)]*)\)',s):
        name,params=m.group(1),m.group(2)
        rest=s[m.end():]
        body_end=rest.find('\n\t};') if '\n\t};' in rest else 2000
        after=rest[body_end:]
        kind=[]
        if re.search(r'\bauto\s*&&?\s*\w+\b',params) or name in rest[:body_end] and re.search(r'\b'+name+r'\s*\(',rest[:max(body_end,0)]): kind.append('recursive')
        if re.search(r'[(,]\s*'+name+r'\s*[,)]',after): kind.append('passed_as_arg')
        if re.search(r'return\s+'+name+r'\b',after): kind.append('returned')
        k='+'.join(kind) or 'local_call_only'
        c[k]+=1
        if len(ex[k])<4: ex[k].append(f.split('/')[-1]+':'+name)
tot=sum(c.values()); print('lambdas',tot)
for k,v in c.most_common(): print(f'{k:30} {v:5} {100*v/tot:5.1f}%  {ex[k]}')
