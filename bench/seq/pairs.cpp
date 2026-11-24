#include <cstdio>
#include <map>
#include <vector>
typedef long long ll;

int main() {
    const ll n = 600000;
    ll seed = 123456789;
    std::vector<ll> h;
    for (ll i = 0; i < n; i++) {
        seed = (seed * 1103515245 + 12345) % 2147483647;
        h.push_back(seed % 200000);
    }
    ll found = 0;
    auto crossed = [&](ll i, ll j, ll k) {
        if (k < n && (h[i] + h[j]) % 3 == h[k] % 3) found++;
    };
    std::map<ll, std::vector<ll>> groups;
    for (ll x = 0; x < n; x++) groups[h[x]].push_back(x);
    for (int r = 0; r < 40; r++)
        for (const auto& [key, group] : groups)
            for (size_t a = 0; a < group.size(); a++)
                for (size_t b = a + 1; b < group.size(); b++) crossed(group[a], group[b], group[a] + h[group[b]]);
    printf("%lld\n", found);
}
