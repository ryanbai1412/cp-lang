#include <bits/stdc++.h>
using namespace std;
using ll = long long;

struct Dsu {
    vector<int> p;
    int parts;
    Dsu(int n) : p(n, -1), parts(n) {}
    int find(int x) { return p[x] < 0 ? x : p[x] = find(p[x]); }
    bool unite(int a, int b) {
        a = find(a); b = find(b);
        if (a == b) return false;
        if (p[a] > p[b]) swap(a, b);
        p[a] += p[b]; p[b] = a; --parts;
        return true;
    }
    bool same(int a, int b) { return find(a) == find(b); }
    int size(int a) { return -p[find(a)]; }
};

int main() {
    const int n = 500000;
    Dsu d(n);
    uint64_t s = 135792468ULL;
    ll acc = 0;
    for (int i = 0; i < 500000; ++i) {
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        int u = (int)((s >> 33) % n);
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        int v = (int)((s >> 33) % n);
        if (d.unite(u, v)) acc += 1;
    }
    for (int i = 0; i < 1000000; ++i) {
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        int u = (int)((s >> 33) % n);
        s = s * 6364136223846793005ULL + 1442695040888963407ULL;
        int v = (int)((s >> 33) % n);
        if (d.same(u, v)) acc += d.size(u);
    }
    cout << acc + d.parts << "\n";
}
