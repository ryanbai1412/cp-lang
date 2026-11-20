
struct FastCin {
    char buf[20 << 20], *i = buf;
    FastCin() {
#ifndef LOCAL
        // std::ignore = freopen("string.in", "r", stdin);
#endif
        std::ignore = fread_unlocked(buf, 1, sizeof(buf), stdin);
    }
    FastCin &operator>>(int &x) {
        while (*i < '0') {
            i++;
        }
        x = 0;
        while (*i >= '0') {
            x = x * 10 + (*(i++) ^ '0');
        }
        return *this;
    }
    FastCin &operator>>(string &s) {
        while (*i < '0') {
            i++;
        }
        s.clear();
        while (*i >= '0') {
            s += *(i++);
        }
        return *this;
    }
} in;
// https://stackoverflow.com/questions/50081465/counting-1-bits-population-count-on-large-data-using-avx-512-or-avx-2
std::uint64_t popcnt_AVX2_lookup(const uint8_t *data, const size_t n) {
    size_t i = 0;
    const __m256i lookup = _mm256_setr_epi8(
                               /* 0 */ 0, /* 1 */ 1, /* 2 */ 1, /* 3 */ 2,
                               /* 4 */ 1, /* 5 */ 2, /* 6 */ 2, /* 7 */ 3,
                               /* 8 */ 1, /* 9 */ 2, /* a */ 2, /* b */ 3,
                               /* c */ 2, /* d */ 3, /* e */ 3, /* f */ 4,
                               /* 0 */ 0, /* 1 */ 1, /* 2 */ 1, /* 3 */ 2,
                               /* 4 */ 1, /* 5 */ 2, /* 6 */ 2, /* 7 */ 3,
                               /* 8 */ 1, /* 9 */ 2, /* a */ 2, /* b */ 3,
                               /* c */ 2, /* d */ 3, /* e */ 3, /* f */ 4);
    const __m256i low_mask = _mm256_set1_epi8(0x0f);
    __m256i acc = _mm256_setzero_si256();
#define ITER                                                                \
    {                                                                       \
        const __m256i vec =                                                 \
            _mm256_loadu_si256(reinterpret_cast<const __m256i*>(data + i)); \
        const __m256i lo = _mm256_and_si256(vec, low_mask);                 \
        const __m256i hi =                                                  \
            _mm256_and_si256(_mm256_srli_epi16(vec, 4), low_mask);          \
        const __m256i popcnt1 = _mm256_shuffle_epi8(lookup, lo);            \
        const __m256i popcnt2 = _mm256_shuffle_epi8(lookup, hi);            \
        local = _mm256_add_epi8(local, popcnt1);                            \
        local = _mm256_add_epi8(local, popcnt2);                            \
        i += 32;                                                            \
    }
    while (i + 8 * 32 <= n) {
        __m256i local = _mm256_setzero_si256();
        ITER ITER ITER ITER ITER ITER ITER ITER acc = _mm256_add_epi64(
                    acc, _mm256_sad_epu8(local, _mm256_setzero_si256()));
    }
    __m256i local = _mm256_setzero_si256();
    while (i + 32 <= n) {
        ITER;
    }
    acc = _mm256_add_epi64(acc, _mm256_sad_epu8(local, _mm256_setzero_si256()));
#undef ITER
    uint64_t result = 0;
    result += static_cast<uint64_t>(_mm256_extract_epi64(acc, 0));
    result += static_cast<uint64_t>(_mm256_extract_epi64(acc, 1));
    result += static_cast<uint64_t>(_mm256_extract_epi64(acc, 2));
    result += static_cast<uint64_t>(_mm256_extract_epi64(acc, 3));
    return result;
}
u64 popcnt_slow(const uint8_t *data, const size_t n) {
    u64 ans = 0;
    for (int i = 0; i < n; i++) {
        ans += __builtin_popcount(data[i]);
    }
    return ans;
}
template <typename B>
int count_pref(const B &b, int iql, int ind) {
    int ans = 0;
    auto w = b._M_getdata();
    // for (int i = iql / 64; i < ind / 64; i++) {
    //     ans += __builtin_popcountll(w[i]);
    // }
    // if (iql / 64 < ind / 64) {
    //     ans += popcnt_AVX2_lookup((uint8_t*)(&w[iql / 64]), 8 * (ind / 64 -
    //     iql / 64));
    // }
    {
        int ql = iql / 64, qr = ind / 64;
        while (ql % 4 && ql < qr) {
            ans += __builtin_popcountll(w[ql++]);
        }
        while (qr % 4 && ql < qr) {
            ans += __builtin_popcountll(w[--qr]);
        }
        if (ql < qr) {
            ans += popcnt_AVX2_lookup((uint8_t *)(&w[ql]), 8 * (qr - ql));
        }
        // int ans0 = 0, ans1 = 0, ans2 = 0, ans3 = 0;
        // for (int j = ql; j < qr; j += 4) {
        //     ans0 += __builtin_popcountll(w[j]);
        //     ans1 += __builtin_popcountll(w[j + 1]);
        //     ans2 += __builtin_popcountll(w[j + 2]);
        //     ans3 += __builtin_popcountll(w[j + 3]);
        // }
        // ans += ans0 + ans1 + ans2 + ans3;
    }
    // int cl = iql & (~63);
    // for (cl = 0; cl + 64 <= ind; cl += 64) {
    //     ans += __builtin_popcountll(b._M_getword(cl));
    // }
    int cl = ind & (~63);
    if (cl < ind) {
        ans += __builtin_popcountll(b._M_getword(cl) &
                                    ((u64(1) << (ind - cl)) - 1));
    }
    return ans;
}
template <typename B>
void solve(int n, int q) {
    static_assert(sizeof(typename B::_WordT) == 8);
    int arr[n];
    {
        string s;
        in >> s;
        for (int i = 0; i < n; i++) {
            arr[i] = s[i] - 'a';
        }
    }
    int qans[q] {};
    vector<pair<int, int>> queries[n];
    for (int qi = 0; qi < q; qi++) {
        int ind, x;
        in >> ind >> x;
        ind--;
        queries[ind].emplace_back(x, qi);
    }
    alignas(256) B dp {}, b_gt[2][26] {}, b_eq[2][26] {};
    for (int i = n - 1; i >= 0; i--) {
        if (i + 1 < n) {
            b_eq[i & 1][arr[i + 1]][(i + 1) / 2] = true;
            for (int j = 0; j < arr[i + 1]; j++) {
                b_gt[i & 1][j][(i + 1) / 2] = true;
            }
        }
        if (i % 2) {
            dp <<= 1;
        }
        {
            auto w1 = b_gt[i & 1][arr[i]]._M_getdata(),
                 w2 = b_eq[i & 1][arr[i]]._M_getdata();
            auto w3 = dp._M_w;
            int ql = i / 2 / 64, qr = sizeof(dp._M_w) / sizeof(dp._M_w[0]);
            while (ql % 4 && ql < qr) {
                w3[ql] = w1[ql] | (w2[ql] & w3[ql]);
                ql++;
            }
            while (qr % 4 && ql < qr) {
                qr--;
                w3[qr] = w1[qr] | (w2[qr] & w3[qr]);
            }
            using v4 = u64 __attribute__((vector_size(32)));
            for (int j = ql; j < qr; j += 4) {
                *(v4 *)(&w3[j]) =
                    *((v4 *)&w1[j]) | (*((v4 *)&w2[j]) & *((v4 *)&w3[j]));
            }
        }
        for (auto& [r, qi] : queries[i]) {
            qans[qi] = count_pref(dp, i / 2, (i + 2 * r) / 2 + (i & 1));
        }
    }
    for (auto &a : qans) {
        cout << a << endl;
    }
}
template <int N1>
bool solve1(int n, int q) {
    if (n <= N1) {
        solve < bitset < (N1 / 2 + 256) & (~255) >> (n, q);
        return true;
    }
    return false;
}
void solve() {
    int n, q;
    in >> n >> q;
    solve1<4000>(n, q) || solve1<23333>(n, q) || solve1<50000>(n, q) ||
    solve1<75000>(n, q) || solve1<90000>(n, q) || solve1<95000>(n, q) ||
    solve1<100000>(n, q);
}
