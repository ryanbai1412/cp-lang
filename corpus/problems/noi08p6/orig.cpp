// NOI 2008 Table Tennis Tournament reference (output-only; scored against the organizers'
// answer). Little Z's expected prize depends only on the winner distributions of the blocks
// he meets: slots 2, 3-4, 5-8, ... Swap hill climbing, restarted until a fixed work budget runs out, recomputing only the
// touched blocks, keeps the best bracket. checker.py accepts any bracket worth at least as much.
#include <bits/stdc++.h>
using namespace std;

int n, k;
vector<vector<double>> P;
vector<double> a;
long long budget = 8'000'000'000;

// winner distribution over slots [lo, hi) of bracket s, indexed by position in the block
vector<double> block(const vector<int>& s, int lo, int hi) {
    int len = hi - lo;
    vector<double> w(len, 1.0);
    budget -= 2000;
    for (int b = 1; b < len; b *= 2)
        for (int st = 0; st < len; st += 2 * b) {
            vector<double> nw(2 * b);
            for (int i = 0; i < b; i++) {
                double l = 0, r = 0;
                for (int j = 0; j < b; j++) {
                    l += w[st + b + j] * P[s[lo + st + i]][s[lo + st + b + j]];
                    r += w[st + j] * P[s[lo + st + b + i]][s[lo + st + j]];
                }
                nw[i] = w[st + i] * l, nw[b + i] = w[st + b + i] * r;
            }
            copy(nw.begin(), nw.end(), w.begin() + st);
            budget -= 2 * b * b;
        }
    return w;
}

double meet(const vector<int>& s, int r, const vector<double>& w) {
    double p = 0;
    for (int j = 0; j < (int)w.size(); j++) p += w[j] * P[0][s[(1 << r) + j]];
    return p;
}

double value(const vector<double>& beat) {
    double alive = 1, e = 0;
    for (int r = 0; r < k; r++) e += alive * (1 - beat[r]) * a[r], alive *= beat[r];
    return e + alive * a[k];
}

int main() {
    int t;
    cin >> t >> n;
    P.assign(n, vector<double>(n));
    for (auto& row : P)
        for (auto& x : row) cin >> x;
    while ((1 << k) < n) k++;
    a.resize(k + 1);
    for (auto& x : a) cin >> x;
    vector<int> best(n);
    iota(best.begin(), best.end(), 0);
    double bestv = -1;
    mt19937 rng(2008);
    for (int start = 0; start == 0 || (budget > 0 && n > 2); start++) {
        vector<int> s(n);
        iota(s.begin(), s.end(), 0);
        if (start == 0) sort(s.begin() + 1, s.end(), [](int x, int y) { return P[0][x] > P[0][y]; });
        else if (start == 1) sort(s.begin() + 1, s.end(), [](int x, int y) { return P[0][x] < P[0][y]; });
        else shuffle(s.begin() + 1, s.end(), rng);
        vector<vector<double>> w(k);
        vector<double> beat(k);
        for (int r = 0; r < k; r++) w[r] = block(s, 1 << r, 2 << r), beat[r] = meet(s, r, w[r]);
        double v = value(beat);
        for (int fails = 0; fails < 64 * n && budget > 0 && n > 2; fails++) {
            int x = 1 + rng() % (n - 1), y = 1 + rng() % (n - 1);
            if (x == y) continue;
            int rx = 31 - __builtin_clz(x), ry = 31 - __builtin_clz(y);
            swap(s[x], s[y]);
            auto wx = block(s, 1 << rx, 2 << rx), wy = rx == ry ? wx : block(s, 1 << ry, 2 << ry);
            auto nb = beat;
            nb[rx] = meet(s, rx, wx), nb[ry] = meet(s, ry, wy);
            double nv = value(nb);
            if (nv > v * (1 + 1e-12)) fails = 0;
            if (nv >= v) v = nv, beat = nb, w[rx] = wx, w[ry] = wy;
            else swap(s[x], s[y]);
        }
        if (v > bestv) bestv = v, best = s;
    }
    for (int x : best) printf("%d\n", x + 1);
}
