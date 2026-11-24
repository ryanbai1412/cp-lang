// NOI 2023 P6 "Merge the Books", corpus-written reference.
// Read the merges top-down as layers: each layer splits some piles into a bottom part (same
// coefficient c) and a top part (c + 1), where a book's coefficient counts the merges in which it
// was moved. The heaviest books take the smallest coefficients, so the weight cost is a sorted dot
// product. A pile of height h costs 2^h - 1 in wear; over layers that is F' = 2F + (piles - 2).
// Splitting the piles with the smallest coefficients is never worse, so a layer is just a count k.
// The method of the NOI 2023 write-ups (linhanpi's notes; 44863 states at n = 100): a DP over
// coefficient multisets (counts per value) by size, where layer counts never fall, so a state also
// keeps its last count, pruned by a lower bound against a greedy bound.
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

void grow(const Counts& c, int k, Counts& d) {
    d.assign(c.begin(), c.end());
    d.push_back(0);
    for (int v = 0; k > 0; v++) {
        int t = min(c[v], k);
        d[v + 1] += t;
        k -= t;
    }
    while (d.back() == 0) d.pop_back();
}

Counts grow(const Counts& c, int k) {
    Counts d;
    grow(c, k, d);
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

// A layer's split count never falls: every split pile has a child one level lower, which the next
// layer splits. So a state is (multiset, last count), and one multiset keeps only the pairs of
// (wear, last count) that no other pair beats in both.
ll solve() {
    if (n == 1) return 0;
    Counts c{1};
    ll F = 0;
    for (int m = 1, last = 1; m < n;) {
        int pick = last;
        ll pl = BIG;
        for (int k = last; k <= min(m, n - m); k++) {
            ll l = lower(grow(c, k), m + k, 2 * F + (m + k - 2), BIG);
            if (l < pl) pl = l, pick = k;
        }
        F = 2 * F + (m + pick - 2);
        c = grow(c, pick);
        m += pick;
        last = pick;
    }
    ll best = F + cost(c);
    vector<unordered_map<Counts, vector<pair<ll, int>>, Hash>> at(n + 1);
    at[1][Counts{1}] = {{0, 1}};
    Counts d;
    for (int m = 1; m < n; m++) {
        for (auto& [s, front] : at[m]) {
            ll least = BIG;
            for (auto [G, last] : front) least = min(least, G);
            if (lower(s, m, least, best) >= best) continue;
            for (auto [G, last] : front) {
                for (int k = last; k <= min(m, n - m); k++) {
                    ll H = 2 * G + (m + k - 2);
                    if (H >= best) break;
                    grow(s, k, d);
                    if (m + k == n) {
                        best = min(best, H + cost(d));
                        continue;
                    }
                    auto it = at[m + k].find(d);
                    if (it == at[m + k].end()) it = at[m + k].emplace(d, vector<pair<ll, int>>()).first;
                    auto& f = it->second;
                    bool beaten = false;
                    for (auto [H2, k2] : f) beaten |= H2 <= H && k2 <= k;
                    if (beaten) continue;
                    erase_if(f, [&](const pair<ll, int>& p) { return H <= p.first && k <= p.second; });
                    f.push_back({H, k});
                }
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
