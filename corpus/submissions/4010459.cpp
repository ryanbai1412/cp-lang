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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 2e5;

struct Solver {
	int n;
	vector<int> arr;
	vector<vector<int>> graph;

	Solver(int n, const vector<int>& arr, const vector<pair<int, int>>& edges)
		: n(n), arr(arr), graph(n) {
		for (auto& [u, v] : edges) {
			graph[u].push_back(v);
			graph[v].push_back(u);
		}
		pdfs(0);
	}

	void pdfs(int u) {
		for (auto& v : graph[u]) {
			graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
			pdfs(v);
		}
	}

	vector<int> query(int u, int d) {
		bool vis[n] {};
		vector<int> ans;
		vector<pair<int, int>> q;
		auto push = [&](int u, int cd) -> void {
			if (!vis[u] && cd <= d) {
				vis[u] = true;
				ans.push_back(arr[u]);
				q.emplace_back(u, cd);
			}
		};
		push(u, 0);
		while (sz(q)) {
			auto [u, d] = q.back();
			q.pop_back();
			for (auto& v : graph[u]) {
				push(v, d + 1);
			}
		}
		return ans;
	}
};

struct Node {
	int v, lc, rc;
} heap[30 * maxn];

int alloc() {
	static int hind = 1;
	return hind++;
}

void maintain(int o) {
	heap[o].v = 0;
	if (heap[o].lc) {
		heap[o].v += heap[heap[o].lc].v;
	}
	if (heap[o].rc) {
		heap[o].v += heap[heap[o].rc].v;
	}
}

void update(int& o, int l, int r, int ind, int x) {
	int po = o;
	o = alloc();
	heap[o] = heap[po];
	if (l == r) {
		heap[o].v += x;
		return;
	}
	int mid = (l + r) / 2;
	if (ind <= mid) {
		update(heap[o].lc, l, mid, ind, x);
	} else {
		update(heap[o].rc, mid + 1, r, ind, x);
	}
	heap[o].v += x;
}

int query(int o, int l, int r, int ql, int qr) {
	if (!o) {
		return 0;
	} else if (ql <= l && r <= qr) {
		return heap[o].v;
	}
	int mid = (l + r) / 2, ans = 0;
	if (ql <= mid) {
		ans += query(heap[o].lc, l, mid, ql, qr);
	}
	if (mid < qr) {
		ans += query(heap[o].rc, mid + 1, r, ql, qr);
	}
	return ans;
}

void merge(int& a, int b, int l, int r) {
	if (!b) {
		return;
	}
	int pa = a;
	a = alloc();
	if (pa) {
		heap[a] = heap[pa];
	}
	if (!a) {
		heap[a] = heap[b];
		return;
	} else if (l == r) {
		heap[a].v += heap[b].v;
		return;
	}
	int mid = (l + r) / 2;
	merge(heap[a].lc, heap[b].lc, l, mid);
	merge(heap[a].rc, heap[b].rc, mid + 1, r);
	maintain(a);
}

int cn, arr[maxn], depth[maxn], siz[maxn], seg[maxn];
vector<int> graph[maxn], compute[maxn], computed[maxn];

void pdfs(int u, int d = 0) {
	depth[u] = d;
	siz[u] = 1;
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v, d + 1);
		siz[u] += siz[v];
		if (siz[v] > siz[graph[u][0]]) {
			swap(graph[u][0], v);
		}
	}
}

map<int, int> dfs(int u) {
	int& cseg = seg[u];
	map<int, int> cur;
	cur[arr[u]] = depth[u];
	update(cseg, 0, cn - 1, depth[u], 1);
	for (auto& v : graph[u]) {
		auto x = dfs(v);
		if (v == graph[u][0]) {
			cseg = seg[v];
			update(cseg, 0, cn - 1, depth[u], 1);
			auto it = x.find(arr[u]);
			if (it != x.end()) {
				update(cseg, 0, cn - 1, it->second, -1);
			}
			swap(cur, x);
			cur[arr[u]] = depth[u];
		} else {
			merge(cseg, seg[v], 0, cn - 1);
			for (auto& [k, v] : x) {
				auto [it, inserted] = cur.try_emplace(k, v);
				if (!inserted) {
					if (v < it->second) {
						update(cseg, 0, cn - 1, it->second, -1);
						it->second = v;
					} else {
						update(cseg, 0, cn - 1, v, -1);
					}
				}
			}
		}
	}
	for (auto& a : compute[u]) {
		auto it = cur.find(a);
		if (it != cur.end()) {
			computed[u].push_back(it->second);
		} else {
			computed[u].push_back(1e9);
		}
	}
	return cur;
}

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	cn = m;
	vector<pair<int, int>> e1(n - 1);
	for (auto& [u, v] : e1) {
		cin >> u >> v;
		u--;
		v--;
	}
	for (int i = 0; i < m - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	vector<int> a1(n);
	for (auto& a : a1) {
		cin >> a;
	}
	for (int i = 0; i < m; i++) {
		cin >> arr[i];
	}
	vector<array<int, 5>> queries;
	while (q--) {
		int t;
		cin >> t;
		array<int, 5> cur {};
		if (t == 1) {
			int u, du, v, dv;
			cin >> u >> v >> du >> dv;
			u--;
			v--;
			cur = {t, u, du, v, dv};
		} else {
			int u, x;
			cin >> u >> x;
			u--;
			cur = {t, u, x};
		}
		queries.push_back(cur);
	}
	{
		vector<int> cur = a1;
		for (auto& a : queries) {
			if (a[0] == 1) {
				auto& [t, u, du, v, dv] = a;
				for (auto& b : cur) {
					compute[v].push_back(b);
				}
			} else {
				auto& [t, u, x, _, __] = a;
				cur[u] = x;
			}
		}
		for (int i = 0; i < m; i++) {
			sort(begin(compute[i]), end(compute[i]));
			compute[i].erase(unique(begin(compute[i]), end(compute[i])),
							 compute[i].end());
		}
	}
	pdfs(0);
	dfs(0);
	Solver solver(n, a1, e1);
	int ans = 0;
	for (auto& a : queries) {
		if (a[0] == 1) {
			auto& [t, u, du, v, dv] = a;
			du ^= ans;
			dv ^= ans;
			vector<int> vals = solver.query(u, du);
			sort(begin(vals), end(vals));
			vals.erase(unique(begin(vals), end(vals)), vals.end());
			ans = sz(vals) + query(seg[v], 0, cn - 1, 0, depth[v] + dv);
			dbg(sz(vals), query(seg[v], 0, cn - 1, 0, depth[v] + dv));
			for (auto& a : vals) {
				auto ind = lower_bound(begin(compute[v]), end(compute[v]), a) -
						   compute[v].begin();
				if (computed[v][ind] <= depth[v] + dv) {
					ans--;
				}
			}
			cout << ans << endl;
		} else {
			auto& [t, u, x, _, __] = a;
			solver.arr[u] = x;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}