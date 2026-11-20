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

struct custom_hash {
	static uint64_t splitmix64(uint64_t x) {
		// http://xorshift.di.unimi.it/splitmix64.c
		x += 0x9e3779b97f4a7c15;
		x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9;
		x = (x ^ (x >> 27)) * 0x94d049bb133111eb;
		return x ^ (x >> 31);
	}

	size_t operator()(uint64_t x) const {
		static const uint64_t FIXED_RANDOM =
			chrono::steady_clock::now().time_since_epoch().count();
		return splitmix64(x + FIXED_RANDOM);
	}
};

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
	int ckx = (1 << (k - 1));
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
const int maxn = 66, mink = ksize, maxr = maxn / mink + 1;

int n, k, rows, cols, must[maxr][maxn];
vector<int> start;
unordered_map<int, int, custom_hash> memo[maxr][maxn];

int hash(const vector<int>& arr) {
	int ans = sz(arr);
	for (auto& a : arr) {
		ans = (ans * (n + 1)) + a;
	}
	return ans;
}

long dp(int i, int j, vector<int>& cur) {
	if (j == cols) {
		return 1;
	} else if (i == sz(cur)) {
		return dp(0, j + 1, cur);
	} else if (i * cols + j >= n) {
		int x = cur.back();
		cur.pop_back();
		long ans = dp(0, j + 1, cur);
		cur.push_back(x);
		return ans;
	}
	int key = hash(cur);
	auto it = memo[i][j].find(key);
	if (it != memo[i][j].end()) {
		return it->second;
	}
	long ans = 0;
	int cval = cur[i], cprev = i ? cur[i - 1] : 0;
	auto process = [&](int x) -> void {
		if ((i == 0 && j != cols - 1) || x < cprev + k / 2) {
			if (j < cols - 1 || i + 1 >= rows ||
				(x <= start[i + 1] && start[i + 1] <= x + 1)) {
				cur[i] = x;
				ans += dp(i + 1, j, cur);
				if (ans >= mod) {
					ans -= mod;
				}
			}
		}
	};
	if (must[i][j] != -1) {
		if (cval <= must[i][j] && must[i][j] <= cval + 1) {
			process(must[i][j]);
		}
	} else {
		process(cval);
		process(cval + 1);
	}
	cur[i] = cval;
	memo[i][j][key] = int(ans);
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
		} else if (!j && i > 1) {
			break;
		}
		cur.push_back(prev + i);
		feach(t);
		cur.pop_back();
	}
}

long solve(int _n, int _k, const string& s) {
	n = _n;
	k = _k;
	rows = (n + k - 1) / k;
	cols = k;
	memset(must, -1, sizeof(must));
	int psum = 0;
	for (int i = 0; i < sz(s); i++) {
		int j = i;
		must[j / k][j % k] = (psum += (s[i] == '1'));
	}
	long ans = 0;
	feach([&](vector<int> cur) -> void {
		start = cur;
		for (int i = 0; i < rows; i++) {
			for (int j = 0; j < cols; j++) {
				memo[i][j].clear();
			}
		}
		ans += dp(0, 1, cur);
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