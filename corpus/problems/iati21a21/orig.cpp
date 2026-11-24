// IATI 2021 Hint. No accepted submission exists (the four C++ ones crash). genHint sends no
// bits, the full score (640 / (L + 1) >= 1); solve finds an optimal common subsequence by
// itself: Hirschberg's split, each half's LCS row computed bit-parallel (Allison-Dix, as
// Hyyro writes it), O(N M / 64) word steps overall. The main stands in for the grader's two
// rounds as interactor.py runs them: N M, A, B, K, C -> L and the bits; N M, A, B, L, bits ->
// the length and the sequence.
#include <bits/stdc++.h>
using namespace std;
typedef uint64_t u64;

vector<bool> genHint(const vector<int>&, const vector<int>&, const vector<int>&) { return {}; }

static const vector<int>*A, *B;
static vector<vector<int>> where;  // positions of each value in B
static vector<int> out;

// row[j] = LCS of A[a0, a1) (reversed if rev) with the first j of B[b0, b1) (its last j if rev).
static void lcsRow(int a0, int a1, int b0, int b1, bool rev, vector<int>& row) {
    int len = b1 - b0, words = (len + 63) / 64;
    vector<u64> v(words, ~0ULL), mk(words, 0);
    // A value seen more often than there are words gets its mask built once for the call;
    // there are fewer than 64 such values, and the rest are set and cleared per row.
    unordered_map<int, vector<u64>> heavy;
    for (int t = 0; t < a1 - a0; t++) {
        int x = (*A)[rev ? a1 - 1 - t : a0 + t];
        auto& w = where[x];
        auto lo = lower_bound(w.begin(), w.end(), b0), hi = lower_bound(w.begin(), w.end(), b1);
        if (lo == hi) continue;
        bool big = hi - lo > words;
        if (big && !heavy.count(x)) {
            auto& h = heavy[x];
            h.assign(words, 0);
            for (auto p = lo; p != hi; ++p) {
                int bit = rev ? b1 - 1 - *p : *p - b0;
                h[bit >> 6] |= 1ULL << (bit & 63);
            }
        }
        if (!big)
            for (auto p = lo; p != hi; ++p) {
                int bit = rev ? b1 - 1 - *p : *p - b0;
                mk[bit >> 6] |= 1ULL << (bit & 63);
            }
        const u64* m = big ? heavy[x].data() : mk.data();
        u64 carry = 0, borrow = 0;
        for (int i = 0; i < words; i++) {
            u64 x0 = v[i], u = x0 & m[i];
            u64 s = x0 + u, c = s < u;
            s += carry, c |= s < carry, carry = c;
            u64 d = x0 - u, b = x0 < u;
            u64 d2 = d - borrow;
            b |= d < borrow, borrow = b;
            v[i] = s | d2;
        }
        if (!big)
            for (auto p = lo; p != hi; ++p) mk[(rev ? b1 - 1 - *p : *p - b0) >> 6] = 0;
    }
    row.assign(len + 1, 0);
    for (int j = 0; j < len; j++) row[j + 1] = row[j] + !(v[j >> 6] >> (j & 63) & 1);
}

static void hirschberg(int a0, int a1, int b0, int b1) {
    if (a0 == a1 || b0 == b1) return;
    if (a1 - a0 == 1) {
        auto& w = where[(*A)[a0]];
        auto p = lower_bound(w.begin(), w.end(), b0);
        if (p != w.end() && *p < b1) out.push_back((*A)[a0]);
        return;
    }
    int mid = (a0 + a1) / 2, len = b1 - b0, best = -1, split = 0;
    vector<int> f, g;
    lcsRow(a0, mid, b0, b1, false, f);
    lcsRow(mid, a1, b0, b1, true, g);
    for (int j = 0; j <= len; j++)
        if (f[j] + g[len - j] > best) best = f[j] + g[len - j], split = j;
    if (best == 0) return;
    hirschberg(a0, mid, b0, b0 + split);
    hirschberg(mid, a1, b0 + split, b1);
}

vector<int> solve(const vector<int>& a, const vector<int>& b, const vector<bool>&) {
    A = &a, B = &b;
    int vmax = 0;
    for (int x : a) vmax = max(vmax, x);
    for (int x : b) vmax = max(vmax, x);
    where.assign(vmax + 1, {});
    for (int j = 0; j < (int)b.size(); j++) where[b[j]].push_back(j);
    out.clear();
    hirschberg(0, a.size(), 0, b.size());
    return out;
}

static vector<int> readv(int n) {
    vector<int> v(n);
    for (int& x : v)
        if (scanf("%d", &x) != 1) exit(1);
    return v;
}

int main() {
    int n, m, k;
    if (scanf("%d %d", &n, &m) != 2) return 1;
    vector<int> a = readv(n), b = readv(m);
    if (scanf("%d", &k) != 1) return 1;
    vector<bool> hint = genHint(a, b, readv(k));
    string bits(hint.size(), '0');
    for (size_t i = 0; i < hint.size(); i++) bits[i] = hint[i] ? '1' : '0';
    printf("%zu\n%s\n", bits.size(), bits.c_str());
    fflush(stdout);
    int l;
    if (scanf("%d %d", &n, &m) != 2) return 1;
    a = readv(n), b = readv(m);
    if (scanf("%d", &l) != 1) return 1;
    static char buf[1 << 20];
    vector<bool> got(l);
    if (l) {
        string s;
        while ((int)s.size() < l && scanf("%1048575s", buf) == 1) s += buf;
        for (int i = 0; i < l; i++) got[i] = s[i] == '1';
    }
    vector<int> c = solve(a, b, got);
    printf("%zu\n", c.size());
    for (int x : c) printf("%d ", x);
    printf("\n");
}
