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
	auto is_median = [&](int i, int j, int k) -> bool {
		return min(j, k) <= i && i <= max(j, k);
	};
	do {
		int ans[n] {};
		for (auto& [i, j, k, x] : arr) {
			int ci = perm[i], cj = perm[j], ck = perm[k];
			int cmed = ci + cj + ck - min({ci, cj, ck}) - max({ci, cj, ck});
			if (ans[cmed] && ans[cmed] != x) {
				goto loop;
			}
			ans[cmed] = x;
		}
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < n; j++) {
				if (ans[j]) {
					if (j && !ans[j - 1]) {
						ans[j - 1] = ans[j];
					}
					if (j + 1 < n && !ans[j + 1]) {
						ans[j + 1] = ans[j];
					}
				}
			}
		}
		cout << "YES" << endl;
		for (auto& a : ans) {
			cout << a << " ";
		}
		cout << endl;
		return;
	loop:;
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