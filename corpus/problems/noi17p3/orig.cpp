// Not a judge-accepted submission: written for the corpus and checked against the sample and an exhaustive brute.
// P(max <= K) by the first column whose safe height is exactly i: s[i][j] is the chance a width-j
// strip with its bottom i rows safe keeps every bottom rectangle within K; the whole width obeys a
// linear recurrence of order K + 1, taken to N by polynomial powering mod its characteristic polynomial.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
const ll MOD = 998244353;
ll pw(ll b, ll e) { ll r = 1; b %= MOD; for (; e; e >>= 1, b = b * b % MOD) if (e & 1) r = r * b % MOD; return r; }
ll N, q, nq;
ll atMost(int K) {  // probability the largest bottom rectangle has area <= K
    if (K == 0) return pw(nq, N);
    vector<ll> qp(K + 2, 1);
    for (int i = 1; i <= K + 1; i++) qp[i] = qp[i - 1] * q % MOD;
    vector<ll> up(2, 1);  // s[K + 1][j] for j = 0 only
    up[1] = 0;
    for (int i = K; i >= 1; i--) {
        int w = K / i;
        vector<ll> s(w + 1, 0);
        s[0] = 1;
        for (int j = 1; j <= w; j++) {
            ll v = j < (int)up.size() ? qp[j] * up[j] % MOD : 0;
            for (int p = 1; p <= j; p++)
                if (p - 1 < (int)up.size()) v = (v + qp[p - 1] * up[p - 1] % MOD * nq % MOD * s[j - p]) % MOD;
            s[j] = v;
        }
        up = s;
    }
    // h[n] = [n <= K] q^n s1[n] + sum_{p=1}^{K+1} a_p h[n - p], a_p = q^(p-1) s1[p-1] (1 - q)
    int L = K + 1;
    vector<ll> a(L + 1, 0), h(L + 1, 0);
    for (int p = 1; p <= L; p++) a[p] = qp[p - 1] * up[p - 1] % MOD * nq % MOD;
    h[0] = 1;
    for (int n = 1; n <= L; n++) {
        ll v = n <= K ? qp[n] * up[n] % MOD : 0;
        for (int p = 1; p <= n; p++) v = (v + a[p] * h[n - p]) % MOD;
        h[n] = v;
    }
    if (N <= L) return h[N];
    // h[n] = sum a_p h[n-p] for n > K; x^N mod (x^L - sum a_p x^(L-p)), applied to h[0..L-1]
    auto mul = [&](const vector<ll>& A, const vector<ll>& B) {
        vector<ll> c(2 * L - 1, 0);
        for (int i = 0; i < L; i++) if (A[i])
            for (int j = 0; j < L; j++) c[i + j] = (c[i + j] + A[i] * B[j]) % MOD;
        for (int d = 2 * L - 2; d >= L; d--) if (c[d])
            for (int p = 1; p <= L; p++) c[d - p] = (c[d - p] + c[d] * a[p]) % MOD;
        c.resize(L);
        return c;
    };
    // the recurrence holds from n = L on (h[L] has no extra term only if L > K, true), so shift by 1
    vector<ll> res(L, 0), base(L, 0);
    res[0] = 1;
    if (L == 1) base[0] = a[1]; else base[1] = 1;
    ll e = N - 1;
    for (; e; e >>= 1, base = mul(base, base)) if (e & 1) res = mul(res, base);
    // res is x^(N-1); terms h[1..L] are a valid window (h[n] for n >= L+1 obeys the recurrence)
    ll ans = 0;
    for (int i = 0; i < L; i++) ans = (ans + res[i] * h[i + 1]) % MOD;
    return ans;
}
int main() {
    ll K, x, y;
    cin >> N >> K >> x >> y;
    q = x * pw(y, MOD - 2) % MOD, nq = (1 - q + MOD) % MOD;
    printf("%lld\n", (atMost(K) - atMost(K - 1) + MOD) % MOD);
}
