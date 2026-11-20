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

struct Vec {
	int* ptr {};
	int size {}, allocated {};

	~Vec() {
		delete[] ptr;
	}

	void push_back(int x) {
		if (!ptr) {
			ptr = new int[2];
			allocated = 2;
		} else if (size == allocated) {
			int* nptr = new int[allocated * 2];
			copy(ptr, ptr + allocated, nptr);
			allocated *= 2;
			delete[] ptr;
			ptr = nptr;
		}
		ptr[size++] = x;
	}

	template <typename T>
	void for_each(const T &t) {
		if (!ptr) {
			return;
		}
		for (int i = 0; i < size; i++) {
			t(ptr[i]);
		}
	}
};

const int maxn = 16e6 + 5;

Vec graph[maxn];
bitset<maxn> vis;

void dfs(int u) {
	vis[u] = true;
	graph[u].for_each([&](int v) -> void {
		if (!vis[v]) {
			dfs(v);
		}
	});
}

bool can_escape(int n, int m, vector<vector<int>> arr) {
	for (int i = 1; i <= n; i++) {
		for (int j = 1; j <= m; j++) {
			graph[i * j].push_back(arr[i][j]);
		}
	}
	dfs(arr[1][1]);
	return vis[n * m];
}

void solve() {
	int n, m;
	cin >> n >> m;
	vector<vector<int>> arr(n + 1, vector<int>(m + 1));
	for (auto& a : arr) {
		for (auto& b : a) {
			cin >> b;
		}
	}
	if (can_escape(n, m, arr)) {
		cout << "yes" << endl;
	} else {
		cout << "no" << endl;
	}
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