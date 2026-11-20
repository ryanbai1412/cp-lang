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

void solve() {
	int n, m, x, y;
	cin >> n >> m >> x >> y;
	if (x > y) {
		swap(x, y);
	}
	int arr[n][m];
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	int ans = 2e9;
	auto upd = [&](int cx) -> void {
		ans = min(ans, abs(cx - x) + abs(cx - y));
	};
	for (int l = 0; l < n; l++) {
		int cur[m] {};
		for (int r = l; r < n; r++) {
			for (int k = 0; k < m; k++) {
				cur[k] += arr[r][k];
			}
			int j = 0, csum = 0;
			for (int i = 0; i < m; i++) {
				while (j < i) {
					csum += cur[j++];
				}
				while (j < m && csum <= y) {
					csum += cur[j++];
					upd(csum);
				}
				csum -= cur[i];
			}
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}