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

void solve1(int n, int m) {
	int arr[m][4];
	for (auto& [i, j, k, x] : arr) {
		cin >> i >> j >> k >> x;
		i--;
		j--;
		k--;
	}
	int perm[n];
	iota(perm, perm + n, 0);
	do {
		bool val = true;
		int ans[n] {};
		for (auto& [i, j, k, x] : arr) {
			vector<pair<int, int>> cur;
			for (auto& a : {i, j, k}) {
				cur.emplace_back(perm[a], a);
			}
			sort(begin(cur), end(cur));
			int cmed = cur[1].second;
			if (ans[cmed] && ans[cmed] != x) {
				val = false;
				break;
			}
			ans[cmed] = x;
		}
		if (!val) {
			continue;
		}
		array<int, 3> cur[n];
		for (int i = 0; i < n; i++) {
			cur[i] = {perm[i], ans[i], i};
		}
		sort(cur, cur + n);
		for (int i = 0; i < n; i++) {
			if (cur[i][1]) {
				for (int j = i - 1; j >= 0 && !cur[j][1]; j--) {
					cur[j][1] = cur[i][1];
				}
				for (int j = i + 1; j < n && !cur[j][1]; j++) {
					cur[j][1] = cur[i][1];
				}
			}
		}
		for (int i = 0; i < n - 1; i++) {
			if (cur[i][1] > cur[i + 1][1]) {
				val = false;
				break;
			}
		}
		if (!val) {
			continue;
		}
		sort(cur, cur + n, [](const auto &a, const auto &b) -> bool {
			return a[2] < b[2];
		});
		cout << "YES" << endl;
		for (auto& [_, x, __] : cur) {
			cout << x << " ";
		}
		cout << endl;
		return;
	} while(next_permutation(perm, perm + n));
	cout << "NO" << endl;
}

void solve() {
	int n, m;
	cin >> n >> m;
	if (n <= 10 && m <= 10) {
		solve1(n, m);
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