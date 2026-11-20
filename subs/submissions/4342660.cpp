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

const long mod = 1e9 + 7;

namespace s1 {

const int maxn = 305;

long pow2[maxn + 1];

void pcomp() {
	pow2[0] = 1;
	for (int i = 0; i < maxn; i++) {
		pow2[i + 1] = (pow2[i] * 2) % mod;
	}
}

long solve1d(const string& s) {
	long ans = 1;
	for (int i = 0; i < sz(s);) {
		int l = i;
		for (; i < sz(s) && s[i] == s[l]; i++)
			;
		ans = (ans * pow2[i - l - 1]) % mod;
	}
	return ans;
}

void solve(int n, int m) {
	assert(n == 1);
	string s;
	cin >> s;
	for (int i = 0; i < m; i++) {
		char c = s[i];
		if (c == 'N') {
			s[i] = 'S';
		} else {
			s[i] = 'N';
		}
		cout << solve1d(s) << " \n"[i == m - 1];
		s[i] = c;
	}
}

}  // namespace s1

const int maxn = 300;

int n, m;
bool arr[maxn][maxn];
long memo[maxn][3][1 << 17];

long dp(int x, int y, int prevh, int mask) {
	prevh = min(prevh, 2);
	if (y == m) {
		if (prevh == 1) {
			return 0;
		}
		return dp(x + 1, 0, 0, mask);
	}
	if (x == n) {
		return 1;
	}
	int key = x * m + y;
	long &ans = memo[key][prevh][mask];
	if (ans != -1) {
		return ans;
	}
	ans = 0;
	int on = 1 << y;
	int off = ~on;
	// stop vertical
	{
		// start vertical
		if (prevh != 1) {
			ans += dp(x, y + 1, 0, mask | on);
		}
		// start horizontal
		if (prevh != 1) {
			ans += dp(x, y + 1, 1, mask & off);
		}
		// continue horizontal
		if (prevh && arr[x][y - 1] == arr[x][y]) {
			ans += dp(x, y + 1, prevh + 1, mask & off);
		}
	}
	// continue vertical
	if (prevh != 1 && (mask >> y) & 1 && arr[x - 1][y] == arr[x][y]) {
		ans += dp(x, y + 1, 0, mask | on);
	}
	ans %= mod;
	return ans;
}

long solvex() {
	for (int i = 0; i < n * m; i++) {
		for (int j = 0; j < 3; j++) {
			fill(memo[i][j], memo[i][j] + (1 << m), -1);
		}
	}
	return dp(0, 0, 0, 0);
}

void solve() {
	cin >> n >> m;
	if (n == 1) {
		s1::solve(n, m);
		return;
	}
	bool swapped = false;
	if (n < m) {
		swapped = true;
		swap(n, m);
		for (int i = 0; i < m; i++) {
			string s;
			cin >> s;
			for (int j = 0; j < n; j++) {
				arr[j][i] = s[j] == 'N';
			}
		}
	} else {
		for (int i = 0; i < n; i++) {
			string s;
			cin >> s;
			for (int j = 0; j < m; j++) {
				arr[i][j] = s[j] == 'N';
			}
		}
	}
	long ans[n][m];
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < m; j++) {
			arr[i][j] ^= true;
			ans[i][j] = solvex();
			arr[i][j] ^= true;
		}
	}
	if (swapped) {
		for (int i = 0; i < m; i++) {
			for (int j = 0; j < n; j++) {
				cout << ans[j][i] << " \n"[j == n - 1];
			}
		}
	} else {
		for (int i = 0; i < n; i++) {
			for (int j = 0; j < m; j++) {
				cout << ans[i][j] << " \n"[j == m - 1];
			}
		}
	}
}

int main() {
	s1::pcomp();
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