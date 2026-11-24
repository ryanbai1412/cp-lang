// noi21p6: inclusion-exclusion over the set P of start positions. For a fixed P a live robot
// (max P + #R <= n - 1) lets cell c take 1 + [all f(0) agree] + [all f(1) agree] values, where
// f ranges over the effects of the starts in P on c (identity outside the robot's reach); a dead
// robot only allows the empty tape. Spans <= K are enumerated as offset sets; wider spans leave
// every live robot short (#R <= K), so a DP over positions keeps only the last K + 1 starts.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
using u64 = unsigned long long;
const ll MOD = 1e9 + 7;

int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<int> R(m);
    vector<u64> one0(m), zero1(m);
    for (int i = 0; i < m; i++) {
        static char s[205];
        scanf("%s", s);
        vector<pair<int, int>> e(1, {0, 1});
        for (char* c = s; *c; c++) {
            auto& f = e.back();
            if (*c == 'R') e.push_back({0, 1});
            else if (*c == '0') f = {0, 0};
            else if (*c == '1') f = {1, 1};
            else f = {1 - f.first, 1 - f.second};
        }
        R[i] = e.size() - 1;
        for (int d = 0; d < (int)e.size() && d < 64; d++) {
            if (e[d].first) one0[i] |= 1ULL << d;
            if (!e[d].second) zero1[i] |= 1ULL << d;
        }
    }
    int lim = n * m + 5;
    vector<ll> p2(lim + 1, 1), p3(lim + 1, 1);
    for (int i = 1; i <= lim; i++) p2[i] = p2[i - 1] * 2 % MOD, p3[i] = p3[i - 1] * 3 % MOD;
    vector<int> ord(m);
    iota(ord.begin(), ord.end(), 0);
    sort(ord.begin(), ord.end(), [&](int a, int b) { return R[a] < R[b]; });
    int K = (n + 1) / 2 - 1;
    ll ans = 0;

    // Spans <= K: P = a + Q with 0 in Q, Q within [0, K]; the factor of a live robot depends on Q only.
    {
        vector<array<u64, 4>> st((K + 2) * (size_t)m);
        vector<ll> two(n), three(n);  // per threshold t: exponent sums over robots with R <= t
        auto rec = [&](auto&& self, int depth, int top, int cnt) -> void {
            // evaluate Q whose largest offset is top
            array<ll, 2> acc{0, 0};
            vector<ll> e2(n + 1, 0), e3(n + 1, 0);
            for (int k = 0; k < m; k++) {
                int i = ord[k];
                if (R[i] > n - 1) break;
                auto& s = st[(size_t)depth * m + i];
                u64 x0 = s[0] & ~s[1], x1 = s[2] & ~s[3];
                int both = __builtin_popcountll(x0 & x1), any = __builtin_popcountll(x0 | x1);
                int one = any - both;
                e2[R[i]] += one, e3[R[i]] += n - any;
            }
            (void)acc;
            ll s2 = 0, s3 = 0;
            for (int t = 0; t <= n - 1; t++) {
                s2 += e2[t], s3 += e3[t];
                int a = n - 1 - top - t;  // t = n - 1 - a - top
                if (a < 0) continue;
                int dead = 0;
                (void)dead;
                // robots with R > t contribute 1
                ll v = p2[s2] * p3[s3] % MOD;
                ans = (ans + (cnt & 1 ? v : MOD - v)) % MOD;
            }
            for (int q = top + 1; q <= K && q <= n - 1; q++) {
                for (int i = 0; i < m; i++) {
                    auto& s = st[(size_t)depth * m + i];
                    auto& t = st[(size_t)(depth + 1) * m + i];
                    u64 a = one0[i] << q, b = zero1[i] << q;
                    t = {s[0] | a, s[1] & a, s[2] | b, s[3] & b};
                }
                self(self, depth + 1, q, cnt + 1);
            }
        };
        for (int i = 0; i < m; i++) st[i] = {one0[i], one0[i], zero1[i], zero1[i]};
        rec(rec, 0, 0, 1);
    }

    // Spans > K: max P = b >= K + 1, some start <= b - K - 1, and live robots have R <= n - 1 - b <= K.
    if (K + 1 <= n - 1) {
        int B = K + 1, S = 1 << B;
        vector<ll> F(S);
        vector<ll> cur(2 * S), nxt(2 * S);
        for (int b = K + 1; b <= n - 1; b++) {
            int t = n - 1 - b;
            for (int W = 0; W < S; W++) {
                int c2 = 0, c3 = 0;
                for (int i = 0; i < m; i++) {
                    if (R[i] > t) continue;
                    int h = 1 + !(W & one0[i]) + !(W & zero1[i]);
                    c2 += h == 2, c3 += h == 3;
                }
                F[W] = p2[c2] * p3[c3] % MOD;
            }
            fill(cur.begin(), cur.end(), 0);
            cur[0] = MOD - 1;  // sign (-1)^(|P| + 1)
            for (int c = 0; c < n; c++) {
                fill(nxt.begin(), nxt.end(), 0);
                for (int fl = 0; fl < 2; fl++)
                    for (int W = 0; W < S; W++) {
                        ll v = cur[fl * S + W];
                        if (!v) continue;
                        int base = (W << 1) & (S - 1);
                        if (c != b) {  // c not chosen
                            int idx = fl * S + base;
                            nxt[idx] = (nxt[idx] + v * F[base]) % MOD;
                        }
                        if (c <= b) {  // c chosen
                            int nf = fl | (c <= b - K - 1), w = base | 1;
                            int idx = nf * S + w;
                            nxt[idx] = (nxt[idx] + (MOD - v) * F[w]) % MOD;
                        }
                    }
                swap(cur, nxt);
            }
            for (int W = 0; W < S; W++) ans = (ans + cur[S + W]) % MOD;
        }
    }
    printf("%lld\n", ans);
}
