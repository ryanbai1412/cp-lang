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

const int logn = 12, maxn = 1000;
int tri[logn][maxn][maxn], sq[logn][maxn][maxn], arr[maxn][maxn];

void solve() {
	int n, k;
	cin >> n >> k;
	for (int i = 0; i < n; i++) {
		for (int j = 0; j <= i; j++) {
			cin >> arr[i][j];
		}
	}
	{
		int tmp[maxn];
		int l = 0, r = n - 1;
		while (l < r) {
			memcpy(tmp, arr[l], sizeof(tmp));
			memcpy(arr[l], arr[r], sizeof(arr[l]));
			memcpy(arr[r], tmp, sizeof(arr[r]));
			l++;
			r--;
		}
	}
	memcpy(sq[0], arr, sizeof(arr));
	memcpy(tri[0], arr, sizeof(arr));
	for (int i = 1; i < logn; i++) {
		int cx = 1 << (i - 1), cy = 1 << i;
		for (int x = 0; x + cy <= n; x++) {
			for (int y = 0; y + cy <= n; y++) {
				dbg(i, x, y);
				sq[i][x][y] = max({sq[i - 1][x][y], sq[i - 1][x + cx][y], sq[i - 1][x][y + cx],
								   sq[i - 1][x + cy][y + cx]});
				tri[i][x][y] = max({
					tri[i - 1][x + cx][y],
					tri[i - 1][x][y + cx],
					sq[i - 1][x][y],
				});
			}
		}
	}
	long ans = 0;
	int trix = __lg(k), sqx = __lg((k + 1) / 2);
	dbg(trix, sqx);
	int triadd = k - (1 << trix);
	dbg(tri[1][0][0]);
	for (int i = 0; i + k <= n; i++) {
		for (int j = 0; j + k <= n - i; j++) {
			ans += max({sq[sqx][i][j], tri[trix][i + triadd][j], tri[trix][i][j + triadd]});
			dbg(i, j);
		}
	}
	cout << ans << endl;
}

int main() {
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