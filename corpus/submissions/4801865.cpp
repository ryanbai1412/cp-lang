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
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxa = 205, maxb = maxa * 2;

void solve() {
	int n;
	cin >> n;
	int arr[n][2];
	for (int i = 0; i < n - 1; i++) {
		cin >> arr[i][0] >> arr[i][1];
	}
	cin >> arr[n - 1][0];
	arr[n - 1][1] = 1e9;
	pair<int, bool> ml[n][maxb], mr[n][maxb];
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < maxb; j++) {
			{
				int cur = j, k;
				for (k = i; k >= 0; k--) {
					if (cur > arr[k][0]) {
						cur -= arr[k][0];
						if (k && cur >= arr[k - 1][1]) {
							cur += arr[k][0];
						}
					} else {
						break;
					}
				}
				ml[i][j] = {k, k >= 0 && cur == arr[k][0]};
			}
			{
				int cur = j, k;
				for (k = i; k < n; k++) {
					if (cur > arr[k][1]) {
						cur -= arr[k][1];
						if (cur >= arr[k + 1][0]) {
							cur += arr[k][1];
						}
					} else {
						break;
					}
				}
				mr[i][j] = {k, cur == arr[k][1]};
			}
		}
	}
	dbg(ml[1][10].first, ml[1][0].second);
	int dp[n + 1][2] {};
	for (int i = n - 1; i >= 0; i--) {
		for (int ax = 0; ax < 2; ax++) {
			for (int j = i; j < n; j++) {
				for (int k = 0; k < maxb; k++) {
					if ((!ax &&
						 (ml[j][k].first > i || (ml[j][k].first == i && !ml[j][k].second))) ||
						(ax && ml[j][k].first > i)) {
						dp[i][ax] = max(dp[i][ax], k + dp[mr[j][k].first + 1][mr[j][k].second]);
					}
				}
			}
			dbg(i, ax, dp[i][ax]);
		}
	}
	cout << dp[0][0] << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}