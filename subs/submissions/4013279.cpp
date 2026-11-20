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

const int ksize = 20;
const long mod = 998244353;

namespace S1 {
const int maxn = 114, maxk = ksize;

int n, k;
long memo[maxn][1 << maxk];

void solve(int _n, int _k, int m, const string& s) {
	n = _n;
	k = _k;
	int cur = 0;
	for (int i = 0; i < k; i++) {
		if (i < m && s[i] == '1') {
			cur |= 1 << i;
		}
	}
	bool val[1 << k];
	int dp[1 << k];
	for (int i = 0; i < (1 << k); i++) {
		val[i] = __builtin_popcount(i) != k / 2;
		dp[i] = val[i];
	}
	int ckx = (1 << k - 1);
	for (int i = n - 1; i >= k; i--) {
		int ndp[1 << k] {};
		for (int i = 0; i < (1 << k); i++) {
			if (val[i]) {
				ndp[i] = dp[i >> 1] + dp[(i >> 1) | ckx];
				if (ndp[i] >= mod) {
					ndp[i] -= mod;
				}
			}
		}
		memcpy(dp, ndp, sizeof(dp));
	}
	long ans = 0;
	for (int i = 0; i < (1 << (k - m)); i++) {
		ans += dp[cur | (i << m)];
	}
	cout << ans % mod << endl;
}
}  // namespace S1

namespace S2 {
const int maxn = 114, mink = ksize;

int n, k, rows, cols, must[maxn / mink][maxn];
vector<int> start;
map<vector<int>, int> memo[maxn / mink][maxn];

long dp(int i, int j, vector<int> cur) {
	if (j == cols) {
		for (int i = 0; i < sz(cur); i++) {
			if (i + 1 < rows && !(cur[i] <= start[i + 1] &&
				start[i + 1] <= cur[i] + 1)) {
				return 0;
			}
		}
		return 1;
	} else if (i == sz(cur)) {
		return dp(0, j + 1, cur);
	} else if (i * cols + j > n) {
		cur.pop_back();
		return dp(0, j + 1, cur);
	}
	auto it = memo[i][j].find(cur);
	if (it != memo[i][j].end()) {
		return it->second;
	}
	long ans = 0;
	auto process = [&](int x) -> void {
		if (cur[i] <= x && x <= cur[i] + 1) {
			if (i == 0 || x < cur[i - 1] + k / 2) {
				vector<int> nxt = cur;
				nxt[i] = x;
				ans += dp(i + 1, j, nxt);
				if (ans >= mod) {
					ans -= mod;
				}
			}
		}
	};
	if (must[i][j] != -1) {
		process(must[i][j]);
	} else {
		process(cur[i]);
		process(cur[i] + 1);
	}
	memo[i][j][cur] = int(ans);
	return ans;
}

template <typename T>
void feach(const T& t) {
	static vector<int> cur;
	if (sz(cur) == rows) {
		t(cur);
		return;
	}
	int j = sz(cur);
	int prev = sz(cur) ? cur.back() : 0;
	for (int i = 0; i < k / 2; i++) {
		if (must[j][0] != -1 && must[j][0] != prev + i) {
			continue;
		}
		cur.push_back(prev + i);
		feach(t);
		cur.pop_back();
	}
}

long solve(int _n, int _k, const string& s) {
	n = _n;
	k = _k;
	rows = (n + 1 + k - 1) / k;
	cols = k;
	memset(must, -1, sizeof(must));
	int psum = 0;
	must[0][0] = 0;
	for (int i = 0; i < sz(s); i++) {
		int j = i + 1;
		must[j / k][j % k] = (psum += (s[i] == '1'));
	}
	long ans = 0;
	feach([&](const vector<int>& cur) -> void {
		start = cur;
		for (int i = 0; i < rows; i++) {
			for (int j = 0; j < cols; j++) {
				memo[i][j].clear();
			}
		}
		ans += dp(0, 1, cur);
		dbg(cur[0], cur[1], ans);
	});
	dbg(ans);
	return ans;
}

void solve(int n, int k, int _, string s) {
	long ans = solve(n, k, s);
	for (auto& a : s) {
		if (a == '0') {
			a = '1';
		} else {
			a = '0';
		}
	}
	ans += solve(n, k, s);
	cout << ans % mod << endl;
}
}  // namespace S2

void solve() {
	int n, k, m;
	cin >> n >> k >> m;
	string s;
	if (m) {
		cin >> s;
	}
	if (k <= ksize) {
		S1::solve(n, k, m, s);
	} else {
		S2::solve(n, k, m, s);
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