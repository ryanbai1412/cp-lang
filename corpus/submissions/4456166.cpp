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

const int maxn = 305, maxm = 2 * maxn * maxn, ninf = -1e9;

struct DS {
	vector<pair<int, int>> vals;
	
	void insert(int i, int x) {
		vals.emplace_back(i, x);
	}
	
	int query(int q) const {
		int ans = 1e9;
		for (auto& [i, x] : vals) {
			if (x >= q) {
				ans = min(ans, i);
			}
		}
		return ans;
	}
};

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	int arr[n], brr[m];
	for (auto& a : arr) {
		cin >> a;
	}
	for (auto& a : brr) {
		cin >> a;
	}
	for (auto& a : arr) {
		if (a < k) {
			cout << "Impossible" << endl;
			return;
		}
	}
	int c[maxm];
	fill(begin(c), end(c), ninf);
	c[0] = 0;
	for (auto& a : brr) {
		if (a < n) {
			continue;
		}
		for (int i = maxm - 1; i >= a; i--) {
			c[i] = max(c[i], 1 + c[i - a]);
		}
	}
	bitset<maxm> d;
	d.set(0);
	for (auto& a : brr) {
		if (a >= n) {
			continue;
		}
		d |= d << a;
	}
	int ans = 1e9, asum = accumulate(arr, arr + n, 0);
	DS ds;
	auto ins = [&](int j) -> void {
		if (0 <= j && j < maxm && c[j] >= 0) {
			dbg(j, c[j]);
			ds.insert(j, n * c[j]);
		}
	};
	for (int i = asum + 1; i < maxm; i++) {
		ins(i);
	}
	for (int i = 0; i < maxm; i++) {
		ins(asum - i);
		if (d[i]) {
			int j = ds.query(k * n - i);
			if (j < maxm) {
				ans = min(ans, j + i - asum);
			}
		}
	}
	if (ans < int(1e9)) {
		cout << ans << endl;
	} else {
		cout << "Impossible" << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}