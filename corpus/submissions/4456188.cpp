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

//#define STRESS

void fail() {
	cout << "detention" << endl;
	exit(0);
}

void print(const vector<vector<int>>& ans) {
#ifndef STRESS
	cout << sz(ans) << endl;
	for (auto& a : ans) {
		cout << sz(a);
		for (auto& b : a) {
			cout << " " << b;
		}
		cout << endl;
	}
#endif
}

bool on(int msk, int bit) {
	return (msk >> bit) & 1;
}

template <typename T>
void fsubmask(int x, const T& t) {
	for (int i = x; i; i = (i - 1) & x) {
		t(i);
	}
}

namespace s1 {

void solve(int n, int p, int q, vector<int> graph[]) {
	bool valid[1 << n] {};
	for (int i = 1; i < (1 << n); i++) {
		if (__builtin_popcount(i) > p) {
			continue;
		}
		int cnt = 0;
		for (int j = 0; j < n; j++) {
			if (on(i, j)) {
				for (auto& a : graph[j]) {
					cnt += !on(i, a);
				}
			}
		}
		valid[i] = cnt <= q;
	}
	bool dp[1 << n] {};
	dp[0] = true;
	for (int i = 1; i < (1 << n); i++) {
		fsubmask(i, [&](int j) -> void {
			if (valid[j]) {
				dp[i] |= dp[i ^ j];
			}
		});
	}
	int cur = (1 << n) - 1;
	if (!dp[cur]) {
		fail();
	}
	cout << "home" << endl;
	vector<int> groups;
	while (cur) {
		int nxt = -1;
		fsubmask(cur, [&](int j) -> void {
			if (valid[j] && dp[cur ^ j]) {
				nxt = j;
			}
		});
		assert(nxt != -1);
		groups.push_back(nxt);
		cur ^= nxt;
	}
	cout << sz(groups) << endl;
	for (auto& a : groups) {
		cout << __builtin_popcount(a);
		for (int i = 0; i < n; i++) {
			if (on(a, i)) {
				cout << " " << i;
			}
		}
		cout << endl;
	}
}

}  // namespace s1

const int maxn = 2505;

int n, mc, cind, comp[maxn];
vector<int> graph[maxn], ccs[maxn];

void dfs(int u) {
	if (comp[u] == cind) {
		return;
	}
	comp[u] = cind;
	for (auto& v : graph[u]) {
		dfs(v);
	}
}

void solve0() {
	for (int i = 0; i < cind; i++) {
		if (sz(ccs[i]) > mc) {
			fail();
		}
	}
	cout << "home" << endl;
	print(vector<vector<int>>(ccs, ccs + cind));
}

namespace q1 {

bool vis[maxn];
vector<int> gt[maxn], cvis;
int t, csz, found, tin[maxn], siz[maxn], low[maxn];

void dfs(int u, int p = -1) {
	siz[u] = 1;
	tin[u] = t++;
	vis[u] = true;
	low[u] = tin[u];
	for (auto& v : graph[u]) {
		if (v == p) {
			continue;
		}
		if (vis[v]) {
			low[u] = min(low[u], tin[v]);
		} else {
			gt[u].push_back(v);
			dfs(v, u);
			siz[u] += siz[v];
			low[u] = min(low[u], low[v]);
			if (tin[u] < low[v] && siz[v] <= mc && csz - siz[v] <= mc) {
				found = v;
			}
		}
	}
}

void dfs1(int u) {
	cvis.push_back(u);
	for (auto& v : gt[u]) {
		dfs1(v);
	}
}

void solve() {
	for (int i = 0; i < cind; i++) {
		if (sz(ccs[i]) > mc * 2) {
			fail();
		}
	}
	vector<vector<int>> ans;
	for (int i = 0; i < cind; i++) {
		if (sz(ccs[i]) <= mc) {
			ans.push_back(ccs[i]);
			continue;
		}
		csz = sz(ccs[i]);
		found = -1;
		t = 0;
		dfs(ccs[i][0]);
		if (found == -1) {
			fail();
		}
		cvis.clear();
		dfs1(found);
		ans.push_back(cvis);
		static bool xvis[maxn] {};
		for (auto& a : cvis) {
			xvis[a] = true;
		}
		vector<int> cur;
		for (auto& a : ccs[i]) {
			if (!xvis[a]) {
				cur.push_back(a);
			}
		}
		ans.push_back(cur);
	}
	cout << "home" << endl;
	print(ans);
}

}  // namespace q1

namespace q2 {

bool vis[maxn], vis1[maxn], bad[maxn][maxn], gtadj[maxn][maxn];
vector<int> gt[maxn], path;
vector<pair<int, int>> back;
vector<vector<int>> comps;
int t, tin[maxn], tout[maxn], siz[maxn], depth[maxn], bcnt[maxn], par[maxn], ru, rv;

void dfs(int u, int p = -1) {
	par[u] = p;
	tin[u] = t++;
	vis[u] = true;
	siz[u] = 1;
	for (auto& v : graph[u]) {
		if (v == p) {
			continue;
		}
		if (vis[v]) {
			if (tin[v] < tin[u]) {
				back.emplace_back(v, u);
			}
		} else {
			dbg("T", u, v);
			depth[v] = depth[u] + 1;
			gt[u].push_back(v);
			gt[v].push_back(u);
			gtadj[u][v] = gtadj[v][u] = true;
			dfs(v, u);
			siz[u] += siz[v];
		}
	}
	tout[u] = t - 1;
}

bool anc(int u, int v) {
	return tin[u] <= tin[v] && tout[v] <= tout[u];
}

bool checkt(int u, int v) {
	if (tin[u] > tin[v]) {
		swap(u, v);
	}
	if (bcnt[v]) {
		return false;
	}
	if (gtadj[u][v] && ru != -1) {
		if (anc(ru, u) && anc(v, rv)) {
			return true;
		}
	}
	return bad[u][v];
}

bool checkeq(int u, int v) {
	if (tin[u] > tin[v]) {
		swap(u, v);
	}
	return u == ru && v == rv;
}

void dfs1(int u) {
	if (vis1[u]) {
		return;
	}
	vis1[u] = true;
	comps.back().push_back(u);
	for (auto& v : graph[u]) {
		if (checkt(u, v)) {
			continue;
		} else if (checkeq(u, v)) {
			continue;
		}
		dfs1(v);
	}
}

void sdfs(int u, int p, int targ) {
	if (sz(path)) {
		return;
	}
	static vector<int> st;
	st.push_back(u);
	if (u == targ) {
		path = st;
		st.pop_back();
		return;
	}
	for (auto& v : gt[u]) {
		if (v != p) {
			sdfs(v, u, targ);
		}
	}
	st.pop_back();
}

void search(int u, int v) {
	path.clear();
	sdfs(u, -1, v);
}

void solve() {
	vector<vector<int>> ans;
	for (int i = 0; i < cind; i++) {
		auto& cc = ccs[i];
		if (sz(cc) <= mc) {
			ans.push_back(cc);
			continue;
		}
		t = 0;
		back.clear();
		dfs(cc[0]);
		for (auto [v, u] : back) {
			while (u != v) {
				bcnt[u]++;
				u = par[u];
			}
		}
		auto check = [&]() -> bool {
			comps.clear();
			for (auto& a : cc) {
				vis1[a] = false;
			}
			for (auto& a : cc) {
				if (!vis1[a]) {
					comps.emplace_back();
					dfs1(a);
					if (sz(comps.back()) > mc) {
						return false;
					}
				}
			}
			ans.insert(ans.end(), begin(comps), end(comps));
			return true;
		};
		vector<int> leaves;
		for (auto& a : cc) {
			if (sz(gt[a]) == 1) {
				leaves.push_back(a);
			}
		}
		ru = rv = -1;
		for (auto& a : leaves) {
			for (auto& b : leaves) {
				if (a < b) {
					search(a, b);
					for (int j = 0; j < sz(path) - 1; j++) {
						int u = path[j], v = path[j + 1];
						bad[u][v] = bad[v][u] = true;
					}
					if (check()) {
						goto loop;
					}
					for (int j = 0; j < sz(path) - 1; j++) {
						int u = path[j], v = path[j + 1];
						bad[u][v] = bad[v][u] = false;
					}
				}
			}
		}
		for (auto& [u, v] : back) {
			for (auto& a : cc) {
				bcnt[a] = 0;
			}
			for (auto [cu, cv] : back) {
				if (anc(cu, u) && anc(v, cv)) {
					continue;
				}
				while (cu != cv) {
					bcnt[cv]++;
					cv = par[cv];
				}
			}
			dbg(u, v);
			ru = u;
			rv = v;
			if (check()) {
				goto loop;
			}
		}
		fail();
	loop:;
	}
	cout << "home" << endl;
	print(ans);
}

}  // namespace q2

void solve() {
	int q;
	cin >> n >> mc >> q;
	for (int i = 0; i < n; i++) {
		auto& a = graph[i];
		int m;
		cin >> m;
		if (m > mc + q - 1) {
			fail();
		}
		a.resize(m);
		for (auto& b : a) {
			cin >> b;
		}
	}
	for (int i = 0; i < n; i++) {
		for (auto& v : graph[i]) {
			auto& gv = graph[v];
			if (find(begin(gv), end(gv), i) == gv.end()) {
				fail();
			}
		}
	}
	if (n <= 16) {
		s1::solve(n, mc, q, graph);
		return;
	}
	memset(comp, -1, sizeof(comp));
	for (int i = 0; i < n; i++) {
		if (comp[i] == -1) {
			dfs(i);
			cind++;
		}
	}
	for (int i = 0; i < n; i++) {
		ccs[comp[i]].push_back(i);
	}
	if (q == 0) {
		solve0();
	} else if (q == 1) {
		q1::solve();
	} else if (q == 2) {
		q2::solve();
	} else {
		assert(false);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}