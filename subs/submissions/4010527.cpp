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

namespace IO {
const int BUFFER_SIZE = 1 << 16;

char input_buffer[BUFFER_SIZE];
size_t input_pos = 0, input_len = 0;

char output_buffer[BUFFER_SIZE];
int output_pos = 0;

char number_buffer[100];
uint8_t lookup[100];

void _update_input_buffer() {
	input_len = fread(input_buffer, sizeof(char), BUFFER_SIZE, stdin);
	input_pos = 0;

	if (input_len == 0)
		input_buffer[0] = EOF;
}

inline char next_char(bool advance = true) {
	if (input_pos >= input_len)
		_update_input_buffer();

	return input_buffer[advance ? input_pos++ : input_pos];
}

template <typename T>
inline void read_int(T& number) {
	bool negative = false;
	number = 0;

	while (!isdigit(next_char(false)))
		if (next_char() == '-')
			negative = true;

	do {
		number = 10 * number + (next_char() - '0');
	} while (isdigit(next_char(false)));

	if (negative)
		number = -number;
}

template <typename T, typename... Args>
inline void read_int(T& number, Args&... args) {
	read_int(number);
	read_int(args...);
}

void _flush_output() {
	fwrite(output_buffer, sizeof(char), output_pos, stdout);
	output_pos = 0;
}

inline void write_char(char c) {
	if (output_pos == BUFFER_SIZE)
		_flush_output();

	output_buffer[output_pos++] = c;
}

template <typename T>
inline void write_int(T number, char after = '\0') {
	if (number < 0) {
		write_char('-');
		number = -number;
	}

	int length = 0;

	while (number >= 10) {
		uint8_t lookup_value = lookup[number % 100];
		number /= 100;
		number_buffer[length++] = char((lookup_value & 15) + '0');
		number_buffer[length++] = char((lookup_value >> 4) + '0');
	}

	if (number != 0 || length == 0)
		write_char(char(number + '0'));

	for (int i = length - 1; i >= 0; i--)
		write_char(number_buffer[i]);

	if (after)
		write_char(after);
}

void init() {
	// Make sure _flush_output() is called at the end of the program.
	bool exit_success = atexit(_flush_output) == 0;
	assert(exit_success);

	for (int i = 0; i < 100; i++)
		lookup[i] = uint8_t((i / 10 << 4) + i % 10);
}
}  // namespace IO

struct Solver {
	int n;
	vector<int> arr;
	vector<vector<int>> graph;

	Solver(int n, const vector<int>& arr, const vector<pair<int, int>>& edges)
		: n(n), arr(arr), graph(n) {
		for (auto& [u, v] : edges) {
			graph[u].push_back(v);
			graph[v].push_back(u);
		}
		pdfs(0);
	}

	void pdfs(int u) {
		for (auto& v : graph[u]) {
			graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
			pdfs(v);
		}
	}

	vector<int> query(int u, int d) {
		bool vis[n] {};
		vector<int> ans;
		vector<pair<int, int>> q;
		auto push = [&](int u, int cd) -> void {
			if (!vis[u] && cd <= d) {
				vis[u] = true;
				ans.push_back(arr[u]);
				q.emplace_back(u, cd);
			}
		};
		push(u, 0);
		while (sz(q)) {
			auto [u, d] = q.back();
			q.pop_back();
			for (auto& v : graph[u]) {
				push(v, d + 1);
			}
		}
		return ans;
	}
};

#ifdef LOCAL
const int hsize = maxn;
#else
const int hsize = 100 * maxn;
#endif

struct Node {
	int v, lc, rc;
} heap[hsize];

int alloc() {
	static int hind = 1;
	return hind++;
}

void maintain(int o) {
	heap[o].v = 0;
	if (heap[o].lc) {
		heap[o].v += heap[heap[o].lc].v;
	}
	if (heap[o].rc) {
		heap[o].v += heap[heap[o].rc].v;
	}
}

void update(int& o, int l, int r, int ind, int x) {
	int po = o;
	o = alloc();
	heap[o] = heap[po];
	if (l == r) {
		heap[o].v += x;
		return;
	}
	int mid = (l + r) / 2;
	if (ind <= mid) {
		update(heap[o].lc, l, mid, ind, x);
	} else {
		update(heap[o].rc, mid + 1, r, ind, x);
	}
	heap[o].v += x;
}

int query(int o, int l, int r, int ql, int qr) {
	if (!o) {
		return 0;
	} else if (ql <= l && r <= qr) {
		return heap[o].v;
	}
	int mid = (l + r) / 2, ans = 0;
	if (ql <= mid) {
		ans += query(heap[o].lc, l, mid, ql, qr);
	}
	if (mid < qr) {
		ans += query(heap[o].rc, mid + 1, r, ql, qr);
	}
	return ans;
}

void merge(int& a, int b, int l, int r) {
	if (!b) {
		return;
	}
	int pa = a;
	a = alloc();
	if (pa) {
		heap[a] = heap[pa];
	}
	if (!a) {
		heap[a] = heap[b];
		return;
	} else if (l == r) {
		heap[a].v += heap[b].v;
		return;
	}
	int mid = (l + r) / 2;
	merge(heap[a].lc, heap[b].lc, l, mid);
	merge(heap[a].rc, heap[b].rc, mid + 1, r);
	maintain(a);
}

int cn, arr[maxn], depth[maxn], siz[maxn], seg[maxn];
vector<int> graph[maxn], compute[maxn], computed[maxn];

void pdfs(int u, int d = 0) {
	depth[u] = d;
	siz[u] = 1;
	for (auto& v : graph[u]) {
		graph[v].erase(find(begin(graph[v]), end(graph[v]), u));
		pdfs(v, d + 1);
		siz[u] += siz[v];
		if (siz[v] > siz[graph[u][0]]) {
			swap(graph[u][0], v);
		}
	}
}

map<int, int> dfs(int u) {
	int& cseg = seg[u];
	map<int, int> cur;
	cur[arr[u]] = depth[u];
	update(cseg, 0, cn - 1, depth[u], 1);
	for (auto& v : graph[u]) {
		auto x = dfs(v);
		if (v == graph[u][0]) {
			cseg = seg[v];
			update(cseg, 0, cn - 1, depth[u], 1);
			auto it = x.find(arr[u]);
			if (it != x.end()) {
				update(cseg, 0, cn - 1, it->second, -1);
			}
			swap(cur, x);
			cur[arr[u]] = depth[u];
		} else {
			merge(cseg, seg[v], 0, cn - 1);
			for (auto& [k, v] : x) {
				auto [it, inserted] = cur.try_emplace(k, v);
				if (!inserted) {
					if (v < it->second) {
						update(cseg, 0, cn - 1, it->second, -1);
						it->second = v;
					} else {
						update(cseg, 0, cn - 1, v, -1);
					}
				}
			}
		}
	}
	for (auto& a : compute[u]) {
		auto it = cur.find(a);
		if (it != cur.end()) {
			computed[u].push_back(it->second);
		} else {
			computed[u].push_back(1e9);
		}
	}
	return cur;
}

void solve() {
	int n, m, q;
	IO::read_int(n, m, q);
	cn = m;
	vector<pair<int, int>> e1(n - 1);
	for (auto& [u, v] : e1) {
		IO::read_int(u, v);
		u--;
		v--;
	}
	for (int i = 0; i < m - 1; i++) {
		int u, v;
		IO::read_int(u, v);
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	vector<int> a1(n);
	for (auto& a : a1) {
		IO::read_int(a);
	}
	for (int i = 0; i < m; i++) {
		IO::read_int(arr[i]);
	}
	vector<array<int, 5>> queries;
	while (q--) {
		int t;
		IO::read_int(t);
		array<int, 5> cur {};
		if (t == 1) {
			int u, du, v, dv;
			IO::read_int(u, v, du, dv);
			u--;
			v--;
			cur = {t, u, du, v, dv};
		} else {
			int u, x;
			IO::read_int(u, x);
			u--;
			cur = {t, u, x};
		}
		queries.push_back(cur);
	}
	{
		vector<int> cur = a1;
		for (auto& a : queries) {
			if (a[0] == 1) {
				auto& [t, u, du, v, dv] = a;
				for (auto& b : cur) {
					compute[v].push_back(b);
				}
			} else {
				auto& [t, u, x, _, __] = a;
				cur[u] = x;
			}
		}
		for (int i = 0; i < m; i++) {
			sort(begin(compute[i]), end(compute[i]));
			compute[i].erase(unique(begin(compute[i]), end(compute[i])),
							 compute[i].end());
		}
	}
	pdfs(0);
	dfs(0);
	Solver solver(n, a1, e1);
	int ans = 0;
	for (auto& a : queries) {
		if (a[0] == 1) {
			auto& [t, u, du, v, dv] = a;
			du ^= ans;
			dv ^= ans;
			vector<int> vals = solver.query(u, du);
			sort(begin(vals), end(vals));
			vals.erase(unique(begin(vals), end(vals)), vals.end());
			ans = sz(vals) + query(seg[v], 0, cn - 1, 0, depth[v] + dv);
			dbg(sz(vals), query(seg[v], 0, cn - 1, 0, depth[v] + dv));
			for (auto& a : vals) {
				auto ind = lower_bound(begin(compute[v]), end(compute[v]), a) -
						   compute[v].begin();
				if (computed[v][ind] <= depth[v] + dv) {
					ans--;
				}
			}
			IO::write_int(ans, '\n');
		} else {
			auto& [t, u, x, _, __] = a;
			solver.arr[u] = x;
		}
	}
}

int main() {
	IO::init();
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