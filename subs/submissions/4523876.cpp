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

const int maxn = 2505;

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

bool found;
int n, p, q, decided[maxn];
vector<int> graph[maxn], cans, incl;

void dfs() {
	if (found || sz(incl) > p) {
		return;
	}
	int cq = 0;
	for (auto& a : incl) {
		for (auto& v : graph[a]) {
			if (!decided[v]) {
				{
					decided[v] = 1;
					incl.push_back(v);
					dfs();
					incl.pop_back();
					decided[v] = 0;
				}
				{
					decided[v] = -1;
					dfs();
					decided[v] = 0;
				}
				return;
			} else if (decided[v] == -1) {
				if (++cq > q) {
					return;
				}
			}
		}
	}
	cans = incl;
	found = true;
}

void solve() {
	cin >> n >> p >> q;
	vector<pair<int, int>> edges;
	for (int i = 0; i < n; i++) {
		auto& a = graph[i];
		int m;
		cin >> m;
		if (m > p + q - 1) {
			fail();
		}
		a.resize(m);
		for (auto& b : a) {
			cin >> b;
			edges.emplace_back(i, b);
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
	vector<int> blocks[n];
	for (int i = 0; i < n; i++) {
		incl = {i};
		decided[i] = 1;
		found = false;
		dfs();
		decided[i] = 0;
		if (!found) {
			fail();
		}
		blocks[i] = cans;
		sort(begin(blocks[i]), end(blocks[i]));
	}
	auto excl = [&](const vector<int> &a, const vector<int>& b) -> vector<int> {
		vector<int> ans;
		for (auto& x : a) {
			if (!binary_search(begin(b), end(b), x)) {
				ans.push_back(x);
			}
		}
		return ans;
	};
	auto check = [&](const vector<int>& arr) -> bool {
		static int vis[maxn] {}, vind = 0;
		vind++;
		if (sz(arr) > p) {
			return false;
		}
		int cnt = 0;
		for (auto& a : arr) {
			vis[a] = vind;
		}
		for (auto& a : arr) {
			for (auto& b : graph[a]) {
				if (vis[b] != vind) {
					if (++cnt > q) {
						return false;
					}
				}
			}
		}
		return true;
	};
	for (int i = 0; i < n; i++) {
		auto& a = blocks[i];
		for (int j = i + 1; j < n; j++) {
			auto& b = blocks[j];
			auto x = excl(a, b);
			if (check(x)) {
				a = x;
			} else {
				b = excl(b, a);
			}
		}
	}
	vector<vector<int>> ans;
	for (auto& a : blocks) {
		if (sz(a)) {
			ans.emplace_back(begin(a), end(a));
		}
	}
	cout << "home" << endl;
	print(ans);
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}