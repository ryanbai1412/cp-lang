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

const int maxn = 3e3 + 5, maxk = 3e3 + 5;

int k, arr[maxk];
long memo[maxn][2];

long dp(int n, bool par) {
	long &ans = memo[n][par];
	if (ans != -1) {
		return ans;
	}
	if (n == 1) {
		return arr[par];
	}
	long f[k + 1][n];
	memset(f, 0x3f, sizeof(f));
	f[0][0] = 0;
	for (int i = 1; i <= k; i++) {
		for (int j = 0; j < n; j++) {
			for (int x = 1; x + j < n; x++) {
				f[i][j + x] = min(f[i][j + x], f[i - 1][j] + dp(x, true));
			}
		}
	}
	ans = 1e18;
	for (int i = 1; i + par <= k; i++) {
		ans = min(ans, f[i][n - 1] + arr[i + par]);
	}
	return ans;
}

void solve() {
	int n;
	cin >> n >> k;
	for (int i = 1; i <= k; i++) {
		cin >> arr[i];
	}
	memset(memo, -1, sizeof(memo));
	cout << dp(n, false) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}