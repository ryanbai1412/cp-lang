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

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
    tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

constexpr int maxn = 2e5 + 5;

int n, kv, arr[maxn], dp1[maxn], dp2[maxn];
pair<int, int> ans;
vector<int> graph[maxn];

void pdfs(int u) {
    if (u < n) {
        dp1[u] = arr[u];
        dp2[u] = -1;
        return;
    }
    for (int i = 0; i < sz(graph[u]); i++) {
        int v = graph[u][i];
        pdfs(v);
        dp1[u] = max(dp1[u], dp1[v]);
        if (i < sz(graph[u]) - 1) {
            dp2[u] = max(dp2[u], dp1[v]);
        } else {
            dp2[u] = max(dp2[u], dp2[v]);
        }
    }
    dbg(u, dp1[u], dp2[u]);
}

void dfs(int u, int cans = 0) {
    if (u < n) {
        dbg(u, cans);
        ans = max(ans, pair<int, int> {cans, -u});
        return;
    }
    cans += kv > dp2[u];
    for (auto& v : graph[u]) {
        dfs(v, cans);
    }
}

int GetBestPosition(int _n, int m, int _kv, int *_arr, int *ql, int *qr) {
    n = _n;
    kv = _kv;
    copy(_arr, _arr + n - 1, arr);
    ost<pair<int, int>> carr;
    for (int i = 0; i < n; i++) {
        carr.insert({i, i});
    }
    for (int i = 0; i < m; i++) {
        int l = ql[i], r = qr[i];
        auto it = carr.find_by_order(l);
        int ck = it->first;
        for (int j = 0; j < r - l + 1; j++) {
            dbg(i + n, it->second);
            graph[i + n].push_back(it->second);
            it = carr.erase(it);
        }
        carr.insert({ck, i + n});
    }
    int root = n + m - 1;
    pdfs(root);
    dfs(root);
    return -ans.second;
}// The original is the grader function GetBestPosition. This `main` stands in for the grader:
// n c r, the n - 1 ranks in order, then c lines `s e`; it prints the best position.
int main() {
	int n, c, r;
	if (scanf("%d %d %d", &n, &c, &r) != 3) return 1;
	vector<int> rank(n - 1), s(c), e(c);
	for (int& x : rank) scanf("%d", &x);
	for (int i = 0; i < c; i++) scanf("%d %d", &s[i], &e[i]);
	printf("%d\n", GetBestPosition(n, c, r, rank.data(), s.data(), e.data()));
}
