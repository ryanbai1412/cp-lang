import re,glob,collections
files=glob.glob('../corpus/**/*.cpp',recursive=True)
pats={
 'ref_lambda_stored': r'auto\s+\w+\s*=\s*\[&\]',
 'std_function': r'function\s*<',
 'recursive_lambda_self': r'auto\s*&&?\s*self|\[&\]\s*\(\s*auto\s*&&?\s*\w+\s*,',
 'struct_ref_member': r'struct\s+\w+\s*\{[^}]*?\b(vector<[^;]*>|int|ll|long long)\s*&\s*\w+\s*;',
 'struct_ptr_member': r'struct\s+\w+\s*\{[^}]*?\b\w+\s*\*\s*\w+\s*;',
 'two_ptr_merge': r'while\s*\(\s*\w+\s*<\s*\w+\s*&&\s*\w+\s*<\s*\w+\s*\)',
 'idx_plus1': r'\[\s*\w+\s*\+\s*1\s*\]',
 'idx_minus1': r'\[\s*\w+\s*-\s*1\s*\]',
 'prefix_build': r'\w+\[\s*\w+\s*\+\s*1\s*\]\s*=\s*\w+\[\s*\w+\s*\]\s*\+',
 'tin_tout': r'\btin\b|\btout\b|\bin\[|\bst\[',
 'compress': r'lower_bound\([^;]*\)\s*-\s*\w+\.begin\(\)',
 'define_int_ll': r'#define\s+int\s+long\s+long',
 'int128': r'__int128',
 'mod_mul': r'%\s*(MOD|mod|M)\b',
 'one_ll': r'1LL\s*\*|1ll\s*\*|\(ll\)|\(long long\)',
 'recursive_dfs': r'void\s+dfs\w*\s*\(',
 'global_arrays': r'^\s*(int|ll|long long|bool)\s+\w+\s*\[\s*\w+',
 'unsigned_bits': r'unsigned|ull|uint64|__builtin_popcount|__builtin_ctz|__builtin_clz',
 'double': r'\bdouble\b|long double',
 'string': r'\bstring\b',
 'vector_of_vector': r'vector\s*<\s*vector',
}
hit=collections.Counter(); tot=collections.Counter()
ex=collections.defaultdict(list)
for f in files:
    s=open(f,errors='ignore').read()
    for k,p in pats.items():
        m=re.findall(p,s,re.M|re.S)
        if m:
            hit[k]+=1; tot[k]+=len(m)
            if len(ex[k])<3: ex[k].append(f.split('/')[-1])
print('files',len(files))
for k in pats: print(f'{k:22} files={hit[k]:5} ({100*hit[k]/len(files):4.1f}%) occ={tot[k]:6}  e.g. {ex[k]}')
