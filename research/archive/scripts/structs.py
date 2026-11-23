import re,glob,collections
files=glob.glob('../corpus/**/*.cpp',recursive=True)
c=collections.Counter(); ex=collections.defaultdict(list); nfiles=collections.Counter()
for f in files:
    s=open(f,errors='ignore').read(); seen=set()
    for m in re.finditer(r'struct\s+(\w+)[^{;]*\{',s):
        name=m.group(1); body=s[m.end():m.end()+3000]
        # cut at matching top-level "};"
        d=1;i=0
        while i<len(body) and d>0:
            if body[i]=='{': d+=1
            elif body[i]=='}': d-=1
            i+=1
        body=body[:i]
        for mm in re.finditer(r'^\s*(const\s+)?([\w:<>, ]+?)\s*([*&])\s*(\w+)\s*(=[^;]*)?;',body,re.M):
            ty,k,fld=mm.group(2).strip(),mm.group(3),mm.group(4)
            if ty in('return','else'): continue
            selfref = ty==name or ty.startswith(name)
            cat=('self_ptr(tree/list node)' if selfref else 'ref_to_outside' if k=='&' else 'ptr_to_outside')
            c[cat]+=1
            if cat not in seen: nfiles[cat]+=1; seen.add(cat)
            if len(ex[cat])<5: ex[cat].append(f"{f.split('/')[-1]}:{name}.{fld}:{ty}{k}")
print('files',len(files))
for k,v in c.most_common(): print(f'{k:26} fields={v:4} files={nfiles[k]:4} ({100*nfiles[k]/len(files):.1f}%) {ex[k]}')
# lambdas: recursive = takes an auto self param
lc=collections.Counter(); mut=0; tot=0
for f in files:
    s=open(f,errors='ignore').read()
    for m in re.finditer(r'auto\s+(\w+)\s*=\s*\[&\]\s*\(([^)]*)\)',s):
        tot+=1; p=m.group(2)
        lc['self_param_recursive' if re.match(r'\s*(auto|const auto)\s*&*\s*\w+\s*(,|$)',p) else 'plain']+=1
print('lambdas',tot,dict(lc))
