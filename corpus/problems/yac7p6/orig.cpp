// yac7p6 reference, offline. A point's value is V of the last assignment covering it (time
// t0, or 0 if none) plus the adds covering it after t0, i.e. A(now) - A(t0). Both the
// latest covering assignment and A split by the step X: a step X >= B touches at most N/B
// cells and is applied cell by cell; each step X < B lays the cells out residue by residue,
// so its progressions are contiguous ranges, kept with sqrt-sized block tags and swept over
// the timeline on its own. One sweep finds t0 and A(now), a second reads A(t0).
#include <bits/stdc++.h>
using namespace std;

int n, q;
vector<int> type_, L, R, X, Y;
vector<long long> V;

struct Group {
    int x, len, bs;  // x = 0: the cell-by-cell group for every step >= B
    vector<int> cellT, blockT;
    vector<long long> cellA, blockA;
    void init(int step, int B) {
        x = step;
        len = x ? x * cnt() : n + 1;
        bs = max(1, (int)sqrt((double)len));
        cellT.assign(len, -1);
        cellA.assign(len, 0);
        blockT.assign(len / bs + 1, -1);
        blockA.assign(len / bs + 1, 0);
        (void)B;
    }
    int cnt() const { return (n + x - 1) / x + 1; }
    int pos(int i) const { return x ? (i % x) * cnt() + i / x : i; }
    void range(int lo, int hi, int t, long long add, bool assign) {
        int blo = lo / bs, bhi = hi / bs;
        auto cell = [&](int p) { if (assign) cellT[p] = t; else cellA[p] += add; };
        auto block = [&](int b) { if (assign) blockT[b] = t; else blockA[b] += add; };
        if (blo == bhi) { for (int p = lo; p <= hi; p++) cell(p); return; }
        for (int p = lo; p < (blo + 1) * bs; p++) cell(p);
        for (int b = blo + 1; b < bhi; b++) block(b);
        for (int p = bhi * bs; p <= hi; p++) cell(p);
    }
    void apply(int t, int B) {
        bool assign = type_[t] == 2;
        if (!x) {
            if (X[t] < B) return;
            for (int i = L[t]; i <= R[t]; i += X[t]) {
                if (assign) cellT[i] = t; else cellA[i] += V[t];
            }
            return;
        }
        if (X[t] != x) return;
        range(pos(L[t]), pos(R[t]), t, V[t], assign);
    }
    int lastT(int i) const { int p = pos(i); return x ? max(cellT[p], blockT[p / bs]) : cellT[p]; }
    long long addA(int i) const { int p = pos(i); return x ? cellA[p] + blockA[p / bs] : cellA[p]; }
};

int main() {
    scanf("%d %d", &n, &q);
    type_.resize(q); L.resize(q); R.resize(q); X.resize(q); V.resize(q); Y.resize(q);
    for (int t = 0; t < q; t++) {
        scanf("%d", &type_[t]);
        if (type_[t] == 3) scanf("%d", &Y[t]);
        else scanf("%d %d %d %lld", &L[t], &R[t], &X[t], &V[t]);
    }
    int B = max(1, (int)sqrt((double)n));
    vector<int> t0(q, -1);
    vector<long long> now(q, 0), before(q, 0);
    vector<vector<int>> readAt(q);
    Group g;
    auto sweep = [&](bool second) {
        for (int step = 0; step < B; step++) {
            g.init(step, B);
            for (int t = 0; t < q; t++) {
                if (type_[t] != 3) g.apply(t, B);
                if (!second) {
                    if (type_[t] == 3) {
                        t0[t] = max(t0[t], g.lastT(Y[t]));
                        now[t] += g.addA(Y[t]);
                    }
                } else {
                    for (int r : readAt[t]) before[r] += g.addA(Y[r]);
                }
            }
        }
    };
    sweep(false);
    for (int t = 0; t < q; t++) if (type_[t] == 3 && t0[t] >= 0) readAt[t0[t]].push_back(t);
    sweep(true);
    string out;
    for (int t = 0; t < q; t++) if (type_[t] == 3) {
        long long v = (t0[t] >= 0 ? V[t0[t]] : 0) + now[t] - before[t];
        out += to_string(v);
        out += '\n';
    }
    fputs(out.c_str(), stdout);
}
