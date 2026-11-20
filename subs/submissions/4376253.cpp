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

const int maxn = 2e5;

using A = array<array<long, 3>, 3>;

void add(A &x, const A &y) {
	for (int i = 0; i < 3; i++) {
		for (int j = 0; j < 3; j++) {
			x[i][j] += y[i][j];
		}
	}
}

void sub(A &x, const A &y) {
	for (int i = 0; i < 3; i++) {
		for (int j = 0; j < 3; j++) {
			x[i][j] -= y[i][j];
		}
	}
}

A vals[maxn], cnt;
bool arr[maxn];
vector<int> graph[maxn];

void pdfs(int u) {
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v);
	}
}

void dfs(int u, int a, int b) {
	if (arr[u]) {
		a++;
	} else {
		b++;
	}
	if (max(a, b) >= 3) {
		return;
	}
	cnt[a][b]++;
	for (auto& v : graph[u]) {
		dfs(v, a, b);
	}
}

void solve() {
	int n;
	cin >> n;
	string s;
	cin >> s;
	for (int i = 0; i < n; i++) {
		arr[i] = s[i] == 'B';
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	pdfs(0);
	for (int i = 0; i < n; i++) {
		cnt = {};
		dfs(i, 0, 0);
		vals[i] = cnt;
		dbg(i);
		for (int a = 0; a < 3; a++) {
			for (int b = 0; b < 3; b++) {
				if (cnt[a][b]) {
					dbg(a, b, cnt[a][b]);
				}
			}
		}
	}
	long ans = 0;
	for (int i = 0; i < n; i++) {
		A cur {};
		int ax = 0, bx = 0;
		if (arr[i]) {
			ax++;
		} else {
			bx++;
		}
		ans++;
		cur[ax][bx]++;
		for (auto& v : graph[i]) {
			for (int a = 0; a < 3; a++) {
				for (int b = 0; b < 3; b++) {
					for (int c = 0; a + c < 3; c++) {
						for (int d = 0; b + d < 3; d++) {
							ans += cur[a][b] * vals[v][c][d];
						}
					}
				}
			}
			for (int a = 0; a + ax < 3; a++) {
				for (int b = 0; b + bx < 3; b++) {
					cur[a + ax][b + bx] += vals[v][a][b];
				}
			}
		}
		dbg(i, ans);
	}
	long x = n;
	x = x + x * (x - 1) / 2;
	dbg(x, ans);
	cout << x - ans << endl;
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}