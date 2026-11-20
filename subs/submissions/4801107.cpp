#include "bits/extc++.h"
#include "count.h"

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
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int maxn = 2e3 + 5;

info merge_r(info a, info b) {
    if (isempty(a)) {
        return b;
    } else if (isempty(b)) {
        return a;
    }
    return MR(a, b);
}

info merge_c(info a, info b) {
    if (isempty(a)) {
        return b;
    } else if (isempty(b)) {
        return a;
    }
    return MC(a, b);
}

vector<pair<int, info>> graph[maxn];
int tc;
info ans1[maxn][maxn];

struct Dfs1 {
    int dcnt[maxn] {};

    void dfs(int u, int p = -1, int d = 0) {
        dcnt[d]++;
        for (auto& [v, _] : graph[u]) {
            if (v != p) {
                dfs(v, u, d + 1);
            }
        }
    }

    static int solve(int u, int m) {
        Dfs1 d;
        d.dfs(u);
        auto& arr = d.dcnt;
        int psum = 0;
        for (int i = 0; i < sz(arr); i++) {
            psum += arr[i];
            if (psum >= m) {
                return i;
            }
        }
        return sz(arr);
    }
};

struct Dfs2 {
    int depth;

    info dfs(int u, int p = -1, int d = 0) {
        if (d >= depth) {
            return emptyinfo;
        }
        info ans = emptyinfo;
        for (auto& [v, w] : graph[u]) {
            if (v != p) {
                ans = merge_r(merge_r(w, dfs(v, u, d + 1)), ans);
            }
        }
        return ans;
    }

    static info solve(int u, int depth) {
        Dfs2 d {depth};
        return d.dfs(u);
    }
};

void init(int _tc, int n, int, vector<int> p, vector<info> e, int M) {
    tc = _tc;
    for (auto& a : p) {
        a--;
    }
    p.insert(p.begin(), -1);
    e.insert(e.begin(), emptyinfo);
    for (int i = 1; i < n; i++) {
        graph[i].emplace_back(p[i], e[i]);
        graph[p[i]].emplace_back(i, e[i]);
    }
    for (int i = 0; i < n; i++) {
        fill(ans1[i], ans1[i] + n, emptyinfo);
        for (int j = Dfs1::solve(i, M); j < n; j++) {
            ans1[i][j] = Dfs2::solve(i, j);
        }
    }
}

info ask(int u, int d) {
    u--;
    if (!d) {
        return emptyinfo;
    }
    if (!isempty(ans1[u][d])) {
        return ans1[u][d];
    }
    return Dfs2::solve(u, d);
}