#include <array>
#include <cstdint>
#include <cstdio>
#include <utility>
#include <vector>

using ll = long long;

static constexpr int L = 25;

struct Trie {
    ll cnt = 0;
    std::array<ll, 2> kid = {-1, -1};
};

std::vector<Trie> nodes;

ll merge(ll x, ll y) {
    if (x == -1) return y;
    if (y == -1) return x;
    nodes[x].cnt += nodes[y].cnt;
    for (int b = 0; b < 2; b++) {
        nodes[x].kid[b] = merge(nodes[x].kid[b], nodes[y].kid[b]);
    }
    return x;
}

ll chain(ll x) {
    nodes.push_back(Trie{1, {-1, -1}});
    for (int z = 0; z < L; z++) {
        ll below = (ll)nodes.size() - 1;
        int bit = (int)((x >> z) & 1);
        Trie t{1, {-1, -1}};
        t.kid[bit] = below;
        nodes.push_back(t);
    }
    return (ll)nodes.size() - 1;
}

ll query_trie(ll root, ll mask, ll want) {
    ll t = root, total = 0;
    for (int b = L - 1; b >= 0; b--) {
        ll next = nodes[t].kid[(want >> b) & 1];
        if (next != -1) {
            t = next;
            if ((mask >> b) & 1) total += nodes[t].cnt << b;
        } else {
            break;
        }
    }
    return total;
}

int main() {
    const int n = 25000;
    ll x = 123456789;
    std::vector<ll> a;
    a.reserve(n);
    for (int i = 0; i < n; i++) {
        x = (x * 1103515245 + 12345) & ((1LL << 31) - 1);
        a.push_back((x >> 6) & ((1LL << L) - 1));
    }
    std::vector<std::pair<ll, ll>> groups;
    ll pref = 0, ans = 0;
    for (ll v_in : a) {
        groups.push_back({(1LL << L) - 1, chain(pref)});
        std::vector<std::pair<ll, ll>> kept;
        for (auto g : groups) {
            ll v = g.first & v_in;
            if (!kept.empty() && kept.back().first == v) {
                merge(kept.back().second, g.second);
            } else {
                kept.push_back({v, g.second});
            }
        }
        groups = std::move(kept);
        pref ^= v_in;
        for (auto g : groups) {
            ans += query_trie(g.second, g.first, pref ^ g.first);
        }
    }
    printf("%lld\n", ans);
    return 0;
}
