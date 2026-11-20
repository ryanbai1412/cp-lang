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

using Key = array<int, 26>;

const long mod = 1e9 + 7;
const int maxn = 2e3 + 5;

long choose[maxn + 1][maxn + 1];

void pcomp() {
	for (int i = 0; i <= maxn; i++) {
		choose[i][0] = choose[i][i] = 1;
		for (int j = 1; j < i; j++) {
			choose[i][j] = choose[i - 1][j - 1] + choose[i - 1][j];
			if (choose[i][j] >= mod) {
				choose[i][j] -= mod;
			}
		}
	}
}

void solve() {
	int n, k;
	cin >> n >> k;
	map<Key, int> m;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
		Key key {};
		for (auto& a : s) {
			key[a - 'a']++;
		}
		m[key]++;
	}
	vector<int> arr;
	for (auto& [_, v] : m) {
		dbg(v);
		arr.push_back(v);
	}
	auto c2 = [&](int x) -> int {
		return x * (x - 1) / 2;
	};
	n = sz(arr);
	long dp[n + 1][k + 1] {};
	dp[n][k] = 1;
	for (int i = n - 1; i >= 0; i--) {
		for (int j = 0; j <= k; j++) {
			for (int cnt = 0; cnt <= arr[i]; cnt++) {
				int cj = j + c2(cnt);
				if (cj > k) {
					break;
				}
				dp[i][j] = (dp[i][j] + choose[arr[i]][cnt] * dp[i + 1][cj]) % mod;
			}
			dbg(i, j, dp[i][j]);
		}
	}
	cout << dp[0][0] << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}