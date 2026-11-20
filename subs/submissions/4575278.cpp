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

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

void solve() {
	int n;
	cin >> n;
	int arr[n][2];
	for (auto& [x, y] : arr) {
		cin >> x >> y;
	}
	bool used[n] {};
	vector<int> vals[n + 1];
	for (int i = 0; i < n; i++) {
		vals[arr[i][0]].push_back(i);
	}
	priority_queue<int> pqa;
	min_pq<pair<int, int>> pqb;
	auto pollb = [&]() -> pair<int, int> {
		while (used[pqb.top().second]) {
			pqb.pop();
		}
		auto ans = pqb.top();
		pqb.pop();
		return ans;
	};
	for (int i = 0; i < n; i++) {
		pqb.emplace(arr[i][1], i);
	}
	int cnt = 0;
	long ans = 0;
	while (cnt < n) {
		for (int i = cnt; i <= cnt; i++) {
			dbg(i, cnt, sz(vals[i]));
			for (auto& a : vals[i]) {
				dbg(a);
				if (!used[a]) {
					used[a] = true;
					cnt++;
				} else {
					pqa.push(arr[a][1]);
				};
			}
		}
		if (cnt >= n) {
			break;
		}
		auto [x, ind] = pollb();
		assert(!used[ind]);
		used[ind] = true;
		ans += x;
		if (sz(pqa)) {
			ans -= pqa.top();
			pqa.pop();
		}
		cnt++;
	}
	cout << ans << endl;
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}