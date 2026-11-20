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

const int maxm = 16;

struct DS {
	int n, m;
	vector<vector<long>> v;
	
	explicit DS(const vector<long> &arr): n(sz(arr)), m(__lg(n) + 3), v(m, vector<long> (n)) {
		v[0] = arr;
		for (int i = 1; i < m; i++) {
			for (int j = 0; j + (1 << i) <= n; j++) {
				v[i][j] = gcd(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
			}
		}
	}
	
	long query(int l, int r) const {
		int k = __lg(r - l + 1);
		return gcd(v[k][l], v[k][r - (1 << k) + 1]);
	}
};

void solve() {
	int n, q;
	cin >> n >> q;
	int queries[q][3];
	for (auto& [l, r, x] : queries) {
		cin >> l >> r >> x;
		l--;
		r--;
	}
	vector<int> evts[n + 1];
	for (auto& [l, r, x] : queries) {
		evts[l].push_back(x);
		evts[r + 1].push_back(-x);
	}
	int cnt[maxm + 1] {};
	long arr[n];
	for (int i = 0; i < n; i++) {
		for (auto& a : evts[i]) {
			if (a < 0) {
				cnt[-a]--;
			} else {
				cnt[a]++;
			}
		}
		long x = 1;
		for (int j = 1; j <= maxm; j++) {
			if (cnt[j]) {
				x = lcm(x, j);
			}
		}
		arr[i] = x;
	}
	DS ds(vector<long> (arr, arr + n));
	for (auto& [l, r, x] : queries) {
		if (ds.query(l, r) != x) {
			cout << "Impossible" << endl;
			return;
		}
	}
	for (int i = 0; i < n; i++) {
		cout << arr[i] << " \n"[i == n - 1];
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