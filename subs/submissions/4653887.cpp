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

struct DSU {
	vector<int> p;

	DSU(int n) : p(n, -1) {}

	int find(int u) {
		return p[u] < 0 ? u : (p[u] = find(p[u]));
	}

	int size(int u) {
		return -p[find(u)];
	}

	void merge(int u, int v) {
		u = find(u);
		v = find(v);
		if (u == v) {
			return;
		}
		if (p[u] < p[v]) {
			swap(u, v);
		}
		p[v] += p[u];
		p[u] = v;
	}
};

long c2(long x) {
	return x * (x + 1) / 2;
}

struct DS {
	int n;
	vector<int> arr;
	DSU dsu;
	long ans;

	DS(const vector<int>& arr) : n(sz(arr)), arr(arr), dsu(n), ans(0) {
		for (int i = 0; i < n - 1; i++) {
			if (!arr[i] && !arr[i + 1]) {
				dsu.merge(i, i + 1);
			}
		}
		for (int i = 0; i < n;) {
			for (; i < n && arr[i]; i++)
				;
			int start = i;
			for (; i < n && !arr[i]; i++)
				;
			ans += c2(i - start);
		}
	}

	void erase(int u) {
		arr[u]--;
		if (arr[u]) {
			return;
		}
		dbg(ans);
		if (u && !arr[u - 1]) {
			ans -= c2(dsu.size(u - 1));
			dsu.merge(u - 1, u);
		}
		dbg(ans);
		if (u + 1 < n && !arr[u + 1]) {
			ans -= c2(dsu.size(u + 1));
			dsu.merge(u, u + 1);
		}
		dbg(ans, dsu.size(u));
		ans += c2(dsu.size(u));
	}

	long solve() {
		return ans;
	}
};

const int M = 5;

void solve() {
	{
		DS ds({1, 1});
		dbg(ds.solve());
		ds.erase(0);
		dbg(ds.solve());
		ds.erase(1);
		dbg(ds.solve());
//		return;
	}
	int n;
	cin >> n;
	string arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int last[n][M];
	memset(last, -1, sizeof(last));
	long ans = 0;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			if (arr[i][j] != '.') {
				last[j][arr[i][j] - 'A'] = i;
			}
		}
		vector<DS> v;
		for (int j = 0; j < (1 << M); j++) {
			vector<int> cur(n);
			for (int k = 0; k < n; k++) {
				for (int a = 0; a < 5; a++) {
					if ((j >> a) & 1) {
						cur[k] += last[k][a] >= 0;
					}
				}
			}
			v.emplace_back(cur);
		}
		vector<pair<int, int>> imp[n];
		for (int j = 0; j < n; j++) {
			for (int k = 0; k < M; k++) {
				if (last[j][k] >= 0) {
					imp[last[j][k]].emplace_back(j, k);
				}
			}
		}
		for (int j = 0; j <= i; j++) {
			for (int k = 0; k < (1 << M); k++) {
				dbg(i, j, k, v[k].solve());
				if (__builtin_popcount(k) & 1) {
					ans -= v[k].solve();
				} else {
					ans += v[k].solve();
				}
			}
			for (auto& [k, x] : imp[j]) {
				for (int a = 0; a < (1 << M); a++) {
					if ((a >> x) & 1) {
						dbg(a, k);
						v[a].erase(k);
					}
				}
			}
			dbg(ans);
		}
		dbg(ans);
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}