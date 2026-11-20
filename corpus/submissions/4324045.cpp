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

const int maxn = 16e6 + 5;

struct PFact {
	int minp[maxn + 1] {};

	PFact() {
		for (int i = 2; i <= maxn; i++) {
			if (minp[i]) {
				continue;
			}
			minp[i] = i;
			if (i <= maxn / i) {
				for (int j = i * i; j <= maxn; j += i) {
					if (!minp[j]) {
						minp[j] = i;
					}
				}
			}
		}
	}
	
	template <typename T>
	void fact(int x, const T &t, int cur = 1) {
		if (x == 1) {
			t(cur);
			return;
		}
		int cnt = 0, cv = minp[x];
		while (x % cv == 0) {
			x /= cv;
			cnt++;
		}
		fact(x, t, cur);
		for (int i = 0; i < cnt; i++) {
			cur *= cv;
			fact(x, t, cur);
		}
	}
} pfact;

int n, m;
vector<vector<int>> arr;
bitset<maxn> vis;

void dfs(int u) {
	vis[u] = true;
	pfact.fact(u, [&](int x) -> void {
		if (x <= n && u / x <= m) {
			int v = arr[x][u / x];
			if (!vis[v]) {
				dfs(v);
			}
		}
	});
}

bool can_escape(int _n, int _m, vector<vector<int>> _arr) {
	n = _n;
	m = _m;
	arr = _arr;
	dfs(arr[1][1]);
	return vis[n * m];
}

void solve() {
	int n, m;
	cin >> n >> m;
	vector<vector<int>> _arr(n + 1, vector<int>(m + 1));
	for (auto& a : _arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	if (can_escape(n, m, _arr)) {
		cout << "yes" << endl;
	} else {
		cout << "no" << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}