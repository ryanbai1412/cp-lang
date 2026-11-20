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

const int maxn = 255;

int n, arr[maxn];
bool vis[maxn][maxn];
vector<int> memo[maxn][maxn];

void dp(int i, int csm) {
	if (vis[i][csm]) {
		return;
	}
	vis[i][csm] = true;
	if (i == n) {
		if (!csm) {
			memo[i][csm].push_back(0);
		}
		return;
	}
	auto &ans = memo[i][csm];
	dp(i + 1, csm);
	ans.insert(ans.end(), begin(memo[i + 1][csm]), end(memo[i + 1][csm]));
	dp(i + 1, csm + 1);
	for (auto& a : memo[i + 1][csm + 1]) {
		ans.push_back(arr[i] * arr[i] + a);
	}
	if (csm) {
		dp(i + 1, csm - 1);
		for (auto& a : memo[i + 1][csm - 1]) {
			ans.push_back(a - arr[i] * arr[i]);
		}
	}
	sort(begin(ans), end(ans));
	ans.erase(unique(begin(ans), end(ans)), ans.end());
}

void solve() {
	cin >> n;
	string s;
	cin >> s;
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] - 'A' + 1;
	}
	dp(0, 0);
	auto &ans = memo[0][0];
	cout << sz(ans) << endl;
	for (auto& a : ans) {
		cout << a << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}