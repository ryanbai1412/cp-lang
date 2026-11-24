// Not a judge-accepted submission: written for the corpus and checked against the sample and an exhaustive brute.
// Counts x/y in lowest terms with gcd(y, k) = 1: sum over d coprime to k of mu(d) [n/d] f([m/d]),
// f(x) = #{y <= x : gcd(y, k) = 1}; S(x, k) = sum of mu(d) over d <= x coprime to k obeys
// S(x, k) = S(x, k / p) + S(x / p, k) for a prime p | k, and S(x, 1) is Mertens by Du's sieve.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;
const int LIM = 2000000;
int mu[LIM + 1];
unordered_map<ll, ll> mert;
vector<int> primes;
map<pair<ll, int>, ll> memo;
ll M(ll x) {
    if (x <= LIM) return mu[x];
    if (auto it = mert.find(x); it != mert.end()) return it->second;
    ll s = 1;
    for (ll l = 2, r; l <= x; l = r + 1) r = x / (x / l), s -= (r - l + 1) * M(x / l);
    return mert[x] = s;
}
ll S(ll x, int j) {  // primes[0..j) still to exclude
    if (x == 0) return 0;
    if (j == 0) return M(x);
    auto key = make_pair(x, j);
    if (auto it = memo.find(key); it != memo.end()) return it->second;
    return memo[key] = S(x, j - 1) + S(x / primes[j - 1], j);
}
int main() {
    ll n, m, k;
    cin >> n >> m >> k;
    vector<int> isc(LIM + 1, 0);
    mu[1] = 1;
    vector<int> pr;
    for (int i = 2; i <= LIM; i++) {
        if (!isc[i]) pr.push_back(i), mu[i] = -1;
        for (int p : pr) {
            if ((ll)p * i > LIM) break;
            isc[p * i] = 1;
            if (i % p == 0) { mu[p * i] = 0; break; }
            mu[p * i] = -mu[i];
        }
    }
    for (int i = 1; i <= LIM; i++) mu[i] += mu[i - 1];
    for (ll t = k, p = 2; p <= t; p++)
        if (t % p == 0) { primes.push_back(p); while (t % p == 0) t /= p; }
    vector<ll> f(k + 1, 0);
    for (int y = 1; y <= k; y++) f[y] = f[y - 1] + (gcd((ll)y, k) == 1);
    auto F = [&](ll x) { return x / k * f[k] + f[x % k]; };
    int P = primes.size();
    __int128 ans = 0;
    ll prev = 0;
    for (ll l = 1, r; l <= min(n, m); l = r + 1) {
        r = min(n / (n / l), m / (m / l));
        ll cur = S(r, P);
        ans += (__int128)(cur - prev) * (n / l) * F(m / l);
        prev = cur;
    }
    printf("%lld\n", (ll)ans);
}
