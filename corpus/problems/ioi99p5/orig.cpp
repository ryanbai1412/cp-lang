// Not a judge-accepted submission: written for the corpus and checked against the sample and checker.py.
// x[p], the chips pile p sends each neighbour over the game, satisfies x[i] - x[i+1] = the excess of
// piles 1..i, so the least x has x[i] = 0 at the pile where that is tightest. Then, round after round,
// each pile with chips still owed sends as many as it holds (m = min(owed, chips / neighbours)).
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    cin >> n;
    vector<long long> c(n), x(n);
    for (auto& v : c) cin >> v;
    long long avg = accumulate(c.begin(), c.end(), 0LL) / n, g = 0, lo = 0;
    for (int i = 0; i + 1 < n; i++) g += c[i] - avg, x[i + 1] = x[i] - g, lo = min(lo, x[i + 1]);
    for (auto& v : x) v -= lo;
    vector<pair<int, long long>> moves;
    for (bool moved = true; moved;) {
        moved = false;
        for (int p = 0; p < n; p++) {
            int deg = (p > 0) + (p + 1 < n);
            long long m = min(x[p], c[p] / deg);
            if (m <= 0) continue;
            moves.push_back({p + 1, m});
            x[p] -= m, c[p] -= deg * m;
            if (p > 0) c[p - 1] += m;
            if (p + 1 < n) c[p + 1] += m;
            moved = true;
        }
    }
    cout << moves.size() << "\n";
    for (auto [p, m] : moves) cout << p << " " << m << "\n";
}
