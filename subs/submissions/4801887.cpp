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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 6005;

bool dp[maxn][maxn][2];
int n, opt[maxn][maxn][2];

vector<int> arr;
vector<pair<int, int>> ans;

void del(int ind, int x) {
	dbg(ind, x);
	assert(x == 2 || x == 3);
	for (int i = 0; i < x; i++) {
		arr.erase(arr.begin() + ind);
	}
	ans.emplace_back(ind, x);
}

void solve(int l, int j, int k, int ind) {
	assert(dp[l][j][k]);
	if (l == n) {
		return;
	}
	int o = opt[l][j][k];
	if (o == 0) {
		if (dp[l + 1][0][0]) {
			solve(l + 1, 0, 0, ind + 1);
		} else {
			solve(l + 1, 0, 1, ind + 1);
		}
	} else if (o == 1) {
		solve(l + 1, j, k, ind + 1);
	} else if (o == 2) {
		solve(l + 1, j + 1, k ^ 1, ind + 1);
		int r;
		for (r = ind; r < sz(arr) && arr[ind] == arr[r]; r++) ;
		int len = r - ind;
		assert(len >= 2);
		if (len & 1) {
			del(ind, 3);
			len -= 3;
		}
		while (len) {
			del(ind, 2);
			len -= 2;
		}
	} else {
		solve(l + 1, j - 1, k ^ 1, ind + 1);
	}
}

void solve() {
	cin >> n;
	string s;
	cin >> s;
	arr.resize(n);
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] == 'A';
	}
	dp[n][0][0] = dp[n][0][1] = true;
	for (int i = n; i >= 1; i--) {
		int x = arr[i - 1];
		if (dp[i][0][0] || dp[i][0][1]) {
			dp[i - 1][1][arr[i - 1]] = true;
			opt[i - 1][1][arr[i - 1]] = 0;
		}
		for (int j = 1; j < n; j++) {
		    for (int k = 0; k < 2; k++) {
				if (dp[i][j][k]) {
					if (k == x) {
						dp[i - 1][j][k] = true;
						opt[i - 1][j][k] = 1;
						if (j) {
							dp[i - 1][j - 1][k ^ 1] = true;
							opt[i - 1][j - 1][k ^ 1] = 2;
						}
					} else {
						dp[i - 1][j + 1][k ^ 1] = true;
						opt[i - 1][j + 1][k ^ 1] = 3;
					}
				}
		    }
		}
	}
	if (!dp[0][0][0] && !dp[0][0][1]) {
		cout << -1 << endl;
		return;
	}
#ifdef STRESS
	cout << "YES" << endl;
#else
	if (dp[0][0][0]) {
		solve(0, 0, 0, 0);
	} else {
		solve(0, 0, 1, 0);
	}
	cout << sz(ans) << endl;
	for (auto& [x, y] : ans) {
		cout << x + 1 << " " << y << endl;
	}
#endif
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}