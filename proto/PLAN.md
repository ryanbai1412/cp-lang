# Prototype plan (cpc: Python compiler -> C++17)

Files: cpc/lexer.py, cpc/parser.py (done, generic N nodes), cpc/types.py (done),
cpc/compiler.py (check+emit in one pass, TODO), cpc/runtime.hpp, cp.py CLI.

Representation decisions
- Int = ll; domain values = C++ int holding the 0-based ORDINAL (label = ord + lo).
  Domain globals D_<name>_lo / D_<name>_len. Arrays [D]T = vector<T>; [A,B]T flat
  with stride; [A][B]T nested. V.bounds = cut domain, len+1, i.before=ord, i.after=ord+1.
- Anonymous count domains: [q]T, [0..=k]T. Indexed by Int -> runtime check unless
  proven (interval or loop fact with identical bound strings).
- Indexing [V]T with Int or another domain = compile error (suggest V(x) / `x in V`).
- `if x in V [and ...]`: x refined to V inside (cpp expr (int)(x - V_lo)); not if x assigned in body.
- i + k / i - k / V(x) / V.at(k): checked (chk_dom), recorded for `--checks`.
  .next/.prev -> ?V. i - j (same V) -> Int. Dom vs Int compare = error.
- ?T: Opt<T> struct; Opt<int> specialisation uses -1 niche (only domains are int).
  No `<` on ?T; ==/!= none; ??, !, if let (+ `and` extra via ok flag), opt min=/max=.
- Tuples/records: generated structs (names or f0..), with == and <.
- Top level: vars become globals; top-level stmts -> init(). C8 REMOVED (Ryan disliked): multitest = `@multitest fn solve()` decorator; fn locals are per test. No `tests:` header;
  top-level fns -> free functions (prototypes if ret declared); local fns ->
  `auto f = [&](auto&& f, args) -> R` called as f(f, ...). Implicit last-expr return.
- solution main -> end of solve(); other solutions (@brute) -> selected by -DSOLUTION=name.
- Value ranges: TInt(lo,hi); interval arithmetic; known bound beyond int64 = compile
  error naming the expression. Unknown -> local build uses -ftrapv. Vars: fixpoint over
  whole-program recompiles (<=5 passes): x = init + sum(trip_product * iv(e)) for +=,
  join for = / min= / max=; non-converging -> unknown. Declared range types on vars are
  trusted (asserted in LOCAL).
- Reductions sum/count/min/max/any/all over `e for x in it if c` -> IIFE loops.
  min/max return T if provably non-empty else ?T.
- Input read/print generated per type (domains read labels, checked; arrays loop).
  Range-typed input validated under LOCAL; `where` asserted under LOCAL.
- Gen mode: same input stmts emit a random generator (values in [lo, min(hi, lo+SIZE)]),
  where -> reject+retry; used by `cp stress` vs brute solution or the ORIGINAL C++.
- @memo (compiler-implemented stand-in for the stdlib decorator): dense table if all
  params are domains.
Milestones: t1.cp prefix sums -> overflow + index error demos -> adapt 10-15 rewrites
-> stress vs original C++ -> perf + size -> report to Ryan.

Progress (compiler.py): written so far = helpers, Sym, unparse, Compiler init, scopes,
cpp_type, tuple_type, domains (new_domain/domain_from_range/bounds_of), resolve_type,
dim_type, coerce, expr dispatch x_num/x_bool/x_char/x_str/x_paren/x_none/x_name/x_tuple/
x_list/x_unop/x_binop. DONE also: x_cmp, membership, x_range, x_unwrap, x_ifexpr, x_index, index_into. DONE x_field, x_call (min/max/abs/sorted/List<T>()/Int()/V(x)/Mod(x)/user fn calls; local fns get self arg), method_call (V.at/try_at, List push/pop/last/sort/rev/clear, i.offset). TODO: reduction(name, arg, e, want) over gen/iterables, x_call (builtins, V(x), reductions,
methods), x_table, x_gen, x_ifexpr, x_unwrap, x_range; statements (s_*), iteration helper,
print/read/gen emitters, fixpoint driver, compile() -> C++ text, cp.py CLI.
runtime.hpp needs cp_ipow.

runtime.hpp also needs cp_lidx(i, size, line) (checked list index).
runtime.hpp also needs cp_rfirst/cp_rlast(Rng, line).
runtime.hpp also needs cp_pop(vec, line).
DONE: bind_pattern, loop(pat, it, node, body_cb) (ranges/domains/Rng/arrays/lists, .rev), capture(f), gen_loops, as_gen, trips_of. NEXT: reduction(), x_table, x_gen, statements, emit print/read/gen, driver, cp.py.
DONE: reduction (sum/product/count/any/all/min/max with value ranges + nonempty proof), x_gen (error), x_table. NEXT: statements s_* (input/domain/let/assign w/ var_iv contributions/if/iflet/for/while/print/assert/return/fn/solution/tests/typedef), print+read+gen emitters, fixpoint driver compile(), cp.py CLI, runtime additions (cp_ipow, cp_lidx, cp_rfirst/rlast, cp_pop).

GENERALITY PASS (Ryan: no CP-specific core syntax). Parser now: no input/print/solution/
tests/output/local keywords. `let x: T = read()` (compiler: read() with declared type T
-> emit typed reader; gen mode -> generator). `let ... where cond` (general checked binding).
`print(...)` is a builtin fn call. `@solution fn main()` / `@brute(main, when: c) fn slow()`
/ `@multitest fn solve()` are decorators on plain fns. `pure fn` = C9 capture check.
`cfg.debug` replaces LOCAL. design-v2 Part 9 records this. t1.cp updated + parses.

STATUS 22:06: compiler.py end-to-end works (python3 cp.py checks f.cp -> f.cpp, g++).
t1.cp compiles, runs, 0 runtime checks. tests/bad/*.cp demo errors: Int index, cut vs elem,
?T ordering, wrong domain, product overflow, accumulator overflow (fixpoint). Gen mode:
compile with -DCP_GEN_MODE, run `./gen SEED SIZE` prints input; exit 4 = where rejected.
NEXT: stress driver (cp.py stress f.cp orig.cpp), port 10-15 rewrites, perf+size.
Known gaps: array element value ranges (declared Int arrays are unbounded), structs,
enums, match, generators, @memo, Map/Set, methods on ?T.
PORTS (ports/): mccc4j5 (MinHeap, [V]List<V>, while let, record read) stress OK 300, ours
0.09s vs orig 0.13s on n=2e5; nccc10s2 (char domain, Perm<L>, pairs(), @memo) stress OK 300.
Added: THeap, TPerm, while-let, pairs loop, @memo (top-level, domain params), List read.

22:08 Ryan: "go ahead and actually build out the language and transpiler and everything".
Sent examples-10.md (~/cplang/examples-10.md). Compiler = Python (told Ryan; Rust/OCaml later).
NEXT FEATURE QUEUE (drive by ports): apio07p1 (?D read -1, let-else, postfix `?`, tuple
returns, recursion with outer var), ceoi09p3 (compress -> fresh domain, sorted records,
upper_bound, 3-D table over bounds), ioi05p6 (Rooted, @memo 2 params, rows as arrays),
ccc23s4 (dijkstra, Graph read), dmopc21c5p4 (@multitest, Journaled/undo), structs/enums/
match, Map/Set, generators (yield) -> callbacks, ranges .split, BitSet, Mod<M>.
Each port: ports/<id>.cp + <id>_orig.cpp, `python3 cp.py stress ports/<id>.cp ports/<id>_orig.cpp 300 6`.
22:09 apio07p1 port stress OK (186 agree, 1314 rejected by where is_forest -> FRICTION: need
structured tree generators (design's Tree/Parents types) instead of rejection sampling).
Added: let-else (parser after_block flag), postfix `?` as whole let RHS (returns {} from ?T fn),
?D read (-1 = none; gen 50% none), x_tuple unwraps ?T want, no operator< on tuples with ?T
fields, last_read set after where (gen must run where before exiting).
3 ports OK: mccc4j5, nccc10s2, apio07p1. Next in queue: ceoi09p3.
22:09 Ryan asked about ADTs/enums: told him yes (in design since v0); prototype enums+match are NEXT after ceoi09p3. ceoi09p3 added: compress -> fresh domain (let (H, h) = compress(list)), arr.last_le/first_ge -> ?Dom, sorted(arr) keeps [D]T.
22:09 ceoi09p3 port stress OK 300 (fresh domain via compress, 3-D table over P.bounds x P.bounds x H,
nonzero flow fact after `if w == 0: return`). 4 ports OK: mccc4j5, nccc10s2, apio07p1, ceoi09p3.
NEXT: enums + match (Ryan asked), then ioi05p6, ccc23s4, dmopc21c5p4. Then perf/size table + report.
22:10 parser: enum (variants w/ named fields) + statement match (arms Name(binds) [if guard]: block, _ wildcard). Compiler TEnum + s_enum + s_match + enum read/construct TODO; test tests/enum1.cp.
22:10 enums+match DONE (TEnum tagged struct, ctors, read tag+fields, exhaustive match w/ guards,
tests/enum1.cp OK, tests/bad/nonexhaustive.cp errors). Ryan: ports must be REAL rewrites (rethink
logic; C++ ints-as-codes/sentinels/flags -> enums/optionals/domains), not translations. Redo
apio07p1 with `enum End: Toy | Rod(Bar)`; all future ports free-style, still stress-checked.
22:10 Memory updated (rewrite-not-translate, general core, fair line counts). Decided apio07p1
keeps ?Bar (genuinely right type). NEXT port ccc23s4 (free style): needs builtin
dijkstra(adj: [V]List<(V, Int)>, src: V) -> [V]?Int, a.sorted_by(|r| key) (lambda -> C++ comparator),
`inout` params on fns pushing into arrays-of-lists, record types. Original: hillclimb/r1/src/ccc23s4.cpp.
Then ioi05p6, dmopc21c5p4 (@multitest). Then perf/size table + report to Ryan.
22:11 ccc23s4 stress OK 300: added struct (named tuple), builtin dijkstra([D]List<(D,Int)>, src)->[D]?Int (runtime cp_dijkstra), arr.sorted_by(|x| key) (stable), inf (CP_INF). 5 ports OK. NOTE: user-visible msg promised to redo the 4 earlier ports around logic: review them (apio07p1 ?Bar kept - explain).
22:12 ioi05p6 stress OK (87 agree / 313 rejected @400). Added: WithMin<D> (domain lo-1..=hi, .bottom, .elem -> ?D, implicit D->WithMin<D> in coerce+index), predeclare_later (mutual recursion between top-level fns via C++ prototypes). 6 ports OK. Next: dmopc21c5p4 (@multitest), then perf/size table + report.
22:13 bench.py written (ports table). Found: ceoi09p3 output differs on large random input (-1 size); ioi05p6 0.17s vs 0.04s. Told Ryan memo scoping rule: memo table lifetime = declaring scope (inside solve => per test), top-level memo reading per-test state rejected; prototype bug: memo only resets on size change. TODO: implement @multitest + local @memo + fix.
22:13 ceoi09p3 'mismatch' = original's assert/int overflow outside real constraints (CEOI09 A,x,y<=3e5); tightened port bounds, stress OK. ioi05p6 rewritten: @memo cost(v, mill) -> Row ([Mills]Int) + spread -> Row (memo returning arrays works). TODO: duplicate  name silently ignored -> make it an error.
22:14 @multitest DONE: T read (gen 1..3), solve body at_top globals, gen-mode stops after last read per test, @memo inside @multitest -> table .clear() at declaration (fixes stale-memo bug for same-size tests); tests/ok/mt_memo.cp verified (12/30/28). Duplicate type names error. run_all.sh = regression (ports stress + bad tests). TODO: reject top-level @memo reading per-test state.
22:15 offset.cp moved to tests/ok (runtime-checked + reported by checks, by design). run_all green.
22:16 generators DONE: fn f() -> yields T, yield x, yield from g(), for over generator call -> template<class F> bool f(..., F&& _cb) + [&] callback; break/continue/return across callbacks via loop_ctx + (hr, rv) flags; works in reductions (sum). tests/ok/gen1.cp verified by hand. TODO: all/any/first reductions over generators (early-return inside lambda) untested; local generators unsupported. NEXT: Map/Set, then more ports.
22:17 Map<K,V>/Set<T> (std::map/set): [] or Map<K,V>() ctor, m[k]=v inserts, m[k] read/op= checked (cp_mget), k in m, m.get(k)->?V, s.insert/remove/min/max, .len, for (k, v) in m / for x in s. test tests/ok/map1.cp.
22:17 noip09p2 port stress OK (multitest+generators+yield from+sieve in-language; 16 agree/384 where-rejected @size30 -> needs custom Gen). Ryan asked about IIFEs -> proposed `do:` block expression (value = last expr, `?` exits innermost do/fn). Implementing in prototype as [&]()->T{...}().
22:18 do: blocks DONE (x_doblock -> [&]()->T{...}(), ? exits block, return inside do rejected, tail if/else implicit returns via tail_returns, inline 'if c: a else: b' one-liner statements parse). tests/ok/do1.cp -> 3,-1,-1 ✓
22:18 do: statement form for let RHS: block + goto end label; tail assigns; ? targets nearest do (same fn depth); return/break/continue behave normally (tests/ok/do2.cp verified). Expression-position do still lambda (return rejected there). run_all green incl. noip09p2 (7 ports).
=== 22:19 CHECKPOINT ===
7 ports stress-OK (apio07p1, ccc23s4, ceoi09p3, ioi05p6, mccc4j5, nccc10s2, noip09p2); run_all.sh green.
Done this stretch: let-else, ?, enums+match, structs, dijkstra, sorted_by, WithMin, mutual recursion,
memo->arrays, @multitest(+memo per test), generators, Map/Set, do: blocks (design-v2 Part 10).
Told Ryan: do:-in-expression-position still lambda (return rejected there).
NEXT (priority): (1) structured generation to cut where-rejection: Gen for parent arrays/trees
(e.g. `Tree<V>` read type or `@gen` hook) + custom Gen for noip09p2; (2) port with enum Op + match
from corpus (query problems); (3) top-level @memo reading per-test state -> compile error;
(4) all/any/first over generators; (5) bench table refresh + report to Ryan.
22:19 added rand(lo, hi) builtin and read(gen: f) (gen mode: target = f(), printed with emit_value). NEXT: check emit_value prints ?D as -1 and tuples/arrays in read format; then gen fns for apio07p1 (random_mobile), ioi05p6, noip09p2.
22:19 read(gen: f) works: apio07p1 now 300/300 agree, 0 rejected (random_mobile gen fn in-language, rand()). is_read accepts kwargs. NEXT: gen fns for ioi05p6 (random parents) and noip09p2 (planted query), then enum Op port, then report.
22:20 gen fns: ioi05p6 (random_river table, Site(rand(..))) and noip09p2 (struct Query + plant() with in-language gcd) -> both 300/300 agree, 0 rejected. All 7 ports now 0% rejection except where none. NEXT: enum Op + match port from corpus; top-level memo per-test-state error; all/any over generators; bench refresh + report to Ryan.
22:20 enum-Op candidates in hillclimb srcs: nccc10s3 (227 lines), noi19p5 (249) - both big; consider searching full corpus (~/cplang/submissions or attachments zip, info.json) for small accepted query problems.
22:21 noi02p1 port (Galactic Heroes, weighted DSU): @tags("M","C") enum Op + match, read(gen: random_ops) table of enum ctors, (Ship, Int) tuple-returning recursive locate w/ global mutation -> 300/300 agree. Added: @tags on enums (word tags read/gen), emit_value for enums, CPP_RESERVED += std names (size etc. clashed). 8 ports.
22:22 bench refreshed for 8 ports (added large-input gens for noi02p1, noip09p2; refs built with -DNDEBUG since ceoi09p3 orig's assert overflows int on max input; ours matches NDEBUG build). noi02p1 mem 15MB vs 4MB: we store all ops (enum struct w/ all payload fields) vs original's online read. Idea: enum payload union/shared storage. NEXT: report to Ryan; then top-level memo per-test check; all/any over generators; expression-position do with return.
22:23 memo staleness check DONE: @memo fn reading a var that is written (assign / mutating method / inout arg) on a later line -> compile error (check_memo_staleness at end of program). Covers top-level memo + per-test state written inside @multitest solve. tests/bad/memo_var.cp, memo_mt.cp. Ports still compile (kids/ahead built before memo decl).
22:24 FIXED any/all over generators (was: early return inside callback = continue -> wrong answers). Reduction lambda now acts as fn ctx (fn_ret BOOL, fresh loop_ctx) so exits go through hr/rv flags. tests/ok/gen2.cp: 36 -> YES NO 9 18 ✓, 12 -> NO NO 6 6 ✓.
22:24 ccc21s2 port (Modern Art): @tags R/C enum Stroke{Across(r: Row), Down(c: Col)} + count over domains -> 300/300. 9 ports. NEXT: enum payload union storage (noi02p1 mem), then more ports.
22:25 enum union storage: all-trivial payloads -> anonymous structs in union (GNU ext), memset ctor, tag-aware ==. noi02p1 mem 15->12MB (orig 4MB; rest = stored ops + input buffer; fix = streaming reads e.g. generator yielding read() per op — read() only works in let today).
22:26 INTERRUPTED: noi02p1 streaming variant (ops() generator w/ read inside fn) breaks gen mode: gen binary exits at last top-level read (render: gen_exit after init). Fix plan: in gen mode run whole program, wrap user print emission in if(!CP_GEN), drop gen_exit split. Stored version at /tmp/noi02p1_stored.cp (currently ports/noi02p1.cp = streaming, stress FAILS -> restore or fix before run_all). Ryan asked (17:25): compiler version stamps (doing), @memo control (proposed @memo(into: dp), asked him), per-part testing (plan: per-fn @brute/stress + port 4-part solution ioi18p3/noi20p3).
22:27 version stamps DONE (local git repo ~/cplang, commit 8ef64b9): cp.py version; build prepends '// generated by cp 0.1 (rev)' and stamps source '// cp: 0.1 rev' on first build, notes mismatch later; --compiler REV runs git-archived compiler from ~/.cache/cp/REV. Gen mode fix: prints wrapped in if(!CP_GEN); if reads occur inside fns, gen mode runs whole program (no early exit). noi02p1 streaming (ops() generator) 300/300 OK; mem 6MB vs 4MB.
22:28 per-fn @brute DONE: '@brute(f) fn g(same params) -> same ret' (params non-empty => fn-level, else solution brute). 'cp.py fntest file.cp [seeds] [size]': builds -DCP_GEN_MODE -DCP_FNTEST, runs whole program on generated input (prints muted), then 1000 random calls per pair (gen_expr: Int ranges, domains, Bool, ?T, tuples, [D]T, List), prints args + both results on mismatch. tests/fn/range_sum.cp OK; range_sum_bug.cp caught (l=1 r=1: 0 vs -100). Told Ryan what inout is. NEXT: port a 3-4 part solution with per-part brutes.
22:28 multi-part port candidates (unused, 2022+, 100-140 lines, many fns): dmopc21c9p4 4565958.cpp, noi01p5 5079272.cpp, apio23p1 5593318.cpp, cco16p2 5566900.cpp, btoi19p3 4456164.cpp, ceoi18p5 5323876.cpp (corpus/submissions). Goal: rewrite as 3-4 pure fns each with @brute, run cp fntest + stress, show Ryan.
22:29 dmopc21c9p4 port: 4 parts (peel 1-D game w/ @brute peel_slow, row parities, per-row peel w/ inversion, verify+output). cp fntest OK (10 inputs x 1000 calls); stress 300/300. Fixes: gen_expr vector<bool> proxy (index loop); emit_value of Char arrays contiguous, 2-D arrays one row per line. 10 ports.
22:30 Ryan (17:29): worried about perf; wants compact subsets (u32/u64) + language-wide perf audit. Told him plan: (1) audit table per abstraction (repr + microbench vs hand C++) -> ~/cplang/perf-audit.md, (2) fix worst: Subset<D> bitmask (u32/u64 when D.len<=64 provable, BitSet else), domain-keyed Set/Map as vectors/hash, (3) bench.py flag >1.2x slower. Also sent rest of cut-off msg (dmopc21c9p4 4 parts, 0.04/0.04s).
22:31 sent Ryan dmopc21c9p4 port + orig as attachments; flagged awkward: no generics (peel takes List<Bool> + Int idx instead of [D]Bool), no list comprehensions. Perf microbench harness: perf/run.py (opt, set, gen, memo .cp vs *_hand.cpp).
22:31 Ryan asked: markdown doc with dmopc21c9p4 port+orig side by side; noted port is longer (brute ~20 lines, comments, loop-built lists). Doc -> ~/cplang/dmopc21c9p4.md incl. line counts + comprehension version. Perf fixes in progress: Opt<ll> LLONG_MIN niche, DenseSet for Set<small range|domain>.
22:32 PERF FIXES: Opt<ll> niche (LLONG_MIN sentinel, fail if stored) -> opt bench 0.20->0.15s, 156->80MB (= hand); memo 141->72MB. DenseSet (bitset) for Set<D> / Set<small Int range> (dense_set/set_ctor in compiler; runtime DenseSet) -> set bench 2.61->0.04s (= hand). perf/run.py table. Sent Ryan ~/cplang/dmopc21c9p4.md (side-by-side + sizes + generics/comprehension sketch). Generic arg parse: Set<0..=N> fails ('expected a type') -> use type alias. NEXT: run_all + commit; Subset<D> u32/u64 bitmask; perf-audit.md doc; bench >1.2x flag; generics; list comprehensions.
22:33 run_all green (10 ports) after perf fixes; committed aab2e10. Starting Subset<D> (u32/u64 bitmask when D.len<=64 provable from range type, else BitSet).
22:34 Subset DONE (prototype): Set<D> with D.len<=64 provable (len_iv[1]) -> Mask64 (uint64 word; insert/erase/count/popcount/ctz iteration/min/max). tests/ok/subset1.cp ✓ (3 16 2 5). Spelling is Set<D> (no separate Subset type). NEXT: |,&,submasks() ops on Mask64 sets, perf-audit.md doc, bench 1.2x flag, generics, list comprehensions.
22:35 set ops | & - for Mask64 sets (x_binop). perf-audit.md written at ~/cplang/perf-audit.md (table per abstraction). tests/ok/subset2.cp
22:36 set ops tested (4 1 2), run_all green. bench.py: SLOW list, warns on stderr + exit 1 if ours > 1.2x orig + 0.02s.
22:37 runtime tree-shaking: '// @part NAME ... // @end' blocks in runtime.hpp dropped unless NAME used (render wraps _render). mccc4j5 12.8->10.0KB. NEXT: generics fn f<D>, list comprehensions.
22:39 generics (prototype): top-level 'fn f<D>(...)' monomorphized per domain binding (unify param type AST vs arg types; specialize(g, bind) clones node, compiles in [globals, {D: domain}] scope; cache self.specs). @brute(f) on generic h: specialized with same binds whenever f is, registered in fn_brutes. tests/ok/generic1.cp ✓ (1 2 / 4). Rewriting dmopc21c9p4 with peel<D>.
22:41 dmopc21c9p4 now uses generic peel<D> + generic @brute peel_slow; fntest checks Row and Col instantiations; stress 300/300; bench 0.02 vs 0.03s, 8/8MB, 19.3KB. Size: orig 69 lines/1231B; port 52/1636; w/o brute 35/1118. dmopc21c9p4.md regenerated. NEXT: send Ryan update; list comprehensions; nicer fntest names; Map perf (domain keys); mccc4j5 mem 17 vs 10.
22:43 list comprehensions: [e for x in it (for/if/let clauses)] -> reduction('List') builds vector in lambda (x_list routes single gen item). tests/ok/listcomp1.cp ✓ (2 5 2 / 1 2 / 3 5). dmopc21c9p4 Part 3 now one comprehension.
22:45 run_all green; dmopc21c9p4 49 lines/1580B (32/1062 w/o brute); bench 0.03 vs 0.05s. md doc refreshed. NEXT: DenseMap for Map<D, V> (domain keys), check map iteration emission; mccc4j5 mem (range-typed field narrowing to int32 = open, overflow-risky).
22:47 DenseMap<V> for Map<D, V> (domain keys): vector slots + has flags, std::map-like API (find/->second, [], count, erase, iteration Ent{first, second&}). tests/ok/map2.cp ✓.
22:49 map microbench (domain keys) 0.28 vs 0.19s, 89 vs 80MB; perf-audit.md updated (DenseMap row, open: hash map for other keys, 32-bit storage of range-typed fields).
22:51 @memo(into: dp) DONE: exposes memo table as var dp: [params...]?T (same flat layout as arrays); sized at declaration so it can be prefilled/inspected/cleared. tests/ok/memo_into.cp ✓ (509 / 10 203 / 6 1 1). Still TODO from memo-control proposal: layout (hash/rolling), order (ascending/iterative), key selection.
22:53 @memo(order: ascending): on first call fills whole table in row-major order (nested loops calling wrapper) -> no deep recursion when deps are smaller keys; {tab}_filled flag reset at declaration (multitest-safe). tests/ok/memo_asc.cp (n=1e6 chain).
22:55 told Ryan memo into+ascending. NEXT: memo layout (hash for non-domain keys / rolling), key selection; Map<Int,...> hash; 32-bit range storage; more multi-part ports.
22:57 design-v2 Part 11 added (generics, per-fn brute, memo control, representation rules, listcomp, version stamps). @memo(hash) implemented (unordered_map<array<ll,K>> + CpArrHash part); auto when params not all domains.
22:58 run_all green after memo(hash); committed 1f1a960. Next: another port (looking at noi01p5 5079272.cpp).
23:01 HashMap<K,V> (unordered_map + CpHash; Map stays ordered - assumption). noi01p5 port (meet in middle) stress 200/200; 11 ports. Answered Ryan: @stage = pure fn + @brute now.
23:02 corrected Ryan: pure fn IS enforced (tests/bad/pure_global.cp added). noi01p5 bench 0.06 vs 0.04s, 25 vs 35MB. Asked if Map should hash by default (kept ordered). NEXT: more multi-part ports; 32-bit range storage; memo key selection/rolling.
23:04 32-bit storage: [D]T with T range-typed fitting int32 -> vector<int>; reads wrapped (ll) unless lvalue (narrow() helper; x_index). All ports compile; testing. Backup of pre-change compiler at /tmp/compiler_backup.py.
23:07 32-bit storage passes run_all (11 ports, 30 iters). Sent Ryan step-change ideas: 1 auto failure localization (shrink + swap each @brute part), 2 complexity warnings from range types, 3 types say what/compiler picks how + cp explain, 4 domain-generic stdlib, 5 invariant/covariant tests w/o brute, 6 compiler stream-instead-of-store. Starting 1 (default) unless Ryan redirects.
23:09 cp locate DONE: 'cp.py locate file.cp ref.cpp [seeds] [max_size]' finds smallest failing generated input (size 1.., seeds), then rebuilds with -DCP_SWAP_<fast> per @brute pair (compiler add_swap_hooks injects forward to brute) and blames the part whose swap fixes it. Demo: dmopc21c9p4 with ready.max bug -> 2x3 grid, blame peel_D2_Col.
23:10 run_all green, committed 3bb81df; told Ryan about cp locate demo. NEXT: step change 2 = complexity warnings: estimate op counts per loop nest from domain len_iv / range bounds (trips_of exists for gens), warn when > ~1e9 at max sizes; 'cp checks' or build warning.
23:12 complexity warnings: s_for keeps trip_stack product (trips_of on iterable), reductions multiply; check_cost warns > 1e9 at max declared sizes; cp.py build prints 'warning: line N: ... ~4.0e10 times'. tests/ok/cost1.cp.
23:13 no false-positive cost warnings on 11 ports. Limitations: loops over Lists/while count as 1, fn-call costs not multiplied. NEXT after commit: message Ryan; then step change 5 (@invariant tests) or 3 (cp explain).
23:14 BUG found by run_all: trips_of in s_for evaluated loop iterables like perm.pairs (only valid in loop context) -> CompileError; wrapped in try/except -> None. Previous commit f2828dc had the bug; fix being verified.
23:15 fix verified, run_all green. Committing; then message Ryan re complexity warnings. NEXT: step change 5 (@invariant/@covariant tests w/o brute) or 3 (cp explain).
23:16 told Ryan complexity warnings. Starting step change 5: @invariant(rev|shuffle) on fns with array/list params, checked by cp fntest (f(args) == f(transformed args)); registered in self.fn_props, emitted in emit_fn_tests.
23:17 @invariant(rev|shuffle) DONE: fntest checks f(args)==f(transformed); tests/fn/invariant.cp (max_pair_sum ok under rev; first_minus_last fails under shuffle as intended). NEXT: commit, tell Ryan; then step 3 (cp explain) / port more.
23:18 run_all green, committed fa7a8ec. Told Ryan @invariant. NEXT: cp explain (list representation choices per var: Mask64/DenseSet/DenseMap/unordered_map/std::set/int32 storage/?Int niche), then more ports (40-50 target).
23:19 cp explain DONE (declare() records reprs; 'cp.py explain file.cp' prints line/name/type -> C++ repr). NEXT: more ports toward 40-50; streaming input (noi02p1 memory); mccc4j5 memory gap.
23:21 mccc4j5 gap analysed: deps kept + users = graph twice; record field ll. Logged in perf-audit.md as open (field narrowing, free after last use, CSR). NEXT: more ports.
23:23 record field narrowing in progress: tuple struct fields with 32-bit range -> I32 wrapper (runtime @part I32, implicit from/to ll, +=/-=); x_field wraps (ll) unless lvalue. Checking all ports/tests compile.
23:25 Ryan: memory very sensitive (4 vs 8 bytes matters for cache). Plan: 32-bit storage everywhere range fits (arrays done, record fields testing), then memo tables, List/heap elems, 16/8-bit; per-port bytes/elem audit vs original.
23:27 committed 488460e (I32 record fields; mccc4j5 still 17MB: gap is duplicated graph). Now: List<T> narrow elems (cpp_type TList + cp_lidx read wrap).
23:28 List narrow elems: all ports/tests compile; running run_all.
23:30 Ryan: wants stylistic decisions doc - each decision with proposals, he picks. Writing ~/cplang/style-decisions.md (options on same snippet + rec; reply '1B 2A').
23:33 committed List narrowing; wrote style-decisions.md (20 decisions).
23:34 sent style doc to Ryan (await picks; default = rec). NEXT: memo table narrowing ([D]?T where T fits int32 -> Opt niche int), heap elems, bytes/elem audit in cp explain.
23:37 OptI32 for ?T narrow inner; compile check running. Answered Ryan on blocks: indentation + inline {} for expression blocks (lambdas).
23:39 OptI32 green (tests/ok/opt32.cp), run_all green. Ryan DECIDED style 1: indentation + inline { ... } expression blocks (stmts separated by ';', last expr is value). Implementing in parser.
23:40 answered Ryan on bindings: keep let/var (A). Next: inline {} blocks in parser.
23:42 inline { s; s; expr } -> doblock in parser primary; test tests/ok/brace1.cp.
23:44 answered Ryan on functions: fn ... -> R: plus one-line fn f(x) -> T = expr; explicit return types on named fns. Fixing do-block type inference for let x = { ... }.
23:46 brace blocks + infer_block_type (probe compile, restore funcs/struct_defs/var_iv) work: tests/ok/brace1.cp -> 5 1 3 7. Answered Ryan arrays: keep [V]Int.
23:48 answered Ryan niche opts: done for ?V/?narrow/?Int/memo tables; planned ?Bool/?Char 1B, enums spare tag, tuples/records via field niche, ??V, flag fallback; cp explain shows. TODO implement those.
23:49 committed 9a005b0 (brace blocks). Now Opt<bool> 1-byte specialization + tests/ok/optbool.cp.
23:50 Ryan asked about domains; sent overview. Opt<bool> 1B; run_all pending. NEXT: enum niche Opt<E> via tag=-1 (struct at compiler.py ~1866).
23:52 sent Ryan domain syntax reference + open choices (keyword, V.at vs V(i), .ord vs .idx). ?Bool committed 2fe0c49. Implementing Opt<Enum> niche (tag=-1).
23:53 Opt<Enum> niche works (tests/ok/optenum.cp: 3 2). run_all+commit running. NEXT: tuple niche ?(V, Int) via first niche-able field; then back to ports.
23:54 answered decision 7 (domain keyword, A). Rewrite told Ryan I'd keep V.at(i) and v.idx -> TODO add .idx (alias of .ord) unless he objects.
23:56 added v.idx (ord alias); sed'd ports/tests to .idx; run_all+commit running. NEXT: tuple niche, then more ports (40-50 target).
23:58 Ryan asked why not C (list-comp style tables): explained type-flip issues; proposed C allowed when declared type is [V]T (bidirectional). Asked keep A as short form or C+annotation only. Pending answer.
23:59 answered Ryan V.rev in tables (order irrelevant; lazy suffix DP; v.mirror proposed TODO; list for reversed seq). CHECK my V.at(n+1-v.idx) example correctness.
00:01 FOUND: V.at(k) was 0-based position (footgun). Changing: V.at(x) = domain value (x - lo), V.nth(k) = 0-based, try_ variants, v.mirror. memo_into.cp switched to nth. Told Ryan.
00:03 sent Ryan final array-comprehension position: keep A ([V] v =>) + list comps; C allowed when declared [V]T (TODO implement in list-comp handler); [V] => 0 fill; pattern index.
00:05 committed b2d9838 (at/nth/mirror). Implementing C+annotation: x_list rewrites [f(x) for x in D] to table when want is [D]T. Test tests/ok/listtable.cp.
00:07 Ryan DECIDED style 8: A + C-with-[V]T-annotation. Sent take on 9 (generics): A <D>, optional <D> for single-capital-letter params. Fixing s_let: declared [D]T + list-comp value is passed elem want (fill path).
00:08 s_let: declared [D]T with list-comp value -> expr(value, ty) (table rewrite in x_list). Testing listtable.cp.
00:10 Ryan DECIDED style 9 (A + implicit single-capital-letter generics; TODO implement). Answered 10: and/or/not (aliases possible). Decided: 1, 8, 9.
00:12 implicit generics: s_fn -> implicit_tparams (undeclared single capital letters in param types). Test tests/ok/implicit_gen.cp. committed 88018bf before.
00:14 answered Ryan decorators: @memo (A), weak pref; C rejected (looks core). Style decided: 1, 8, 9; takes sent on 2,3,4,5,7,10,11.
00:16 Ryan asked => vs ':'; I recommended => (switching parser; ':' accepted w/ warning during migration; convert ports/tests). Style decided: 1, 8, 9, 12(=>, pending objection).
00:18 match arms: parser accepts '=>' (and ':'); converted ports/tests arms via regex. Verify with run_all, commit.
00:19 Ryan DECIDED 12: =>. Decided: 1, 8, 9, 12. Ports/tests converted; running run_all.
00:21 sent Ryan copy/ref/value plan: value semantics; params borrow (mut exclusive); returns/last-use moved; OPEN big-value copy: A explicit .copy() (rec) vs B implicit+warning. Pending picks: copy A/B, mutation spelling A-D (default A + mut). Match arms => converted; run_all pending -> commit.
00:22 committed match => change. Awaiting Ryan: copy A/B, mutation spelling (default explicit .copy() + mut).
00:23 adding 'mut' param mode alias of inout (parser.py ~340: at('inout') or at('ref'); lexer KEYWORDS line 20; compiler checks p.mode == 'inout').
00:24 mut param alias added + tests/ok/mut1.cp; committed if green. Call-site marker not yet enforced. Awaiting Ryan copy/mut picks.
00:26 Ryan DECIDED 15: mut at param + call site. Implementing call-site 'mut' (required for mut params, rejected otherwise). Decided: 1,8,9,12,15. Copies: default explicit .copy() (not yet confirmed).
00:28 answered Ryan multi-dim: [Row,Col]T, g[r,c], flat row-major, Cell domain neighbors4, g.row(r), g.t, [I,J,K]; rec only g[r,c] (decision 16). VERIFY prototype 2D storage is flat.
00:30 call-site mut implemented (parser marks arg.mut; compiler errors both ways); ccc23s4 link(mut g,...) updated; tests/bad/mut_missing.cp added. 2D storage verified flat (compiler.py ~737). run_all pending -> commit.
00:31 run_all green w/ call-site mut; ccc23s4 signature -> mut; committing. NEXT: explicit .copy() rule (pending Ryan), decision 16 g[r,c]-only (pending), design-v2 update.
00:33 Ryan DECIDED 16: g[r,c] only (TODO: g[r] error -> g.row(r), add .row). Answered output (17): print + interpolation; VERIFY claims (buffered, domain prints value, print ?T error, print Bool).
00:34 answered f-string q: all literals interpolate (no f prefix), {{ escapes, {x:.9} specs, typecheck ?T inside. Corrected ternary. Verified: print(true) prints YES, print(?Int) compiles -> both to become errors (TODO). Pending Ryan: 17 output style, copies.
00:35 grid partial index error message -> suggests .row(...) (.row itself TODO). tests/bad/grid_partial.cp.
00:36 Ryan DECIDED 17: print + all literals interpolate, r"..." raw (TODO implement interpolation, print Bool/?T errors). Decided: 1,8,9,12,15,16,17.
00:38 implementing interpolation: lexer r".." raw (RSTR), parser STR with {..} -> N('interp', parts), print expands interp parts with sep ''. Look at _emit_print_body compiler.py:2770.
00:39 sent Ryan struct design: struct Edge: fields+defaults+methods (self / mut self), one-line form, auto memberwise ctor (labels or positional), static fn ctors Edge.read(), 'e with w: 7', derived ==/order/hash, destructuring, generic structs, nominal vs anon records. Open: receiver mut marker (lean no); methods in body vs impl (rec body). Checking prototype support.
00:40 prototype struct status: struct = structural named-record alias (s_struct -> tuple_type), methods error, P(...) ctor call fails. TODO: ctor calls (labels/positional), methods, nominal, with, derived order. Then tell Ryan.
00:41 struct ctor calls: x_call rewrites P(...) (labels/positional) -> tuple literal w/ struct type. test tests/ok/struct_ctor.cp (expect 3 7 4).
00:42 fixed Sym.ty attr in ctor rewrite; retesting struct_ctor.
00:43 struct_ctor OK (3 7 4). run_all+commit running. Then tell Ryan prototype struct status: fields/records/ctor calls work; methods, nominal typing, with, derived order not yet.
00:44 Ryan DECIDED: keep anonymous records (one-off returns); NO mut at method-call receiver. Sent params proposal: positional + any arg by name, defaults, named-only after ';', no *args/**kwargs, typed variadics ..xs: T. Pending his OK. TODO impl: kwargs to user fns, defaults, ';' named-only, struct methods.
00:46 Ryan DECIDED 18 both enum forms, 19 current naming, 20 no semicolons. Assumed params proposal accepted (stated). NEXT: struct methods, then kwargs/defaults/';'.
00:48 struct methods: s_struct desugars methods to S__m fns (self param typed S, fields injected as syms via fn.self_struct); method_call dispatches via struct_methods[t.cpp]; static fns via Type.m(). test tests/ok/struct_methods.cp expect '5 7'.
00:50 sent Ryan missing style decisions 21-33 table (lambdas =>, inline if, ** power, floor div/mod, 1e9 ints, by step, chained cmp, last-expr return, traits in body, if const, _, assert, modules). Default lock unless changes. Traits/generics/specialization design sent.
00:51 prototype / and % use C++ truncation (-7/3=-2, -7%3=-1). If floor div decided: change emit to floordiv/floormod helpers (cost: extra branch; skip when operands provably >=0 via ranges).
00:53 Ryan asked: add 21-33 to style-decisions.md (mark proposed) + mark settled ones. In progress.
00:55 style doc: marked 2-7,10,11,13-15 decided (default picks), appended 21-33 proposed + 'also decided' section. Attach to Ryan.
00:57 Ryan challenged trait-on-type monoids. REVISED: monoids/actions = const values (Monoid<T>(id, op)), SegTree<V,T>(sum); laws on value checked by fntest; traits only for one-canonical-instance (Ord/Eq/Hash/Show/Read/ops). Doc decision 29 rewritten. design-v2 to update.
00:58 Ryan: rewrite 21-33 in the existing doc format (snippet per option side-by-side columns, (rec)/(now) marks). In progress.
00:59 rewrote 21-33 in existing format (side-by-side options, now/rec). Commit + tell Ryan.
01:02 wrote ~/cplang/operations.md (operation values: algebra table, orderings, combinators, structures table, laws, needed features, open qs). Attach to Ryan.
01:05 wrote ~/cplang/dsa-catalogue.md (Part 1 catalogue, Part 2 abstractions A-I, coverage table, gaps). Attach to Ryan.
01:07 Ryan asked runs(a) -> (l,r,x). Added sequence views to dsa-catalogue F (runs, runs_by, pairs, windows, groups, scan). Generators (yields) now higher priority in prototype.
01:10 Ryan DECIDED 24: floor div, % non-negative. Implementing (plain / % when both operands provably >= 0 via ranges).
01:12 floor div: compiler.py binop / % -> plain when a>=0,b>0 else cp_ediv/cp_emod (runtime @part ediv, Euclidean: % in [0,|b|-1]). test tests/ok/ediv.cp: expect '-3 2' and (-7/-3: 3 2).
01:14 Ryan asked getters: zero-arg read-only methods called w/o parens (s.len), mut methods need parens; add to style doc 'also decided'. Answered 26 (by + .rev, .step alias, no negative steps). FIXING: cp_ediv part not kept by tree-shaker (see compiler.py ~2940 keep()); run_all after; earlier run_all killed before commit.
01:16 committed floor div (ediv test: -7/2 -> -4 1 ok), run_all green. Getters added to style doc.
01:17 Ryan: getters must be explicit -> @get opt-in (style doc updated). TODO implement @get in prototype (no-paren call only for @get methods).
01:20 Ryan locked: .copy() for big values (moves implicit), constructors move args, Int + ranges (I32/U8 only bitwise), Int capitalized. Recorded in style doc.
01:23 Ryan: indexing was one P0 example; drafted ~/cplang/goals.md (P0 G1-G7, P1 G8-G11, P2, non-goals, tie-break, open qs). Also proposed design-v2 restructure: features 1-6 x aspects (semantics/checking/syntax/representation/tooling).
01:26 Ryan locked: ~95% stdlib in ordinary code + intrinsics layer (Buffer, prims, bit ops, I/O, hash seed) + @trusted; goals G8 updated. Next: language design parts (features 1-6 x aspects).
01:28 Ryan: 64KB critical. cp build errors >64KB, warns >48KB. Ports 9.8-20.3KB (I earlier said 7-15, corrected). Per-part size breakdown + --compact still TODO.
01:30 Ryan aligned on style. Writing ~/cplang/program-structure.md (file layout, entry, fns/parts, multitest/per-test state, modules, tests/brute/invariant, checker/interactor, local-only, versioning).
01:33 wrote program-structure.md: top-level code = entry; top-level let visible to fns, var not; mut fn nested; pure fn for brute/invariant/memo; @multitest locals; tests table; checker/interactor; cfg.local/debug(); use lib.x tree-shaken; changes vs design-v2; open qs.
01:36 wrote libraries.md (layout, cp.toml roots, module example, decl-only lib modules, imports/clashes/prelude, coherence rule, bundling pipeline+size report, lib tests, stamps/frozen/vendor, stdlib layers 0-6, caching, open qs).
01:39 Ryan asked std vs stdlib -> recommended std (cp.toml std key -> cpp). Docs updated (use std.x).
01:42 Ryan: turn libraries.md into tooling.md = whole tooling plan (fs layout, projects/config levels, samples org, libs/store, cp UX build/run/test/stress, compile <500ms, debug mode w/o gcc).
01:47 wrote tooling.md (replaces libraries.md). Measured: transpile .04s, g++ O2 1.01s, O0 1.00s, PCH O2 .44s, PCH O0 .34s. Debug = interpreter on typed IR; check = O1+PCH+sanitizers; release = O2+PCH. Store for compat. Command UX.
01:50 Ryan: wants cp run d i1 i2 with shared scratch inputs i{1..5}. Added to tooling.md §1 + command table (oN optional expected, tests="scratch" stress saves to next iN). Not implemented in proto cp.py yet.
01:53 Ryan: lib updates must be easy in-file; local path libs. Policy: try current lib, fall back to stamped if it breaks, pin/--frozen; store snapshots path libs incl dirty; names not paths; cp diff-lib. tooling.md §5 updated.
01:56 Ryan: great errors + debug messages are a key goal. goals G5 rewritten (errors/debugging P0); tooling.md new §7 Diagnostics (compile errors w/ fix, runtime errors source terms, auto stage tracing --trace, stress side-by-side traces, feel).
01:58 Ryan: snapshot a stage's input from debug run & rerun only that stage. Added tooling.md §7 Stage snapshots (cp snap, --stage, --brute, auto from locate/stress, keep→fntest; pure fn only).
02:01 wrote debugging.md (walkthrough distinct-values-in-range problem: compile errs, sample WA + trace, stress side-by-side, snapshot stage rerun, runtime errors, perf/explain, submit, multitest/interactive, principles, what it takes).
02:04 Ryan: --trace key, needs filtering, not noisy. debugging.md §8b: quiet defaults (~20 lines, fold loops, hide helpers, cap 200 lines→--record), filters table, @trace/@notrace, --record + cp trace browser.
02:06 Ryan: trace should output HTML. debugging.md §8b: terminal summary + self-contained HTML trace (tree, search/when filter, typed rendering, brute diff, source links, snapshot button); always recorded in debug.
02:08 Ryan: HTML should walk through entire execution. debugging.md updated (chapters, scrubber, live data views, jump to first wrong). Building mock ~/cplang/trace-mock.html for distinct-values example.
02:12 building mock/trace-mock.html (3-pane: chapters, source w/ inline values + current line, step view w/ data cells; scrubber; jump-first-wrong; sample i2 a=[1,2,1,3,2], bug fw.add(p,1) should be -1). Verify in browser then attach.
02:14 mock done ~/cplang/mock/trace-mock.html (verified headless screenshot; fixed uppercase headings). Attach to Ryan.
02:17 Ryan: need top-notch LSP + VS Code ext. tooling.md §9 rewritten (LSP features, latency target, VS Code lenses/test panel/trace webview/size status). Asked Ryan P0 vs P1 for editor goal.
02:20 Ryan: stress must shrink smartly + live updates. tooling.md §7b added. Ryan said remaining tooling gaps fine for now; trace data contract (renderer UI later) is the language-design priority.
02:23 NEXT (in progress): write ~/cplang/abstractions.md from dsa-catalogue.md + operations.md: Part1 mechanisms (generics, traits vs op/order values, laws, if const, @trusted+Buffer intrinsics, niche hook trait, generators yields, pools/handles, derive Show for trace contract); Part2 stdlib by layer 0-6 with real signatures (List on Buffer, Opt niche, Set<D> via if const Mask64/DenseSet/tree, SegTree<D,T>(Monoid), LazySegTree, Fenwick(Group), Dsu, Graph<V>/Csr views, Tree, ModInt<M>, Heap(Order), sequence views runs/windows); Part3 coverage check vs every catalogue section + gaps. Then attach to Ryan, block.
02:27 wrote abstractions.md. Attach to Ryan.
02:31 Ryan asked what else like Niche: added abstractions.md M7b representation traits (Niche Index Bits Small Relocate Zero OrdKey Bounded).
02:36 expanded abstractions.md Part 1: section per M1-M10 (what/syntax/rules/cost/example/errors/open) + SegTree-in-user-code example.

INDEX + LENDS (plan/19-index-and-lends.md, approved 2026-11-20). cpc/lends.py = the checker
(flow facts, place paths, `law disjoint`, let-bound lends, second-class views); compiler.py calls it.
- v.l / v.r (= .before / .after), v.l(k) / v.r(k) (k-wide window, counts v), V.K, v.to_idx, V.from_idx(i).
- `impl Index for Struct: let K = e; fn to_idx(self) -> Int; fn from_idx(i: Int) -> Struct` -> [Struct]T,
  `for x in Struct`. `const K = <runtime>` and K sized by a var are errors. g[p] with p: (Row, Col).
- Projections: struct method `fn f(self, ..) lends [mut] T: [law disjoint(..) = ..] ..; lend [mut] place`.
  Emitted as `template <class S_> decltype(auto) f(S_& self_, ..) { ..; return (place); }`.
- Views: a[l..=r] / a[x..y] (bounds) -> slice, g.row(r) -> [B]T; C++ CpView<T>{p, lo, hi}. Passed to a
  top-level fn by specialising it (view_spec); .sort() .rev() .copy() .positions .len; swap(mut x, mut y).
- Checks: args of one call lent together (check_args); `let part = a[s]` holds `a` until the last
  statement naming `part`; views can't be var/assigned/pushed/stored/returned.
- Tests: tests/lend/*.cp + .in + .out (run), tests/bad/*.cp with `// error: text` on line 1 (message locked).
- Ergonomics audit (research/ergonomics-audit.md), tests in tests/ergo/*.cp + .in + .out:
  open ranges `..i` `..=i` `i..` (a boundary may end a range), `span(a, b)`, `r.prefix(k)` / `r.suffix(k)`;
  `i + k` / `i - k` -> ?I (cp_step), `b.left` / `b.right`, `if let a = x and let b = y`;
  `Counter<K>` / `counts(xs)` / `.distinct`, `prefix(a)` + `pre.sum(range[, range])`, `hash.of(range)`;
  `x f= y` == `x = f(x, y)`; `[domain S]Letter = read()`; code after `lend` (write-back, `<proj>_after`);
  a fn declared in a top-level loop or `if` is a closure.
- Char/Str: ASCII only, lexer rejects non-ASCII in literals. `/` and `%` are Euclidean (cp_ediv/cp_emod,
  native when a >= 0 and b > 0 are proven).
- Red-team pass (plan/19 section 5.6, cases 11-23; tests/redteam/*.cp + .in + .out, run under ASan/UBSan
  with -DLOCAL by run_all.sh, and tests/bad/rt_*.cp):
  - lends.py: fn effects (own_effects / fn_effects): a call is a use of what the callee writes, so it is
    checked against live lends, drops facts, and can't take a `mut` alias of what it writes itself.
    `for x in xs` lends xs until the loop ends (hold_iterated). `a[i] += f(mut a[i])` is an error.
  - compiler.py: operands run left to right (ordered(): temporaries only when the order can be observed);
    an assignment runs its right side, then its target's indices. Unbounded + - * neg abs << go through
    cp_add / cp_sub / cp_mul / cp_abs / cp_shl, which stop a -DLOCAL build on overflow. Domain length is
    computed in __int128 and capped at 2^31 - 1. min/max of a slice is ?T. `List<List<Int>>` parses.
  - runtime.hpp: cp_ediv / cp_emod are correct for every operand pair, stop on b == 0 and on
    Int.min / -1; rd_int reads Int.min. List.copy().
