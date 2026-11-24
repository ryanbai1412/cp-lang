// NOI 2023 P6 "Merge the Books", corpus-written reference.
// Read the merges top-down as layers: each layer splits some piles into a bottom part (same
// coefficient c) and a top part (c + 1), where a book's coefficient counts the merges in which it
// was moved. The heaviest books take the smallest coefficients, so the weight cost is a sorted dot
// product. A pile of height h costs 2^h - 1 in wear; over layers that is F' = 2F + (piles - 2).
// Splitting the piles with the smallest coefficients is never worse, so a layer is just a count k.
// A layer always adds piles, so a DP over coefficient multisets (counts per value) by size
// expands each multiset once with its least F, pruned by a lower bound against a greedy bound.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
typedef vector<int> Counts;
const ll BIG = LLONG_MAX / 4;  // every answer is below 2^50

int n;
vector<ll> P;  // prefix sums of the weights, heaviest first

ll cost(const Counts& c) {
    ll s = 0;
    int at = 0;
    for (int v = 0; v < (int)c.size() && at < n; v++) {
        int t = min(c[v], n - at);
        s += v * (P[at + t] - P[at]);
        at += t;
    }
    return s;
}

Counts grow(const Counts& c, int k) {
    Counts d(c);
    d.push_back(0);
    for (int v = 0; k > 0; v++) {
        int t = min(c[v], k);
        d[v + 1] += t;
        k -= t;
    }
    while (d.back() == 0) d.pop_back();
    return d;
}

// The least final cost any completion of (c, F) with m piles can reach. With R more layers the
// pile counts are at least m + i and n / 2^(R - i), and each layer at most adds one c + 1 per c.
ll lower(const Counts& c, int m, ll F, ll best) {
    if (m == n) return F + cost(c);
    ll lo = BIG;
    Counts all(c);
    for (int R = 1; R <= n; R++) {
        if (F > (BIG >> R)) break;
        all.push_back(0);
        for (int v = all.size() - 1; v > 0; v--) all[v] = min(n, all[v] + all[v - 1]);
        ll wear = F << R;
        for (int i = 1; i <= R && wear < BIG; i++) {
            ll need = max<ll>(m + i, R - i >= 30 ? 1 : (n + (1LL << (R - i)) - 1) >> (R - i));
            if (need > n) { wear = BIG; break; }
            if (need > 2) wear += R - i >= 50 ? BIG : (need - 2) << (R - i);
        }
        if ((F << R) >= best) break;
        if (wear >= BIG) continue;
        lo = min(lo, wear + cost(all));
    }
    return lo;
}

struct Hash {
    size_t operator()(const Counts& c) const {
        size_t h = c.size();
        for (int x : c) h = h * 1000003 ^ x;
        return h;
    }
};

ll solve() {
    if (n == 1) return 0;
    Counts c{1};
    ll F = 0, best = BIG;
    for (int m = 1; m < n;) {
        int pick = 1;
        ll pl = BIG;
        for (int k = 1; k <= min(m, n - m); k++) {
            ll l = lower(grow(c, k), m + k, 2 * F + (m + k - 2), BIG);
            if (l < pl) pl = l, pick = k;
        }
        F = 2 * F + (m + pick - 2);
        c = grow(c, pick);
        m += pick;
    }
    best = F + cost(c);
    vector<unordered_map<Counts, ll, Hash>> at(n + 1);
    at[1][Counts{1}] = 0;
    for (int m = 1; m < n; m++) {
        for (auto& [s, G] : at[m]) {
            if (lower(s, m, G, best) >= best) continue;
            for (int k = 1; k <= min(m, n - m); k++) {
                Counts d = grow(s, k);
                ll H = 2 * G + (m + k - 2);
                if (m + k == n) {
                    best = min(best, H + cost(d));
                    continue;
                }
                auto [it, fresh] = at[m + k].emplace(d, H);
                if (!fresh) it->second = min(it->second, H);
            }
        }
        at[m].clear();
    }
    return best;
}

int main() {
    int t;
    scanf("%d", &t);
    while (t--) {
        scanf("%d", &n);
        vector<ll> w(n);
        for (auto& x : w) scanf("%lld", &x);
        sort(w.rbegin(), w.rend());
        P.assign(n + 1, 0);
        for (int i = 0; i < n; i++) P[i + 1] = P[i] + w[i];
        printf("%lld\n", solve());
    }
}
