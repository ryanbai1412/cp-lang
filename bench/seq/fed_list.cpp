#include <cstdio>
#include <vector>
typedef long long ll;

// The `mut` accumulator version: one buffer reused across calls.
static void step(ll x, std::vector<ll>& out) {
    out.clear();
    out.push_back(x);
    if (x % 3 == 0) out.push_back(-x);
}

int main() {
    const ll n = 100000000;
    ll s = 0;
    std::vector<ll> out;
    for (ll i = 0; i < n; i++) {
        step(i, out);
        for (ll v : out) s += v;
    }
    printf("%lld\n", s);
}
