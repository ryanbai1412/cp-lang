#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

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

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
	}
	int cnt[55] {};
	for (auto& a : arr) {
		cnt[a]++;
	}
	sort(arr, arr + n, greater<>());
	int m = n * 50 + 5;
	bool dp[52][n][m];
	memset(dp, 0, sizeof(dp));
	int mx = *max_element(arr, arr + n);
	dp[51][0][mx] = true;
	cnt[mx]--;
	int suff[55] {};
	for (int i = 50; i >= 0; i--) {
		suff[i] = suff[i + 1] + cnt[i];
	}
	for (int i = 51; i >= 1; i--) {
		for (int j = suff[i + 1]; j < n; j++) {
			int req = suff[i], hav = j, need = max(0, req - hav);
			for (int sm = 0; sm < m; sm++) {
				if (dp[i][j][sm]) {
					if (cnt[i + 1] && j + 1 < n && sm + i + 1 < m) {
						dp[i][j + 1][sm + i + 1] = true;
					}
					{
						int k = need;
						if (!(k && !cnt[i])) {
							if (j + k < n && sm + i * k < m) {
								dp[i - 1][j + k][sm + i * k] = true;
							}
						}
					}
				}
			}
		}
	}
	vector<int> ans;
	int tot = accumulate(arr, arr + n, 0);
	for (int i = 0; i < m; i++) {
		if (dp[0][n - 1][i]) {
			ans.push_back(i - tot);
		}
	}
	for (int i = 0; i < sz(ans); i++) {
		cout << ans[i] << " \n"[i == sz(ans) - 1];
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}