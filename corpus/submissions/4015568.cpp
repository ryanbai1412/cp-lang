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

const int maxn = 2e5;

int n, k;
bool arr[maxn];
int starting[maxn], ideg[maxn], odeg[maxn];
vector<int> graph[maxn];

namespace S1 {
bool check() {
	return none_of(begin(arr), end(arr), [](bool b) -> bool { return b; });
}

int siz[maxn];

int dfs(int u) {
	if (siz[u]) {
		return siz[u];
	}
	siz[u] = 1;
	for (auto& v : graph[u]) {
		siz[u] ^= dfs(v);
	}
	return siz[u];
}

void solve() {
	int ans = 0;
	for (int i = 0; i < k; i++) {
		ans ^= dfs(starting[i]);
	}
	if (ans & 1) {
		cout << "Alice" << endl;
	} else {
		cout << "Bob" << endl;
	}
}
}  // namespace S1

namespace S2 {
bool check() {
	return k == 1;
}

int memo[maxn];

// 0 - first mover's turn now
// 1 - second mover's turn now
// 2 - first chooses
// 3 - second chooses
int dp(int u) {
	if (memo[u] != -1) {
		return memo[u];
	}
	int& ans = memo[u];
	ans = 1;
	if (arr[u]) {
		for (auto& v : graph[u]) {
			int cur = dp(v);
			if (cur == 3 || cur == 1) {
				ans = 2;
				return ans;
			}
		}
		ans = 1;
		return ans;
	} else {
		for (auto& v : graph[u]) {
			int cur = dp(v);
			ans ^= cur;
			if (cur >= 2) {
				return ans;
			}
		}
	}
	return ans;
}

void solve() {
	memset(memo, -1, sizeof(memo));
	int ans = dp(starting[0]);
	if (ans == 0 || ans == 3) {
		cout << "Bob" << endl;
	} else {
		cout << "Alice" << endl;
	}
}
}  // namespace S2

namespace S3 {
bool check() {
	return n <= 10 && accumulate(begin(ideg), end(ideg), 0) <= 10 && k <= 10;
}

const int maxn = 10;

int memo[1 << maxn], chash;
vector<pair<int, int>> stack;

int mex(vector<int> arr) {
	sort(begin(arr), end(arr));
	auto cend = unique(begin(arr), end(arr)) - arr.begin();
	for (int i = 0; i < cend; i++) {
		if (arr[i] != i) {
			return i;
		}
	}
	return cend;
}

void push(int u, int i) {
	chash |= 1 << u;
	stack.emplace_back(u, i);
}

void pop() {
	auto [u, _] = stack.back();
	chash ^= 1 << u;
	stack.pop_back();
}

void rehash() {
	chash = 0;
	for (auto& [u, _] : stack) {
		chash |= 1 << u;
	}
}

int dfs() {
	int key = chash;
	int& ans = memo[key];
	if (ans != -1) {
		return ans;
	}
	vector<int> states;
	auto add = [&](int u, int ind) -> void {
		push(u, ind);
		states.push_back(dfs());
		pop();
	};
	auto add_ret = [&]() -> void {
		auto tmp = stack;
		while (sz(stack) > 1) {
			auto [v, ind] = stack.back();
			auto [u, _] = stack[sz(stack) - 2];
			if (ind + 1 < sz(graph[u])) {
				stack.back() = {graph[u][ind + 1], ind + 1};
				rehash();
				states.push_back(dfs());
				break;
			} else {
				stack.pop_back();
			}
		}
		if (sz(stack) == 1) {
			states.push_back(0);
		}
		stack = tmp;
		rehash();
	};
	auto [u, _] = stack.back();
	if (arr[u]) {
		for (int i = 0; i < sz(graph[u]); i++) {
			add(graph[u][i], i);
		}
		add_ret();
	} else {
		if (sz(graph[u])) {
			add(graph[u][0], 0);
		} else {
			add_ret();
		}
	}
	ans = mex(states);
	return ans;
}

void solve() {
	int ans = 0;
	for (int i = 0; i < k; i++) {
		memset(memo, -1, sizeof(memo));
		push(starting[i], 0);
		ans ^= dfs();
		pop();
	}
	if (ans) {
		cout << "Alice" << endl;
	} else {
		cout << "Bob" << endl;
	}
}
}  // namespace S3

void solve() {
	cin >> n;
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	for (int i = 0; i < n; i++) {
		int m;
		cin >> m;
		ideg[i] = m;
		for (int j = 0; j < m; j++) {
			int v;
			cin >> v;
			v--;
			odeg[v]++;
			graph[i].push_back(v);
		}
	}
	cin >> k;
	for (int i = 0; i < k; i++) {
		cin >> starting[i];
		starting[i]--;
	}
	S3::solve();
//	if (S1::check()) {
//		S1::solve();
//	} else if (S2::check()) {
//		S2::solve();
//	} else if (S3::check()) {
//		S3::solve();
//	}
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