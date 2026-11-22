#include <bits/stdc++.h>
using namespace std;
vector<int> p, sz;
int f(int x) { return p[x] == x ? x : p[x] = f(p[x]); }
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    int n, m;
    cin >> n >> m;
    p.resize(n + 1);
    sz.assign(n + 1, 1);
    iota(p.begin(), p.end(), 0);
    int c = n, b = 1;
    while (m--) {
        int x, y;
        cin >> x >> y;
        x = f(x), y = f(y);
        if (x != y) {
            if (sz[x] < sz[y]) swap(x, y);
            p[y] = x, sz[x] += sz[y], c--, b = max(b, sz[x]);
        }
        cout << c << ' ' << b << '\n';
    }
}
