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

namespace s1 {
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

void solve(int n, int m, int q) {
	vector<pair<int, int>> e1(n - 1), e2(m - 1);
	for (auto& [u, v] : e1) {
		cin >> u >> v;
		u--;
		v--;
	}
	for (auto& [u, v] : e2) {
		cin >> u >> v;
		u--;
		v--;
	}
	vector<int> a1(n), a2(m);
	for (auto& a : a1) {
		cin >> a;
	}
	for (auto& a : a2) {
		cin >> a;
	}
	Solver s1(n, a1, e1);
	Solver s2(m, a2, e2);
	int prev = 0;
	while (q--) {
		int t;
		cin >> t;
		if (t == 1) {
			int u1, u2, d1, d2;
			cin >> u1 >> u2 >> d1 >> d2;
			d1 ^= prev;
			d2 ^= prev;
			u1--;
			u2--;
			vector<int> cur;
			{
				auto x = s1.query(u1, d1);
				cur.insert(cur.end(), begin(x), end(x));
			}
			{
				auto x = s2.query(u2, d2);
				cur.insert(cur.end(), begin(x), end(x));
			}
			sort(begin(cur), end(cur));
			prev = int(unique(begin(cur), end(cur)) - cur.begin());
			cout << prev << endl;
		} else {
			int u, x;
			cin >> u >> x;
			u--;
			s1.arr[u] = x;
		}
	}
}
}  // namespace s1

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	if (max({n, m, q}) <= 2000) {
		s1::solve(n, m, q);
		return;
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