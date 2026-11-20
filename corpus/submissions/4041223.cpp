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

const int m = 30;
const double angle = M_PI / m, ca = cos(angle), sa = sin(angle);

void solve() {
	int n;
	cin >> n;
	tuple<double, double, int> arr[n];
	int ind = 0;
	for (auto& [x, y, cind] : arr) {
		cin >> x >> y;
		cind = ind++;
	}
	double ans[n] {};
	for (int i = 0; i < m; i++) {
		for (auto& [x, y, _] : arr) {
			double rx = x * ca + y * sa;
			double ry = - x * sa + y * ca;
			tie(x, y) = tie(rx, ry);
		}
		sort(arr, arr + n);
		double sl = 0, sr = 0;
		for (auto& [x, _, __] : arr) {
			sr += x;
		}
		for (int i = 0; i < n; i++) {
			auto &[x, _, ind] = arr[i];
			ans[ind] += i * x - sl + sr - (n - i) * x;
			sl += x;
			sr -= x;
		}
	}
	for (int i = 0; i < n; i++) {
		ans[i] /= m;
		ans[i] *= M_PI / 2;
		cout << ans[i] << endl;
	}
}

int main() {
	cout << fixed << setprecision(2);
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