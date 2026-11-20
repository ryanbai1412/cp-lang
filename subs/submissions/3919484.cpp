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

vector<int> solve(int n, vector<pair<int, int>> edges, vector<int> own) {
	vector<int> graph[n], igraph[n];
	for (auto& [u, v] : edges) {
		graph[u].push_back(v);
		igraph[v].push_back(u);
	}

	int dep[n]{};
	{
		for (int i = 0; i < n; i++) {
			dep[i] = sz(graph[i]);
		}
	}

	bool qd[n]{};
	vector<int> q;
	auto push = [&](int u, bool f = false) -> void {
		if (!qd[u] && (f || !dep[u])) {
			qd[u] = true;
			q.push_back(u);
		}
	};
	for (int i = 0; i < n; i++) {
		push(i);
	}

	vector<int> ans(n, -1);
	while (sz(q)) {
		int u = q.back();
		q.pop_back();

		if (ans[u] == -1) {
			ans[u] = false;
		}

		for (auto& v : igraph[u]) {
			dep[v]--;
			if (!own[v]) {
				ans[v] = false;
				push(v, true);
			} else {
				push(v);
			}
		}
	}

	for (auto& a : ans) {
		if (a == -1) {
			a = true;
		}
	}
	return ans;
}

vector<int> who_wins(vector<int> own,
				  vector<int> charging,
				  vector<int> us,
				  vector<int> vs) {
	int n = sz(own), m = sz(us);

	auto encode = [&](int u, int d) -> int { return d * n + u; };
	int maxn = (n + 1) * n;

	vector<pair<int, int>> edges;
	{
		for (int i = 0; i < m; i++) {
			int u = us[i], v = vs[i];
			if (charging[v]) {
				for (int j = 0; j < n; j++) {
					edges.emplace_back(encode(u, j), encode(v, 0));
				}
			} else {
				for (int j = 0; j < n; j++) {
					edges.emplace_back(encode(u, j), encode(v, j + 1));
				}
			}
		}
	}

	vector<int> nown(maxn);
	for (int i = 0; i < n; i++) {
		for (int j = 0; j <= n; j++) {
			nown[encode(i, j)] = own[i];
		}
	}

	auto cans = solve(maxn, edges, nown);
	vector<int> ans(n);

	for (int i = 0; i < n; i++) {
		ans[i] = cans[encode(i, 0)];
	}

	return ans;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, m;
	cin >> n >> m;
	vector<int> own(n), charging(n), us(m), vs(m);
	for (auto& a : own) {
		cin >> a;
	}
	for (auto& a : charging) {
		cin >> a;
	}
	for (int i = 0; i < m; i++) {
		cin >> us[i] >> vs[i];
	}
	auto ans = who_wins(own, charging, us, vs);
	assert(sz(ans) == n);
	for (auto& a : ans) {
		cout << a << " ";
	}
	cout << endl;
}