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

struct DS {
	vector<int> v;

	DS(int n) : v(n, 1e9) {}

	void set(int ind, int x) {
		v[ind] = x;
	}

	void update(int l) {
		for (int i = l; i < sz(v); i++) {
			v[i]++;
		}
	}

	int query(int l) const {
		return *min_element(v.begin() + l, v.end());
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n];
	fill(arr, arr + n, n);
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		if (u > v) {
			swap(u, v);
		}
		arr[v] = min(arr[v], u);
	}
	vector<int> inds[n];
	for (int i = 0; i < n; i++) {
		if (arr[i] < n) {
			inds[arr[i]].push_back(i);
		}
	}
	int suff[n + 1];
	suff[n] = 1e9;
	for (int i = n - 1; i >= 0; i--) {
		suff[i] = min(suff[i + 1], arr[i]);
	}
	DS ds(n + 1);
	ds.set(n, 0);
	int j = n;
	for (int i = n - 1; i >= 1; i--) {
		while (j >= 0 && suff[j] >= i) {
			j--;
		}
		for (auto& a : inds[i]) {
			ds.update(a + 1);
		}
		int cur = ds.query(j + 1);
		if (arr[i] >= n) {
			cur++;
			ds.update(i + 1);
		}
		ds.set(i, cur);
		if (i == 1) {
			cout << cur << endl;
		}
	}
//	for (int i = n - 1; i >= 1; i--) {
//		dp[i] = 1e9;
//		int cur = 0;
//		for (int j = i + 1; j <= n; j++) {
//			cur += arr[j - 1] >= i;
//			if (suff[j] >= i) {
//				dp[i] = min(dp[i], cur + dp[j]);
//			}
//		}
//	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}