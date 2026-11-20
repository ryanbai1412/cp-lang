#pragma GCC optimize("Ofast")

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
#define sz(x) int(std::size(x))

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

constexpr int maxn = 2e6 + 5;

vector<pair<int, int>> graph[maxn];

struct Farthest {
    pair<int, int> opt {-1, -1};

    template <typename Cb>
    void dfs(int u, int p, int d, const Cb& cb) {
        opt = max(opt, {d, u});
        for (auto& [v, w] : graph[u]) {
            if (v == p || cb(v)) {
                continue;
            }
            dfs(v, u, d + w, cb);
        }
    }

    static pair<int, int> solve(int u) {
        return solve(u, [&](int) { return false; });
    }

    template <typename Cb>
    static pair<int, int> solve(int u, const Cb& cb) {
        Farthest solver;
        solver.dfs(u, -1, 0, cb);
        return solver.opt;
    }
};

struct FindPath {
    int dest;
    vector<pair<int, int>> ans, st;

    FindPath(int dest) : dest(dest) {}

    void dfs(int u, int p) {
        if (u == dest) {
            ans = st;
            ans.emplace_back(u, -1);
            return;
        }
        for (auto& [v, w] : graph[u]) {
            if (v == p) {
                continue;
            }
            st.emplace_back(u, w);
            dfs(v, u);
            st.pop_back();
        }
    }

    static vector<pair<int, int>> solve(int u, int v) {
        FindPath solver(v);
        solver.dfs(u, -1);
        return solver.ans;
    }
};

struct DS {
    int l = 0, r = 0;
    deque<pair<int, int>> q;

    void push_back(int x) {
        while (sz(q) && q.back().second <= x) {
            q.pop_back();
        }
        q.emplace_back(r++, x);
    }

    void pop_front() {
        if (l++ == q.front().first) {
            q.pop_front();
        }
    }

    int query_max() const {
        return q.front().second;
    }
};

void solve() {
    int n, kv;
    IO::read_int(n, kv);
    for (int i = 0; i < n - 1; i++) {
        int u, v, w;
        IO::read_int(u, v, w);
        u--;
        v--;
        graph[u].emplace_back(v, w);
        graph[v].emplace_back(u, w);
    }
    int du = Farthest::solve(0).second, dv = Farthest::solve(du).second;
    auto path = FindPath::solve(du, dv);
    bool is_diam[n] {};
    for (auto& [u, _w] : path) {
        is_diam[u] = true;
    }
    auto farthest_skip = [&](int u) -> bool { return is_diam[u]; };
    int ans = INT_MAX, psum[sz(path)];
    psum[0] = 0;
    for (int i = 0; i + 1 < sz(path); i++) {
        psum[i + 1] = psum[i] + path[i].second;
    }
    DS ds;
    for (int i = 0, j = 0, clen = 0; i < sz(path); i++) {
        while (j < sz(path) && clen <= kv) {
            auto& [u, ulen] = path[j++];
            ds.push_back(Farthest::solve(u, farthest_skip).first);
            clen += ulen;
        }
        ans = min(ans, max({ds.query_max(), psum[i],
                            psum[sz(path) - 1] - psum[j - 1]}));
        ds.pop_front();
        clen -= path[i].second;
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    IO::init();
    solve();
}