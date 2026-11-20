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

const int maxn = 3000;

bool power_of2(int n) {
	if (n & (n + 1)) {
		return false;
	}
	for (int i = 1; i <= n; i++) {
		for (int j = i + 1; j <= n; j++) {
			int k = i ^ j;
			if (i < j && j < k) {
				cout << i << " " << j << " " << k << endl;
			}
		}
	}
	return true;
}

bool power_of3(int n) {
	vector<int> pows;
	int cur = 1;
	while(cur <= maxn) {
		pows.push_back(cur);
		cur *= 3;
	}
	if (find(begin(pows), end(pows), n) == pows.end()) {
		return false;
	}
	auto ind = find(begin(pows), end(pows), n) - pows.begin();
	auto print = [&](int i, int j, int k) -> void {
		cout << i + 1 << " " << j + 1 << " " << k + 1 << endl;
	};
	for (int i = 1; i <= ind; i++) {
		for (int base = 0; base < n; base += pows[i]) {
			for (int a = 0; a < pows[i - 1]; a++) {
				for (int b = 0; b < pows[i - 1]; b++) {
					int c = (a + b) % pows[i - 1];
					print(base + a, base + pows[i - 1] + b, base + 2 * pows[i - 1] + c);
				}
			}
		}
	}
	return true;
}

void solve() {
	int n;
	cin >> n;
	power_of2(n) || power_of3(n);
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