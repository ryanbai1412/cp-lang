// Matched C++ treap benchmark for SortedBag.range(r):
// identical randomized treap with multiplicity + subtree counts + parent pointers.
#include <cstdint>
#include <iostream>
#include <vector>
using namespace std;

struct Treap {
    vector<int64_t> key, cnt, sub, left, right, par;
    vector<uint64_t> pri;
    int64_t root = -1;
    uint64_t seed = 0x9e3779b97f4a7c15ULL;

    int64_t sub_of(int64_t u) const { return u >= 0 ? sub[u] : 0; }
    void pull(int64_t u) {
        if (left[u] >= 0) par[left[u]] = u;
        if (right[u] >= 0) par[right[u]] = u;
        sub[u] = cnt[u] + sub_of(left[u]) + sub_of(right[u]);
    }
    int64_t make_node(int64_t k) {
        seed += 0x9e3779b97f4a7c15ULL;
        uint64_t z = seed;
        z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
        z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
        z ^= z >> 31;
        int64_t id = (int64_t)key.size();
        key.push_back(k); pri.push_back(z); cnt.push_back(1); sub.push_back(1);
        left.push_back(-1); right.push_back(-1); par.push_back(-1);
        return id;
    }
    pair<int64_t, int64_t> split_lt(int64_t u, int64_t k) {
        if (u < 0) return {-1, -1};
        if (key[u] < k) {
            auto [a, b] = split_lt(right[u], k);
            right[u] = a; pull(u);
            if (b >= 0) par[b] = -1;
            return {u, b};
        } else {
            auto [a, b] = split_lt(left[u], k);
            left[u] = b; pull(u);
            if (a >= 0) par[a] = -1;
            return {a, u};
        }
    }
    pair<int64_t, int64_t> split_le(int64_t u, int64_t k) {
        if (u < 0) return {-1, -1};
        if (!(k < key[u])) {
            auto [a, b] = split_le(right[u], k);
            right[u] = a; pull(u);
            if (b >= 0) par[b] = -1;
            return {u, b};
        } else {
            auto [a, b] = split_le(left[u], k);
            left[u] = b; pull(u);
            if (a >= 0) par[a] = -1;
            return {a, u};
        }
    }
    int64_t merge(int64_t a, int64_t b) {
        if (a < 0) return b;
        if (b < 0) return a;
        if (pri[a] > pri[b]) {
            right[a] = merge(right[a], b); pull(a); return a;
        } else {
            left[b] = merge(a, left[b]); pull(b); return b;
        }
    }
    void add(int64_t k) {
        auto [lo, rest] = split_lt(root, k);
        auto [eq, hi] = split_le(rest, k);
        int64_t mid = eq;
        if (mid >= 0) { cnt[mid] += 1; pull(mid); }
        else mid = make_node(k);
        root = merge(merge(lo, mid), hi);
        if (root >= 0) par[root] = -1;
    }
    int64_t first_ge(int64_t k) const {
        int64_t ans = -1, u = root;
        while (u >= 0) {
            if (!(key[u] < k)) { ans = u; u = left[u]; }
            else u = right[u];
        }
        return ans >= 0 ? key[ans] : 0;
    }
    int64_t last_lt(int64_t k) const {
        int64_t ans = -1, u = root;
        while (u >= 0) {
            if (key[u] < k) { ans = u; u = right[u]; }
            else u = left[u];
        }
        return ans >= 0 ? key[ans] : 0;
    }
    int64_t rank(int64_t k) const {
        int64_t ans = 0, u = root;
        while (u >= 0) {
            if (key[u] < k) { ans += sub_of(left[u]) + cnt[u]; u = right[u]; }
            else u = left[u];
        }
        return ans;
    }
    int64_t cursor_ge(int64_t k) const {
        int64_t ans = -1, u = root;
        while (u >= 0) {
            if (!(key[u] < k)) { ans = u; u = left[u]; }
            else u = right[u];
        }
        return ans;
    }
    int64_t next_cursor(int64_t u) const {
        if (right[u] >= 0) {
            u = right[u];
            while (left[u] >= 0) u = left[u];
            return u;
        }
        while (par[u] >= 0) {
            int64_t p = par[u];
            if (left[p] == u) return p;
            u = p;
        }
        return -1;
    }
};

int main() {
    Treap bag;
    int64_t rng = 123456789;
    auto next_rng = [&]() -> int64_t {
        rng = (rng * 1103515245LL + 12345LL) & 2147483647LL;
        return rng;
    };
    for (int i = 0; i < 100000; i++) bag.add(next_rng() % 50000);
    int64_t acc = 0;
    for (int i = 0; i < 100000; i++) {
        int64_t a = next_rng() % 50000;
        int64_t b = a + (next_rng() % 200);
        acc += bag.first_ge(a);
        acc += bag.last_lt(b);
        int64_t r = bag.rank(b) - bag.rank(a);
        acc += r > 0 ? r : 0;
    }
    for (int i = 0; i < 20000; i++) {
        int64_t a = next_rng() % 50000;
        int64_t hi = a + 12;
        for (int64_t u = bag.cursor_ge(a); u >= 0 && bag.key[u] < hi; u = bag.next_cursor(u)) {
            acc += bag.key[u] * bag.cnt[u];
        }
    }
    cout << acc << "\n";
}
