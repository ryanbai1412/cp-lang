#include <cstdint>
#include <iostream>
#include <optional>
#include <utility>
#include <vector>

struct Affine {
    int64_t base;
    int64_t step;
};

struct Node {
    int l = -1;
    int r = -1;
    uint64_t pri;
    int sz = 1;
    int64_t val;
    std::optional<Affine> tag;
};

struct LazyTreap {
    std::vector<Node> nodes;
    int root = -1;
    uint64_t rng_state = 48;

    uint64_t next_u64() {
        rng_state += 0x9e3779b97f4a7c15ULL;
        uint64_t z = rng_state;
        z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
        z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
        return z ^ (z >> 31);
    }

    int sz_of(int u) const { return u < 0 ? 0 : nodes[u].sz; }

    void pull(int u) {
        nodes[u].sz = 1 + sz_of(nodes[u].l) + sz_of(nodes[u].r);
    }

    void put(int u, Affine f) {
        if (u < 0) return;
        int lsz = sz_of(nodes[u].l);
        nodes[u].val += f.base + static_cast<int64_t>(lsz) * f.step;
        if (nodes[u].tag.has_value()) {
            nodes[u].tag->base += f.base;
            nodes[u].tag->step += f.step;
        } else {
            nodes[u].tag = f;
        }
    }

    void push(int u) {
        if (!nodes[u].tag.has_value()) return;
        Affine f = *nodes[u].tag;
        nodes[u].tag.reset();
        int lsz = sz_of(nodes[u].l);
        put(nodes[u].l, f);
        put(nodes[u].r, Affine{f.base + static_cast<int64_t>(lsz + 1) * f.step, f.step});
    }

    std::pair<int, int> split_val(int u, int64_t x) {
        if (u < 0) return {-1, -1};
        push(u);
        if (nodes[u].val >= x) {
            auto [rl, rr] = split_val(nodes[u].r, x);
            nodes[u].r = rl;
            pull(u);
            return {u, rr};
        } else {
            auto [ll, lr] = split_val(nodes[u].l, x);
            nodes[u].l = lr;
            pull(u);
            return {ll, u};
        }
    }

    int merge(int a, int b) {
        if (a < 0) return b;
        if (b < 0) return a;
        if (nodes[a].pri > nodes[b].pri) {
            push(a);
            nodes[a].r = merge(nodes[a].r, b);
            pull(a);
            return a;
        } else {
            push(b);
            nodes[b].l = merge(a, nodes[b].l);
            pull(b);
            return b;
        }
    }

    void apply_all(Affine f) { put(root, f); }

    void insert(int64_t x) {
        int id = static_cast<int>(nodes.size());
        nodes.push_back(Node{-1, -1, next_u64(), 1, x, std::nullopt});
        auto [a, b] = split_val(root, x);
        root = merge(merge(a, id), b);
    }

    void collect(int u, int& rem, std::vector<int64_t>& out) {
        if (u < 0 || rem == 0) return;
        push(u);
        collect(nodes[u].l, rem, out);
        if (rem == 0) return;
        out.push_back(nodes[u].val);
        --rem;
        collect(nodes[u].r, rem, out);
    }

    std::vector<int64_t> take(int k) {
        std::vector<int64_t> out;
        out.reserve(k);
        int rem = k;
        collect(root, rem, out);
        return out;
    }
};

int main() {
    const int n = 200000;
    LazyTreap slopes;
    int64_t x = 1234567;
    for (int i = 1; i <= n; ++i) {
        x = (x * 1103515245LL + 12345LL) & 0x7fffffffLL;
        int64_t base = (x % 1000) - 500;
        int64_t step = -(i % 7);
        slopes.apply_all(Affine{base, step});
        slopes.insert(base * 2 + step);
    }
    auto vals = slopes.take(n / 2);
    int64_t sum = 0;
    for (int64_t v : vals) sum += v;
    std::cout << sum << '\n';
}
