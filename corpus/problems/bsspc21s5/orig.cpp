// BSSPC '21 S5 James Solves His Tree Problems: the smallest tree that survives the operations, or
// -1. Corpus-written; every C++ submission is RTE or WA. Built backwards from the end: a D puts
// back its dying endpoint (the one whose last D it is) as a leaf of the survivor, an L hangs a new
// leaf on every leaf ("tip"), so each L costs one vertex per tip. A D whose survivor is a tip keeps
// the tip count; otherwise it adds a tip. A D whose endpoints both end there leaves a survivor
// that must already be placed: as a new leaf of the next L (its D then lands on a tip), else in
// place of an already built fresh vertex, else as an extra leaf. The tree after the last
// operation is a path of such survivors, or empty (one vertex or an edge put back by a final L,
// or the final D eating both ends); both empty-start options are tried.
#include <cstdio>
#include <vector>
#include <algorithm>
using namespace std;
int q, X;
vector<int> A, B;  // A<0: L
vector<int> dieOf, survOf;  // per op
vector<char> bothDie;
vector<int> segOf;  // number of Ls before op
int mL;
vector<vector<int>> Bseg;  // B labels per segment

struct Sim {
    vector<int> lab, deg, vOf;
    vector<pair<int,int>> e;
    vector<int> tips, pool;
    int nv() { return lab.size(); }
    int nw(int l) { lab.push_back(l); deg.push_back(0); if (l) vOf[l] = lab.size() - 1; return lab.size() - 1; }
    void ed(int u, int v) { e.push_back({u, v}); deg[u]++; deg[v]++; }
};

bool run(int tq, Sim& S) {
    S = Sim();
    S.vOf.assign(X + 1, -1);
    if (q && A[q - 1] >= 0) {
        auto& bm = Bseg[mL];
        if (!bothDie[q - 1]) {
            int prev = -1;
            for (int l : bm) { int v = S.nw(l); if (prev >= 0) S.ed(prev, v); prev = v; }
        }
    } else if (tq == 1) {
        S.pool.push_back(S.nw(0));
    }
    int j = mL;
    for (int t = q - 1; t >= 0; t--) {
        if (A[t] >= 0) {
            if (bothDie[t]) {
                if (S.nv()) return false;
                int a = S.nw(A[t]), b = S.nw(B[t]);
                S.ed(a, b); S.tips = {a, b};
                continue;
            }
            int s = S.vOf[survOf[t]];
            if (s < 0 || S.nv() < 2 || S.vOf[dieOf[t]] >= 0) return false;
            int d = S.nw(dieOf[t]);
            S.ed(d, s); S.tips.push_back(d);
            continue;
        }
        auto& bs = Bseg[j - 1];
        int c = bs.size();
        vector<int> par, nl;
        int sz = S.nv();
        if (sz == 0) {
            if (c > 2) return false;
            int k = (c == 2 || (t > 0 && A[t - 1] >= 0)) ? 2 : 1;
            for (int i = 0; i < k; i++) nl.push_back(S.nw(0));
            if (k == 2) S.ed(nl[0], nl[1]);
        } else {
            if (sz == 1) par = {0, 0};
            else for (int v : S.tips) if (S.deg[v] == 1) par.push_back(v);
            for (int p : par) { int v = S.nw(0); S.ed(p, v); nl.push_back(v); }
        }
        size_t i = 0;
        vector<int> tipsNew = nl;
        for (; i < bs.size() && i < nl.size(); i++) { S.lab[nl[i]] = bs[i]; S.vOf[bs[i]] = nl[i]; }
        for (; i < bs.size(); i++) {
            if (!S.pool.empty()) { int w = S.pool.back(); S.pool.pop_back(); S.lab[w] = bs[i]; S.vOf[bs[i]] = w; }
            else {
                if (sz == 0) return false;
                int v = S.nw(bs[i]); S.ed(par[0], v); tipsNew.push_back(v);
            }
        }
        for (size_t k = bs.size(); k < nl.size(); k++) S.pool.push_back(nl[k]);
        S.tips = tipsNew;
        j--;
    }
    return true;
}

int main() {
    scanf("%d", &q);
    A.assign(q, -1); B.assign(q, -1);
    for (int i = 0; i < q; i++) {
        char c[2]; scanf("%1s", c);
        if (c[0] == 'D') { scanf("%d %d", &A[i], &B[i]); X = max(X, max(A[i], B[i])); }
    }
    vector<int> last(X + 1, -1), prevS(X + 1, -1);
    for (int i = 0; i < q; i++) if (A[i] >= 0) last[A[i]] = last[B[i]] = i;
    segOf.assign(q, 0);
    int Ls = 0;
    for (int i = 0; i < q; i++) { segOf[i] = Ls; if (A[i] < 0) Ls++; }
    mL = Ls;
    Bseg.assign(mL + 1, {});
    dieOf.assign(q, 0); survOf.assign(q, 0); bothDie.assign(q, 0);
    vector<int> finalAt;
    for (int i = 0; i < q; i++) {
        if (A[i] < 0) continue;
        int a = A[i], b = B[i];
        bool la = last[a] == i, lb = last[b] == i;
        if (!la && !lb) { puts("-1"); return 0; }
        if (la && lb) {
            auto fr = [&](int v) { return prevS[v] >= 0 && segOf[prevS[v]] == segOf[i]; };
            int d = fr(a) ? a : fr(b) ? b : a;
            dieOf[i] = d; survOf[i] = d == a ? b : a;
            Bseg[segOf[i]].push_back(survOf[i]);
        } else {
            dieOf[i] = la ? a : b; survOf[i] = la ? b : a;
            prevS[survOf[i]] = i;
        }
    }
    if (q && A[q - 1] >= 0 && Bseg[mL].size() == 1) { bothDie[q - 1] = 1; Bseg[mL].clear(); }
    Sim best; bool have = false;
    for (int tq = 0; tq < 2; tq++) {
        if (tq == 1 && q && A[q - 1] >= 0) break;
        Sim S;
        if (run(tq, S) && (!have || S.nv() < best.nv())) best = S, have = true;
    }
    if (!have) { puts("-1"); return 0; }
    int n = best.nv(), nxt = X;
    vector<int> id(n);
    for (int v = 0; v < n; v++) id[v] = best.lab[v] ? best.lab[v] : ++nxt;
    printf("%d\n", n);
    for (auto [u, v] : best.e) printf("%d %d\n", id[u], id[v]);
}
