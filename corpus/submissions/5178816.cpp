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

struct IOPre {
    static constexpr int TEN = 10, SZ = TEN * TEN * TEN * TEN;
    std::array<char, 4 * SZ> num;
    constexpr IOPre() : num {} {
        for (int i = 0; i < SZ; i++) {
            int n = i;
            for (int j = 3; j >= 0; j--) {
                num[i * 4 + j] = static_cast<char>(n % TEN + '0');
                n /= TEN;
            }
        }
    }
};
struct IO {
#if !HAVE_DECL_FREAD_UNLOCKED
#define fread_unlocked fread
#endif
#if !HAVE_DECL_FWRITE_UNLOCKED
#define fwrite_unlocked fwrite
#endif
    static constexpr int SZ = 1 << 17, LEN = 32, TEN = 10, HUNDRED = TEN * TEN,
                         THOUSAND = HUNDRED * TEN, TENTHOUSAND = THOUSAND * TEN,
                         MAGIC_MULTIPLY = 205, MAGIC_SHIFT = 11, MASK = 15,
                         TWELVE = 12, SIXTEEN = 16;
    static constexpr IOPre io_pre = {};
    std::array<char, SZ> input_buffer, output_buffer;
    int input_ptr_left, input_ptr_right, output_ptr_right;

    IO()
        : input_buffer {},
          output_buffer {},
          input_ptr_left {},
          input_ptr_right {},
          output_ptr_right {} {}
    IO(const IO&) = delete;
    IO(IO&&) = delete;
    IO& operator=(const IO&) = delete;
    IO& operator=(IO&&) = delete;

    ~IO() {
        flush();
    }

    template <class T>
    struct is_char {
        static constexpr bool value = std::is_same_v<T, char>;
    };

    template <class T>
    struct is_bool {
        static constexpr bool value = std::is_same_v<T, bool>;
    };

    template <class T>
    struct is_string {
        static constexpr bool value =
            std::is_same_v<T, std::string> || std::is_same_v<T, const char*> ||
            std::is_same_v<T, char*> || std::is_same_v<std::decay_t<T>, char*>;
    };

    template <class T, class D = void>
    struct is_custom {
        static constexpr bool value = false;
    };

    template <class T>
    struct is_custom<T, std::void_t<typename T::internal_value_type>> {
        static constexpr bool value = true;
    };

    template <class T>
    struct is_default {
        static constexpr bool value = is_char<T>::value || is_bool<T>::value ||
                                      is_string<T>::value ||
                                      std::is_integral_v<T>;
    };

    template <class T, class D = void>
    struct is_iterable {
        static constexpr bool value = false;
    };

    template <class T>
    struct is_iterable<
        T,
        typename std::void_t<decltype(std::begin(std::declval<T>()))>> {
        static constexpr bool value = true;
    };

    template <class T, class D = void, class E = void>
    struct is_applyable {
        static constexpr bool value = false;
    };

    template <class T>
    struct is_applyable<T,
                        std::void_t<typename std::tuple_size<T>::type>,
                        std::void_t<decltype(std::get<0>(std::declval<T>()))>> {
        static constexpr bool value = true;
    };

    template <class T>
    static constexpr bool needs_newline =
        (is_iterable<T>::value || is_applyable<T>::value) &&
        (!is_default<T>::value);

    template <typename T, typename U>
    struct any_needs_newline {
        static constexpr bool value = false;
    };
    template <typename T>
    struct any_needs_newline<T, std::index_sequence<>> {
        static constexpr bool value = false;
    };
    template <typename T, std::size_t I, std::size_t... Is>
    struct any_needs_newline<T, std::index_sequence<I, Is...>> {
        static constexpr bool value =
            needs_newline<decltype(std::get<I>(std::declval<T>()))> ||
            any_needs_newline<T, std::index_sequence<Is...>>::value;
    };

    inline void load() {
        memmove(std::begin(input_buffer),
                std::begin(input_buffer) + input_ptr_left,
                input_ptr_right - input_ptr_left);
        input_ptr_right =
            input_ptr_right - input_ptr_left +
            static_cast<int>(fread_unlocked(
                std::begin(input_buffer) + input_ptr_right - input_ptr_left, 1,
                SZ - input_ptr_right + input_ptr_left, stdin));
        input_ptr_left = 0;
    }

    inline void read_char(char& c) {
        if (input_ptr_left + LEN > input_ptr_right)
            load();
        c = input_buffer[input_ptr_left++];
    }
    inline void read_string(std::string& x) {
        char c;
        while (read_char(c), c < '!')
            continue;
        x = c;
        while (read_char(c), c >= '!')
            x += c;
    }
    inline void read_string(char* x) {
        char c;
        while (read_char(c), c < '!')
            continue;
        *(x++) = c;
        while (read_char(c), c >= '!')
            *(x++) = c;
        *x = 0;
    }
    template <class T>
    inline std::enable_if_t<std::is_integral_v<T>, void> read_int(T& x) {
        if (input_ptr_left + LEN > input_ptr_right)
            load();
        char c = 0;
        do
            c = input_buffer[input_ptr_left++];
        while (c < '-');
        [[maybe_unused]] bool minus = false;
        if constexpr (std::is_signed<T>::value == true)
            if (c == '-')
                minus = true, c = input_buffer[input_ptr_left++];
        x = 0;
        while (c >= '0')
            x = x * TEN + (c & MASK), c = input_buffer[input_ptr_left++];
        if constexpr (std::is_signed<T>::value == true)
            if (minus)
                x = -x;
    }

    inline void skip_space() {
        if (input_ptr_left + LEN > input_ptr_right)
            load();
        while (input_buffer[input_ptr_left] <= ' ')
            input_ptr_left++;
    }

    inline void flush() {
        fwrite_unlocked(std::begin(output_buffer), 1, output_ptr_right, stdout);
        output_ptr_right = 0;
    }

    inline void write_char(char c) {
        if (output_ptr_right > SZ - LEN)
            flush();
        output_buffer[output_ptr_right++] = c;
    }

    inline void write_bool(bool b) {
        if (output_ptr_right > SZ - LEN)
            flush();
        output_buffer[output_ptr_right++] = b ? '1' : '0';
    }

    inline void write_string(const std::string& s) {
        for (auto x : s)
            write_char(x);
    }

    inline void write_string(const char* s) {
        while (*s)
            write_char(*s++);
    }

    inline void write_string(char* s) {
        while (*s)
            write_char(*s++);
    }

    template <typename T>
    inline std::enable_if_t<std::is_integral_v<T>, void> write_int(T x) {
        if (output_ptr_right > SZ - LEN)
            flush();
        if (!x) {
            output_buffer[output_ptr_right++] = '0';
            return;
        }
        if constexpr (std::is_signed<T>::value == true)
            if (x < 0)
                output_buffer[output_ptr_right++] = '-', x = -x;
        int i = TWELVE;
        std::array<char, SIXTEEN> buf {};
        while (x >= TENTHOUSAND) {
            memcpy(std::begin(buf) + i,
                   std::begin(io_pre.num) + (x % TENTHOUSAND) * 4, 4);
            x /= TENTHOUSAND;
            i -= 4;
        }
        if (x < HUNDRED) {
            if (x < TEN) {
                output_buffer[output_ptr_right++] = static_cast<char>('0' + x);
            } else {
                std::uint32_t q =
                    (static_cast<std::uint32_t>(x) * MAGIC_MULTIPLY) >>
                    MAGIC_SHIFT;
                std::uint32_t r = static_cast<std::uint32_t>(x) - q * TEN;
                output_buffer[output_ptr_right] = static_cast<char>('0' + q);
                output_buffer[output_ptr_right + 1] =
                    static_cast<char>('0' + r);
                output_ptr_right += 2;
            }
        } else {
            if (x < THOUSAND) {
                memcpy(std::begin(output_buffer) + output_ptr_right,
                       std::begin(io_pre.num) + (x << 2) + 1, 3),
                    output_ptr_right += 3;
            } else {
                memcpy(std::begin(output_buffer) + output_ptr_right,
                       std::begin(io_pre.num) + (x << 2), 4),
                    output_ptr_right += 4;
            }
        }
        memcpy(std::begin(output_buffer) + output_ptr_right,
               std::begin(buf) + i + 4, TWELVE - i);
        output_ptr_right += TWELVE - i;
    }
    template <typename T_>
    IO& operator<<(T_&& x) {
        using T = typename std::remove_cv<
            typename std::remove_reference<T_>::type>::type;
        static_assert(is_custom<T>::value or is_default<T>::value or
                      is_iterable<T>::value or is_applyable<T>::value);
        if constexpr (is_custom<T>::value) {
            write_int(x.get());
        } else if constexpr (is_default<T>::value) {
            if constexpr (is_bool<T>::value) {
                write_bool(x);
            } else if constexpr (is_string<T>::value) {
                write_string(x);
            } else if constexpr (is_char<T>::value) {
                write_char(x);
            } else if constexpr (std::is_integral_v<T>) {
                write_int(x);
            }
        } else if constexpr (is_iterable<T>::value) {
            // strings are immune
            using E = decltype(*std::begin(x));
            constexpr char sep = needs_newline<E> ? '\n' : ' ';
            int i = 0;
            for (const auto& y : x) {
                if (i++)
                    write_char(sep);
                operator<<(y);
            }
        } else if constexpr (is_applyable<T>::value) {
            // strings are immune
            constexpr char sep =
                (any_needs_newline<
                    T, std::make_index_sequence<std::tuple_size_v<T>>>::value)
                    ? '\n'
                    : ' ';
            int i = 0;
            std::apply(
                [this, &sep, &i](auto const&... y) {
                    (((i++ ? write_char(sep) : void()), this->operator<<(y)),
                     ...);
                },
                x);
        }
        return *this;
    }
    template <typename T>
    IO& operator>>(T& x) {
        static_assert(is_custom<T>::value or is_default<T>::value or
                      is_iterable<T>::value or is_applyable<T>::value);
        static_assert(!is_bool<T>::value);
        if constexpr (is_custom<T>::value) {
            typename T::internal_value_type y;
            read_int(y);
            x = y;
        } else if constexpr (is_default<T>::value) {
            if constexpr (is_string<T>::value) {
                read_string(x);
            } else if constexpr (is_char<T>::value) {
                read_char(x);
            } else if constexpr (std::is_integral_v<T>) {
                read_int(x);
            }
        } else if constexpr (is_iterable<T>::value) {
            for (auto& y : x)
                operator>>(y);
        } else if constexpr (is_applyable<T>::value) {
            std::apply([this](auto&... y) { ((this->operator>>(y)), ...); }, x);
        }
        return *this;
    }

    IO* tie(std::nullptr_t) {
        return this;
    }
    void sync_with_stdio(bool) {}
    void exceptions(ios::iostate) {}
};
IO io;
#define cin io
#define cout io

// mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
mt19937 cowng(1412);

template <typename T, size_t N>
struct A {
    T heap[N];
    int free_inds[N - 1], free_inds_ptr = N - 1;

    A() {
        iota(rbegin(free_inds), rend(free_inds), 1);
    }

    int alloc() {
        int ans = free_inds[--free_inds_ptr];
        return ans;
    }

    int alloc(const T& t) {
        int ptr = alloc();
        heap[ptr] = t;
        return ptr;
    }

    void free(int ind) {
        free_inds[free_inds_ptr++] = ind;
    }

    T& operator[](int i) {
        return heap[i];
    }
};

struct DS {
    struct NV {
        int sum, pref, suff, v;

        friend NV operator+(const NV& l, const NV& r) {
            return {l.sum + r.sum, max(l.pref, l.sum + r.pref),
                    max(r.suff, l.suff + r.sum),
                    max({l.v, r.v, l.suff + r.pref})};
        }

        static NV from(int x) {
            return {x, x, x, x};
        }

        static NV from(int len, int x) {
            assert(len);
            return {x * len, max(x, x * len), max(x, x * len), max(x, x * len)};
        }

        static NV c_default() {
            return {0, -int(1e9), -int(1e9), -int(1e9)};
        }
    };

    struct Lazy {
        static constexpr int NO_SET = INT_MIN;

        int set;
        bool rev;

        Lazy& operator+=(const Lazy& l) {
            if (l.set != NO_SET) {
                return *this = l;
            }
            rev ^= l.rev;
            return *this;
        }

        void apply(NV& n, int siz) const {
            if (set != NO_SET) {
                n = NV::from(siz, set);
                return;
            }
            if (rev) {
                swap(n.suff, n.pref);
            }
        }

        bool is_noop() const {
            return set == NO_SET && !rev;
        }

        static Lazy c_def() {
            return {NO_SET, false};
        }

        static Lazy c_set(int x) {
            return {x, false};
        }

        static Lazy c_rev() {
            return {NO_SET, true};
        }
    };

    struct Node {
        NV v;
        Lazy lazy;
        int ox, siz, lc, rc, pri;

        static Node from(int x) {
            return {NV::from(x), Lazy::c_def(), x, 1, 0, 0, int(cowng())};
        }
    };

    A<Node, 1 << 19> heap;

    int root = 0;

    DS() {
        heap[0].v = NV::c_default();
    }

    template <bool LAZY_ONLY = false>
    void maintain(int o) {
        if (!o) {
            return;
        }
        auto& [v, lazy, ox, siz, lc, rc, _pri] = heap[o];
        if constexpr (!LAZY_ONLY) {
            siz = heap[lc].siz + heap[rc].siz + 1;
            v = heap[rc].v + NV::from(ox) + heap[lc].v;
        }
        lazy.apply(v, siz);
    }

    void pushdown(int o) {
        assert(o);
        auto& lazy = heap[o].lazy;
        auto &lc = heap[o].lc, &rc = heap[o].rc;
        if (lazy.rev) {
            swap(lc, rc);
        }
        auto go = [&](int c) -> void {
            if (!c) {
                return;
            }
            heap[c].lazy += lazy;
            lazy.apply(heap[c].v, heap[c].siz);
        };
        if (!lazy.is_noop()) {
            go(lc);
            go(rc);
        }
        if (lazy.set != Lazy::NO_SET) {
            heap[o].ox = lazy.set;
        }
        lazy = Lazy::c_def();
    }

    // kv <= sz(o)
    // [0, kv) goes to l, [kv, inf) goes to r
    void split(int o, int kv, int& l, int& r) {
        if (!o) {
            assert(!kv);
            l = r = 0;
            return;
        }
        pushdown(o);
        auto &lc = heap[o].lc, &rc = heap[o].rc;
        int lsiz = heap[lc].siz;
        if (kv <= lsiz) {
            r = o;
            split(lc, kv, l, lc);
        } else {
            l = o;
            split(rc, kv - lsiz - 1, rc, r);
        }
        maintain(o);
    }

    pair<int, int> split(int o, int kv) {
        int l, r;
        split(o, kv, l, r);
        return {l, r};
    }

    tuple<int, int, int> split(int o, int ind, int len) {
        auto [l, r1] = split(o, ind);
        auto [mid, r] = split(r1, len);
        return {l, mid, r};
    }

    int merge(int l, int r) {
        if (!l || !r) {
            maintain(l);
            maintain(r);
            return l + r;
        }
        if (heap[l].pri < heap[r].pri) {
            pushdown(l);
            heap[l].rc = merge(heap[l].rc, r);
            maintain(l);
            return l;
        } else {
            pushdown(r);
            heap[r].lc = merge(l, heap[r].lc);
            maintain(r);
            return r;
        }
    }

    int merge(int a, int b, int c) {
        return merge(a, merge(b, c));
    }

    void free(int o) {
        if (!o) {
            return;
        }
        free(heap[o].lc);
        free(heap[o].rc);
        heap.free(o);
    }

    void update_insert(int ind, const vector<int>& arr) {
        auto [l, r] = split(root, ind);
        for (auto& a : arr) {
            l = merge(l, heap.alloc(Node::from(a)));
        }
        root = merge(l, r);
    }

    void update_delete(int ind, int len) {
        auto [l, mid, r] = split(root, ind, len);
        free(mid);
        root = merge(l, r);
    }

    void update_set(int ind, int len, int x) {
        auto [l, mid, r] = split(root, ind, len);
        heap[mid].lazy += Lazy::c_set(x);
        root = merge(l, mid, r);
    }

    void update_reverse(int ind, int len) {
        auto [l, mid, r] = split(root, ind, len);
        heap[mid].lazy += Lazy::c_rev();
        root = merge(l, mid, r);
    }

    int query_sum(int ind, int len) {
        auto [l, mid, r] = split(root, ind, len);
        int ans = heap[mid].v.sum;
        root = merge(l, mid, r);
        return ans;
    }

    int query_max_subarray_sum() {
        return heap[root].v.v;
    }
};

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    DS ds;
    ds.update_insert(0, arr);
    dbg("START");
    while (q--) {
        char s[10] {};
        cin >> s;
        if (s[2] == 'X') {
            cout << ds.query_max_subarray_sum() << endl;
            continue;
        }
        int ind, len;
        cin >> ind >> len;
        if (s[0] == 'I') {
            vector<int> carr(len);
            for (auto& a : carr) {
                cin >> a;
            }
            ds.update_insert(ind, carr);
            continue;
        }
        ind--;
        if (s[0] == 'D') {
            ds.update_delete(ind, len);
        } else if (s[0] == 'R') {
            ds.update_reverse(ind, len);
        } else if (s[0] == 'G') {
            if (!len) {
                cout << 0 << endl;
            } else {
                cout << ds.query_sum(ind, len) << endl;
            }
        } else if (s[8] == 'E') {
            int x;
            cin >> x;
            ds.update_set(ind, len, x);
        } else {
            assert(false);
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}