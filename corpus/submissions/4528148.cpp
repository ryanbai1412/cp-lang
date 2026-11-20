#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

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
#define sz(x) int((x).size())

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B> &p) {
	return out << "p{" << p.first << ", " << p.second << "}";
}

template <typename T>
ostream& operator<<(ostream& out, const vector<T> &arr) {
	bool first = true;
	out << "v{";
	for (auto& a : arr) {
		if (!first) {
			out << ", ";
		}
		first = false;
		out << a;
	}
	out << "}";
	return out;
}

const int maxn = 205;

vector<int> subtree[maxn];
vector<pair<int, int>> graph[maxn];
int n, m, dist[maxn][maxn], arr[maxn], memo[maxn][maxn][maxn];

void pdfs(int u) {
	subtree[u].push_back(u);
	for (auto& [v, w] : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), pair<int, int> {u, w}));
		pdfs(v);
		subtree[u].insert(subtree[u].end(), begin(subtree[v]), end(subtree[v]));
	}
}

void merge(int* f, const vector<pair<int, int>>& g) {
	for (int i = m; i >= 0; i--) {
		int cur = 1e9;
		for (auto& [x, y] : g) {
			if (i >= x) {
				cur = min(cur, y + f[i - x]);
			} else {
				break;
			}
		}
		f[i] = cur;
	}
}

vector<pair<int, int>> construct(int *f) {
	vector<pair<int, int>> cans;
	for (int i = 0; i <= m; i++) {
		if (!i || f[i] != f[i - 1]) {
			cans.emplace_back(i, f[i]);
		}
	}
	return cans;
}

bool convex(const vector<pair<int, int>> &arr) {
	for (int i = 0; i < sz(arr) - 2; i++) {
		auto &[x1, y1] = arr[i];
		auto &[x2, y2] = arr[i + 1];
		auto &[x3, y3] = arr[i + 2];
		if ((y2 - y1) * (x3 - x2) > (y3 - y2) * (x2 - x1)) {
			return false;
		}
	}
	return true;
}

vector<vector<pair<int, int>>> dp(int u) {
	vector<vector<pair<int, int>>> ans(n), nans(n), child[max(1, sz(graph[u]))];
	for (int i = 0; i < sz(graph[u]); i++) {
		child[i] = dp(graph[u][i].first);
	}
	for (int v = 0; v < n; v++) {
		int f[m + 1];
		memset(f, 0x3f, sizeof(f));
		f[0] = dist[u][v];
		for (int i = 0; i < sz(graph[u]); i++) {
			merge(f, child[i][v]);
		}
		ans[v] = construct(f);
	}
	int cur[m + 1], ord[n];
	memset(cur, 0x3f, sizeof(cur));
	iota(ord, ord + n, 0);
	sort(ord, ord + n, [&](int a, int b) -> bool {
		return dist[u][a] < dist[u][b];
	});
	for (auto& a : ord) {
		int acur[m + 1], xcur[m + 1];
		{
			int j = 0, c = 1e9;
			for (int i = 0; i <= m; i++) {
				while (j < sz(ans[a]) && ans[a][j].first <= i) {
					c = min(c, ans[a][j].second);
					j++;
				}
				acur[i] = c;
			}
		}
		for (int i = 0; i <= m; i++) {
			xcur[i] = min(cur[i], acur[i]);
		}
		nans[a] = construct(xcur);
		for (int i = 1; i <= m; i++) {
			cur[i] = min(cur[i], acur[i - 1] + arr[a]);
		}
	}
	return nans;
}

void solve() {
	cin >> n >> m;
	m--;
	assert(n <= maxn);
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
		assert(arr[i] * maxn <= 1e9);
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		assert(w * maxn <= 1e9);
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	memset(dist, -1, sizeof(dist));
	for (int i = 0; i < n; i++) {
		vector<int> q;
		q.push_back(i);
		dist[i][i] = 0;
		while (sz(q)) {
			int u = q.back();
			q.pop_back();
			for (auto& [v, w] : graph[u]) {
				if (dist[i][v] == -1) {
					dist[i][v] = dist[i][u] + w;
					q.push_back(v);
				}
			}
		}
	}
	memset(memo, -1, sizeof(memo));
	pdfs(0);
	auto ans = dp(0);
	pair<int, int> mans {1e9, 1e9};
	for (int i = 0; i < n; i++) {
		auto &[up, cost] = ans[i].back();
		mans = min(mans, pair<int, int>{cost + arr[i], up + 1});
	}
	cout << mans.first << endl;
	cout << mans.second << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}