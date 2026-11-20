#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

using C = complex<long>;
using Cd = complex<ld>;

long cross(const C& a, const C& b) {
    return (conj(a) * b).imag();
}

long cross(const C& a, const C& b, const C& c) {
    return cross(b - a, c - a);
}

const int maxn = 2e5 + 5;

int arr[maxn], siz[maxn], ans[maxn];
vector<int> graph[maxn];

void pdfs(int u) {
    siz[u] = 1;
    for (auto& v : graph[u]) {
        graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
        pdfs(v);
        siz[u] += siz[v];
    }
}

void dfs(int u, int l, int r) {
    if (!sz(graph[u])) {
        ans[l] = u;
        return;
    } else if (sz(graph[u]) == 1) {
        ans[l] = u;
        dfs(graph[u][0], l + 1, r);
    } else if (sz(graph[u]) == 2) {
        int mid = l + siz[graph[u][0]];
        ans[mid] = u;
        dfs(graph[u][0], l, mid - 1);
        dfs(graph[u][1], mid + 1, r);
    } else {
        assert(false);
    }
}

void solve() {
    int n;
    cin >> n;
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    pair<Cd, int> carr[n];
    for (int i = 0; i < n; i++) {
        int x, y;
        cin >> x >> y;
        carr[i] = {{ld(x), ld(y)}, i};
    }
    Cd o = carr[0].first;
    sort(carr, carr + n, [&](const auto& a, const auto& b) -> bool {
        if (a.first == o) {
            return true;
        } else if (b.first == o) {
            return false;
        }
        ld ax = arg(a.first - o), bx = arg(b.first - o);
        assert(ax != bx);
        return ax < bx;
    });
    for (auto& a : carr) {
        dbg(a.first);
    }
    for (int i = 0; i + 3 <= n; i++) {
        assert(cross(carr[i].first, carr[i + 1].first, carr[i + 2].first) > 0);
    }
    for (int i = 0; i < n; i++) {
        if (sz(graph[i]) == 1) {
            pdfs(i);
            dfs(i, 0, n - 1);
            break;
        }
    }
    int cans[n];
    for (int i = 0; i < n; i++) {
        cans[carr[i].second] = ans[i];
    }
    for (int i = 0; i < n; i++) {
        cout << cans[i] + 1 << " \n"[i == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}