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

const int maxm = 25000;

void solve() {
	int n;
	cin >> n;
	vector<int> arr(n);
	for (auto& a : arr) {
		cin >> a;
	}
	sort(begin(arr), end(arr));
	auto process = [&](vector<int> x) -> vector<int> {
		bool dp[maxm + 1] {};
		dp[0] = true;
		vector<int> nxt;
		for (auto& a : x) {
			if (!dp[a]) {
				nxt.push_back(a);
				for (int i = a; i <= maxm; i++) {
					dp[i] |= dp[i - a];
				}
			}
		}
		return nxt;
	};
	while(true) {
		auto next = process(arr);
		if (arr == next) {
			break;
		}
		arr = next;
	}
	cout << sz(arr) << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}