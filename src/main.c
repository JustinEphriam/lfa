// C Keyword Analyzer (C Ultra - Unified Work-Stealing)
#define _GNU_SOURCE
#include <assert.h>
#include <errno.h>
#include <immintrin.h>
#include <inttypes.h>
#include <pthread.h>
#include <sched.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

// Raw Linux x86_64 syscalls
#define SYS_READ_NR       0L
#define SYS_WRITE_NR      1L
#define SYS_CLOSE_NR      3L
#define SYS_DUP_NR        32L
#define SYS_GETDENTS64_NR 217L
#define SYS_OPENAT_NR     257L
#define SYS_STATX_NR      332L

#define AT_FDCWD_VAL          (-100L)
#define AT_SYMLINK_NOFOLLOW_V 0x100U
#define O_RDONLY_VAL          0L
#define O_NONBLOCK_VAL        04000L
#define O_DIRECTORY_VAL       0200000L
#define O_CLOEXEC_VAL         02000000L
#define STATX_TYPE_VAL        0x0001U
#define S_IFMT_VAL            0170000U
#define S_IFDIR_VAL           0040000U
#define S_IFREG_VAL           0100000U
#define DT_UNKNOWN_VAL        0
#define DT_DIR_VAL            4
#define DT_REG_VAL            8
#define EINTR_VAL             (-4L)

struct c_statx {
    uint32_t stx_mask;
    uint32_t stx_blksize;
    uint64_t stx_attributes;
    uint32_t stx_nlink;
    uint32_t stx_uid;
    uint32_t stx_gid;
    uint16_t stx_mode;
    uint8_t  pad[226];
};

static inline long sys_openat(long dirfd, const void *path, long flags) {
    assert(path != NULL);
    long ret;
    register long r10 __asm__("r10") = 0L;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "0"(SYS_OPENAT_NR), "D"(dirfd), "S"(path), "d"(flags), "r"(r10)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static inline long sys_read(long fd, void *buf, size_t count) {
    assert(fd >= 0);
    assert(buf != NULL);
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "0"(SYS_READ_NR), "D"(fd), "S"(buf), "d"(count)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static inline long sys_write(long fd, const void *buf, size_t count) {
    assert(fd >= 0);
    assert(buf != NULL);
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "0"(SYS_WRITE_NR), "D"(fd), "S"(buf), "d"(count)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static inline long sys_close(long fd) {
    assert(fd >= 0);
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "0"(SYS_CLOSE_NR), "D"(fd)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static inline long sys_dup(long oldfd) {
    assert(oldfd >= 0);
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "0"(SYS_DUP_NR), "D"(oldfd)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static inline long sys_getdents64(long fd, void *dirp, size_t count) {
    assert(fd >= 0);
    assert(dirp != NULL);
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "0"(SYS_GETDENTS64_NR), "D"(fd), "S"(dirp), "d"(count)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static inline long sys_statx(long dirfd, const void *path, unsigned int flags, unsigned int mask, struct c_statx *buf) {
    assert(path != NULL);
    assert(buf != NULL);
    long ret;
    register long r10 __asm__("r10") = (long)mask;
    register void *r8 __asm__("r8") = (void *)buf;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "0"(SYS_STATX_NR), "D"(dirfd), "S"(path), "d"((long)flags), "r"(r10), "r"(r8)
        : "rcx", "r11", "memory"
    );
    return ret;
}

// Supported C keywords across C89, C99, C11, and C23 (all 59 C keywords).
#define KEYWORD_COUNT 59
#define KW_SLOT_MASK  255U
#define KW_MAX_LEN    14U
#define KW_EMPTY      255U

static const char *const KEYWORD_NAMES[KEYWORD_COUNT] = {
    // C89/C90 (32)
    "auto", "break", "case", "char", "const", "continue",
    "default", "do", "double", "else", "enum", "extern",
    "float", "for", "goto", "if", "int", "long",
    "register", "return", "short", "signed", "sizeof", "static",
    "struct", "switch", "typedef", "union", "unsigned", "void",
    "volatile", "while",
    // C99 (5)
    "inline", "restrict", "_Bool", "_Complex", "_Imaginary",
    // C11 (7)
    "_Alignas", "_Alignof", "_Atomic", "_Static_assert", "_Noreturn",
    "_Thread_local", "_Generic",
    // C23 (15)
    "_BitInt", "typeof", "typeof_unqual", "_Decimal128", "_Decimal32",
    "_Decimal64", "alignas", "alignof", "bool", "constexpr",
    "false", "nullptr", "static_assert", "thread_local", "true"
};

static const uint8_t KEYWORD_LENS[KEYWORD_COUNT] = {
    4, 5, 4, 4, 5, 8,
    7, 2, 6, 4, 4, 6,
    5, 3, 4, 2, 3, 4,
    8, 6, 5, 6, 6, 6,
    6, 6, 7, 5, 8, 4,
    8, 5,
    6, 8, 5, 8, 10,
    8, 8, 7, 14, 9,
    13, 8,
    7, 6, 13, 11, 10,
    10, 7, 7, 4, 9,
    5, 7, 13, 12, 4
};

enum {
    TOK_NONE       = 0,
    TOK_IDENT      = 1,
    TOK_SLASH      = 2,
    TOK_QUOTE      = 3,
    TOK_APOSTROPHE = 4
};

static uint8_t KW_TABLE[256];
static uint8_t TOKEN_KIND[256];
static uint8_t IDENT_CONT[256];
static bool    TABLES_INITIALIZED = false;

static inline uint32_t kw_hash(uint8_t first, uint8_t last, size_t len) {
    return ((uint32_t)first + (uint32_t)last * 9U + (uint32_t)len * 11U) & KW_SLOT_MASK;
}

static void init_tables(void) {
    memset(KW_TABLE, KW_EMPTY, sizeof(KW_TABLE));
    for (uint32_t i = 0; i < KEYWORD_COUNT; i++) {
        const char *name = KEYWORD_NAMES[i];
        size_t len = KEYWORD_LENS[i];
        assert(strlen(name) == len);
        assert(len >= 2 && len <= KW_MAX_LEN);
        uint32_t h = kw_hash((uint8_t)name[0], (uint8_t)name[len - 1], len);
        assert(KW_TABLE[h] == KW_EMPTY && "Keyword minimal perfect hash collision!");
        KW_TABLE[h] = (uint8_t)i;
    }

    memset(TOKEN_KIND, TOK_NONE, sizeof(TOKEN_KIND));
    memset(IDENT_CONT, 0, sizeof(IDENT_CONT));
    for (int c = 'a'; c <= 'z'; c++) {
        TOKEN_KIND[c] = TOK_IDENT;
        IDENT_CONT[c] = 1;
    }
    for (int c = 'A'; c <= 'Z'; c++) {
        TOKEN_KIND[c] = TOK_IDENT;
        IDENT_CONT[c] = 1;
    }
    for (int c = '0'; c <= '9'; c++) {
        IDENT_CONT[c] = 1;
    }
    TOKEN_KIND['_']  = TOK_IDENT;
    IDENT_CONT['_']  = 1;
    TOKEN_KIND['/']  = TOK_SLASH;
    TOKEN_KIND['"']  = TOK_QUOTE;
    TOKEN_KIND['\''] = TOK_APOSTROPHE;
    TABLES_INITIALIZED = true;
}

static inline size_t count_newlines_avx2(const uint8_t *ptr, size_t len) {
    const __m256i target = _mm256_set1_epi8('\n');
    const __m256i zero = _mm256_setzero_si256();
    size_t count = 0;
    size_t i = 0;

    while (i + 32 <= len) {
        __m256i acc = _mm256_setzero_si256();
        size_t block_end = i + 32 * 255;
        size_t aligned_end = len & ~(size_t)31;
        if (block_end > aligned_end) block_end = aligned_end;
        while (i < block_end) {
            __m256i chunk = _mm256_loadu_si256((const __m256i *)(ptr + i));
            __m256i cmp = _mm256_cmpeq_epi8(chunk, target);
            acc = _mm256_sub_epi8(acc, cmp);
            i += 32;
        }
        __m256i sad = _mm256_sad_epu8(acc, zero);
        count += (size_t)(_mm256_extract_epi64(sad, 0) +
                          _mm256_extract_epi64(sad, 1) +
                          _mm256_extract_epi64(sad, 2) +
                          _mm256_extract_epi64(sad, 3));
    }
    while (i < len) {
        if (ptr[i] == '\n') count++;
        i++;
    }
    return count;
}

static inline ptrdiff_t find_byte_avx2(const uint8_t *ptr, size_t len, uint8_t needle) {
    const __m256i target = _mm256_set1_epi8((char)needle);
    size_t i = 0;
    while (i + 32 <= len) {
        __m256i chunk = _mm256_loadu_si256((const __m256i *)(ptr + i));
        __m256i cmp = _mm256_cmpeq_epi8(chunk, target);
        uint32_t mask = (uint32_t)_mm256_movemask_epi8(cmp);
        if (mask != 0) {
            return (ptrdiff_t)(i + (size_t)__builtin_ctz(mask));
        }
        i += 32;
    }
    while (i < len) {
        if (ptr[i] == needle) return (ptrdiff_t)i;
        i++;
    }
    return -1;
}

static inline ptrdiff_t find_star_slash(const uint8_t *slice, size_t len) {
    if (len < 2) return -1;
    const __m256i stars = _mm256_set1_epi8('*');
    const __m256i slashes = _mm256_set1_epi8('/');
    size_t i = 0;
    while (i + 33 <= len) {
        __m256i c0 = _mm256_loadu_si256((const __m256i *)(slice + i));
        __m256i c1 = _mm256_loadu_si256((const __m256i *)(slice + i + 1));
        __m256i eq_star = _mm256_cmpeq_epi8(c0, stars);
        __m256i eq_slash = _mm256_cmpeq_epi8(c1, slashes);
        uint32_t mask = (uint32_t)_mm256_movemask_epi8(_mm256_and_si256(eq_star, eq_slash));
        if (mask != 0) {
            return (ptrdiff_t)(i + (size_t)__builtin_ctz(mask));
        }
        i += 32;
    }
    while (i + 1 < len) {
        if (slice[i] == '*' && slice[i + 1] == '/') {
            return (ptrdiff_t)i;
        }
        i++;
    }
    return -1;
}

static inline size_t skip_line_comment(const uint8_t *content, size_t content_len, size_t start_i) {
    size_t i = start_i;
    while (i < content_len) {
        ptrdiff_t pos = find_byte_avx2(content + i, content_len - i, '\n');
        if (pos < 0) {
            return content_len;
        }
        size_t nl_idx = i + (size_t)pos;
        size_t check = nl_idx;
        if (check > start_i && content[check - 1] == '\r') {
            check--;
        }
        if (check > start_i && content[check - 1] == '\\') {
            i = nl_idx + 1;
            continue;
        }
        return nl_idx + 1;
    }
    return content_len;
}

static inline size_t skip_literal(const uint8_t *content, size_t content_len, size_t start_i, uint8_t quote) {
    size_t i = start_i;
    while (i < content_len) {
        ptrdiff_t pos = find_byte_avx2(content + i, content_len - i, quote);
        if (pos >= 0) {
            i += (size_t)pos;
            size_t escapes = 0;
            size_t j = i;
            while (j > start_i && content[j - 1] == '\\') {
                escapes++;
                j--;
            }
            if ((escapes & 1U) == 0) {
                return i + 1;
            }
            i++;
        } else {
            return content_len;
        }
    }
    return content_len;
}

static inline bool kw_match_len_2_to_14(const uint8_t *cand, const uint8_t *kw, size_t n) {
    assert(n >= 2 && n <= KW_MAX_LEN);
    if (n >= 8) {
        uint64_t a1, b1, a2, b2;
        memcpy(&a1, cand, 8);
        memcpy(&b1, kw, 8);
        memcpy(&a2, cand + n - 8, 8);
        memcpy(&b2, kw + n - 8, 8);
        return ((a1 ^ b1) | (a2 ^ b2)) == 0;
    } else if (n >= 4) {
        uint32_t a1, b1, a2, b2;
        memcpy(&a1, cand, 4);
        memcpy(&b1, kw, 4);
        memcpy(&a2, cand + n - 4, 4);
        memcpy(&b2, kw + n - 4, 4);
        return ((a1 ^ b1) | (a2 ^ b2)) == 0;
    } else {
        uint16_t a1, b1;
        memcpy(&a1, cand, 2);
        memcpy(&b1, kw, 2);
        return (a1 == b1) && (cand[n - 1] == kw[n - 1]);
    }
}

typedef struct {
    int64_t keyword_counts[KEYWORD_COUNT];
    int64_t total_keywords;
    int64_t lines_processed;
} ParseResult;

static inline size_t skip_non_tokens_avx2(const uint8_t *ptr, size_t len, size_t i) {
    // Exact AVX2 Dual-LUT filter for token starters ('A'-'Z', 'a'-'z', '_', '/', '"', '\'').
    // Note: _mm256_shuffle_epi8 clears lane to 0 if bit 7 of index is set, which automatically
    // rejects all bytes >= 128 without extra instructions!
    // Low nibbles:
    //   0x0        : 'P','p' (bit 1 = 2)
    //   0x1..0xA   : 'A'-'J','Q'-'Z','a'-'j','q'-'z' (bit 0 | bit 1 = 3), plus '"' (0x22) and '\'' (0x27) (bit 3 = 8)
    //   0xB..0xE   : 'K'-'N','k'-'n' (bit 0 = 1)
    //   0xF        : 'O','o' (bit 0 = 1), '_' (0x5F, bit 2 = 4), '/' (0x2F, bit 3 = 8) -> 13
    // High nibbles:
    //   0x2        : '"', '\'', '/' (bit 3 = 8)
    //   0x4, 0x6   : 'A'-'O', 'a'-'o' (bit 0 = 1)
    //   0x5        : 'P'-'Z' (bit 1 = 2), '_' (bit 2 = 4) -> 6
    //   0x7        : 'p'-'z' (bit 1 = 2)
    // Every single byte 0..255 is classified with ZERO false positives and ZERO false negatives!
    const __m256i lut_lo = _mm256_setr_epi8(
        2, 3, 11, 3, 3, 3, 3, 11, 3, 3, 3, 1, 1, 1, 1, 13,
        2, 3, 11, 3, 3, 3, 3, 11, 3, 3, 3, 1, 1, 1, 1, 13
    );
    const __m256i lut_hi = _mm256_setr_epi8(
        0, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 8, 0, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0
    );
    const __m256i mask_0f = _mm256_set1_epi8(0x0F);
    const __m256i zero = _mm256_setzero_si256();

    while (i + 32 <= len) {
        __m256i v = _mm256_loadu_si256((const __m256i *)(ptr + i));
        __m256i hi = _mm256_and_si256(_mm256_srli_epi16(v, 4), mask_0f);
        __m256i m_lo = _mm256_shuffle_epi8(lut_lo, v);
        __m256i m_hi = _mm256_shuffle_epi8(lut_hi, hi);
        __m256i hit = _mm256_and_si256(m_lo, m_hi);
        __m256i is_zero = _mm256_cmpeq_epi8(hit, zero);
        uint32_t mask = ~(uint32_t)_mm256_movemask_epi8(is_zero);
        if (mask != 0) {
            return i + (size_t)__builtin_ctz(mask);
        }
        i += 32;
    }
    while (i < len && TOKEN_KIND[ptr[i]] == TOK_NONE) {
        i++;
    }
    return i;
}

static inline size_t skip_ident_cont_avx2(const uint8_t *ptr, size_t len, size_t i) {
    // Exact AVX2 Dual-LUT filter for identifier continuation ('A'-'Z', 'a'-'z', '0'-'9', '_'):
    //   0x3 ('0'-'9', low 0x0..0x9) -> bit 3 (8)
    //   0x4, 0x6 ('A'-'O', 'a'-'o', low 0x1..0xF) -> bit 0 (1)
    //   0x5 ('P'-'Z' low 0x0..0xA -> bit 1 (2), '_' low 0xF -> bit 2 (4)) -> 6
    //   0x7 ('p'-'z' low 0x0..0xA) -> bit 1 (2)
    const __m256i lut_lo = _mm256_setr_epi8(
        10, 11, 11, 11, 11, 11, 11, 11, 11, 11, 3, 1, 1, 1, 1, 5,
        10, 11, 11, 11, 11, 11, 11, 11, 11, 11, 3, 1, 1, 1, 1, 5
    );
    const __m256i lut_hi = _mm256_setr_epi8(
        0, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 8, 1, 6, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0
    );
    const __m256i mask_0f = _mm256_set1_epi8(0x0F);
    const __m256i zero = _mm256_setzero_si256();

    while (i + 32 <= len) {
        __m256i v = _mm256_loadu_si256((const __m256i *)(ptr + i));
        __m256i hi = _mm256_and_si256(_mm256_srli_epi16(v, 4), mask_0f);
        __m256i m_lo = _mm256_shuffle_epi8(lut_lo, v);
        __m256i m_hi = _mm256_shuffle_epi8(lut_hi, hi);
        __m256i hit = _mm256_and_si256(m_lo, m_hi);
        __m256i is_non_ident = _mm256_cmpeq_epi8(hit, zero);
        uint32_t mask = (uint32_t)_mm256_movemask_epi8(is_non_ident);
        if (mask != 0) {
            return i + (size_t)__builtin_ctz(mask);
        }
        i += 32;
    }
    while (i < len && IDENT_CONT[ptr[i]]) {
        i++;
    }
    return i;
}

static void parse_into(const uint8_t *content, size_t content_len, ParseResult *acc) {
    assert(TABLES_INITIALIZED);
    assert(acc != NULL);
    if (content_len == 0) return;
    assert(content != NULL);

    acc->lines_processed += (int64_t)count_newlines_avx2(content, content_len);

    size_t i = 0;
    while (i < content_len) {
        if (TOKEN_KIND[content[i]] == TOK_NONE) {
            i++;
            if (i < content_len && TOKEN_KIND[content[i]] == TOK_NONE) {
                i = skip_non_tokens_avx2(content, content_len, i);
            }
        }
        if (i >= content_len) break;

        switch (TOKEN_KIND[content[i]]) {
            case TOK_IDENT: {
                size_t start = i;
                i = skip_ident_cont_avx2(content, content_len, i + 1);
                size_t n = i - start;
                if (n - 2U <= KW_MAX_LEN - 2U) {
                    uint32_t h = kw_hash(content[start], content[i - 1], n);
                    uint8_t id = KW_TABLE[h];
                    if (id != KW_EMPTY && KEYWORD_LENS[id] == n) {
                        if (kw_match_len_2_to_14(content + start, (const uint8_t *)KEYWORD_NAMES[id], n)) {
                            acc->keyword_counts[id]++;
                            acc->total_keywords++;
                        }
                    }
                }
                break;
            }
            case TOK_SLASH: {
                if (i + 1 < content_len) {
                    uint8_t next_b = content[i + 1];
                    if (next_b == '/') {
                        i = skip_line_comment(content, content_len, i + 2);
                        continue;
                    } else if (next_b == '*') {
                        ptrdiff_t pos = find_star_slash(content + i + 2, content_len - (i + 2));
                        if (pos >= 0) {
                            i += 2 + (size_t)pos + 2;
                        } else {
                            i = content_len;
                        }
                        continue;
                    }
                }
                i++;
                break;
            }
            case TOK_QUOTE:
                i = skip_literal(content, content_len, i + 1, '"');
                break;
            case TOK_APOSTROPHE: {
                // C23 digit separator check: if preceded by [0-9a-fA-F] and followed by [0-9a-fA-F],
                // it is a numeric digit separator (e.g., 1'000'000 or 0xFF'FF), not a character literal!
                if (i > 0 && i + 1 < content_len) {
                    uint8_t prev = content[i - 1];
                    uint8_t next = content[i + 1];
                    if (IDENT_CONT[prev] && IDENT_CONT[next]) {
                        i++;
                        break;
                    }
                }
                i = skip_literal(content, content_len, i + 1, '\'');
                break;
            }
            default:
                i++;
                break;
        }
    }
}

// Strict C file extension check: only .c and .h (case-insensitive: .c, .h, .C, .H).
// Excludes C++ extensions (.cpp, .cc, .cxx, .hpp, .hxx, .c++, .h++).
static inline bool is_c_extension(const uint8_t *name, size_t name_len) {
    if (name_len < 2) return false;
    // Fast check: since only 2-byte extensions (".c", ".h", ".C", ".H") are valid C extensions,
    // any valid filename must end with '.' at name_len - 2 and 'c'/'h'/'C'/'H' at name_len - 1,
    // unless the filename is literally ".c" or ".h" (which has dot_idx == 0, still valid).
    if (name[name_len - 2] != '.') return false;
    uint8_t c = name[name_len - 1] | 0x20U;
    return c == 'c' || c == 'h';
}

#define BATCH_SIZE     64
#define BATCH_BUF_SIZE 8192

typedef struct {
    long     dirfd;
    uint8_t  data[BATCH_BUF_SIZE];
    uint16_t offsets[BATCH_SIZE];
    uint16_t name_offsets[BATCH_SIZE];
    uint16_t lens[BATCH_SIZE];
    size_t   data_len;
    size_t   count;
} Batch;

static inline void batch_reset(Batch *b) {
    assert(b != NULL);
    b->dirfd = AT_FDCWD_VAL;
    b->data_len = 0;
    b->count = 0;
}

static inline bool batch_append(Batch *b, const char *dir, size_t dir_len, const char *name, size_t name_len, bool *too_long) {
    assert(b != NULL);
    assert(dir != NULL);
    assert(name != NULL);
    bool has_slash = dir_len > 0 && dir[dir_len - 1] == '/';
    size_t sep_len = has_slash ? 0 : 1;
    size_t path_len = dir_len + sep_len + name_len;
    size_t needed = path_len + 1;
    if (needed > BATCH_BUF_SIZE) {
        if (too_long) *too_long = true;
        return false;
    }
    if (too_long) *too_long = false;
    if (b->count >= BATCH_SIZE || b->data_len + needed > BATCH_BUF_SIZE) {
        return false;
    }
    size_t off = b->data_len;
    memcpy(b->data + off, dir, dir_len);
    if (!has_slash) {
        b->data[off + dir_len] = '/';
    }
    size_t name_off = off + dir_len + sep_len;
    memcpy(b->data + name_off, name, name_len);
    b->data[off + path_len] = '\0';

    b->offsets[b->count] = (uint16_t)off;
    b->name_offsets[b->count] = (uint16_t)name_off;
    b->lens[b->count] = (uint16_t)path_len;
    b->data_len = off + needed;
    b->count++;
    return true;
}

static inline const char *batch_get_path(const Batch *b, size_t idx) {
    assert(b != NULL);
    assert(idx < b->count);
    return (const char *)(b->data + b->offsets[idx]);
}

static inline const char *batch_get_rel_name(const Batch *b, size_t idx) {
    assert(b != NULL);
    assert(idx < b->count);
    return (const char *)(b->data + b->name_offsets[idx]);
}

typedef struct {
    char  *ptr;
    size_t len;
} DirEntry;

enum {
    TASK_DONE  = 0,
    TASK_BATCH = 1,
    TASK_DIR   = 2
};

typedef struct {
    int      kind;
    Batch   *batch;
    DirEntry dir;
} Task;

typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t  cond;

    DirEntry *dirs;
    size_t    dirs_len;
    size_t    dirs_cap;

    Batch   **ready_batches;
    size_t    ready_len;
    size_t    ready_cap;

    Batch   **free_batches;
    size_t    free_len;
    size_t    free_cap;

    Batch   **all_batches;
    size_t    all_len;
    size_t    all_cap;

    size_t    active_workers;
    bool      done;
} WorkScheduler;

static void sched_init(WorkScheduler *s, const char *root, size_t root_len) {
    assert(s != NULL);
    assert(root != NULL);
    pthread_mutex_init(&s->mutex, NULL);
    pthread_cond_init(&s->cond, NULL);

    s->dirs_cap = 64;
    s->dirs_len = 0;
    s->dirs = (DirEntry *)malloc(s->dirs_cap * sizeof(DirEntry));
    assert(s->dirs != NULL);

    s->ready_cap = 64;
    s->ready_len = 0;
    s->ready_batches = (Batch **)malloc(s->ready_cap * sizeof(Batch *));
    assert(s->ready_batches != NULL);

    s->free_cap = 64;
    s->free_len = 0;
    s->free_batches = (Batch **)malloc(s->free_cap * sizeof(Batch *));
    assert(s->free_batches != NULL);

    s->all_cap = 64;
    s->all_len = 0;
    s->all_batches = (Batch **)malloc(s->all_cap * sizeof(Batch *));
    assert(s->all_batches != NULL);

    s->active_workers = 0;
    s->done = false;

    char *root_copy = (char *)malloc(root_len + 1);
    assert(root_copy != NULL);
    memcpy(root_copy, root, root_len);
    root_copy[root_len] = '\0';
    s->dirs[s->dirs_len++] = (DirEntry){ .ptr = root_copy, .len = root_len };
}

static void sched_deinit(WorkScheduler *s) {
    assert(s != NULL);
    for (size_t i = 0; i < s->dirs_len; i++) {
        free(s->dirs[i].ptr);
    }
    free(s->dirs);
    for (size_t i = 0; i < s->all_len; i++) {
        free(s->all_batches[i]);
    }
    free(s->ready_batches);
    free(s->free_batches);
    free(s->all_batches);
    pthread_cond_destroy(&s->cond);
    pthread_mutex_destroy(&s->mutex);
}

static inline void push_ready(WorkScheduler *s, Batch *b) {
    assert(b != NULL && b->count > 0);
    if (s->ready_len == s->ready_cap) {
        s->ready_cap *= 2;
        s->ready_batches = (Batch **)realloc(s->ready_batches, s->ready_cap * sizeof(Batch *));
        assert(s->ready_batches != NULL);
    }
    s->ready_batches[s->ready_len++] = b;
}

static inline void push_free(WorkScheduler *s, Batch *b) {
    assert(b != NULL);
    if (s->free_len == s->free_cap) {
        s->free_cap *= 2;
        s->free_batches = (Batch **)realloc(s->free_batches, s->free_cap * sizeof(Batch *));
        assert(s->free_batches != NULL);
    }
    s->free_batches[s->free_len++] = b;
}

static inline void push_all(WorkScheduler *s, Batch *b) {
    assert(b != NULL);
    if (s->all_len == s->all_cap) {
        s->all_cap *= 2;
        s->all_batches = (Batch **)realloc(s->all_batches, s->all_cap * sizeof(Batch *));
        assert(s->all_batches != NULL);
    }
    s->all_batches[s->all_len++] = b;
}

static Batch *sched_acquire_batch(WorkScheduler *s) {
    pthread_mutex_lock(&s->mutex);
    if (s->free_len > 0) {
        Batch *b = s->free_batches[--s->free_len];
        pthread_mutex_unlock(&s->mutex);
        batch_reset(b);
        return b;
    }
    Batch *b = (Batch *)malloc(sizeof(Batch));
    assert(b != NULL);
    push_all(s, b);
    pthread_mutex_unlock(&s->mutex);
    batch_reset(b);
    return b;
}

static Batch *sched_submit_batch_and_swap(WorkScheduler *s, Batch *full_batch) {
    assert(full_batch != NULL && full_batch->count > 0);
    pthread_mutex_lock(&s->mutex);
    push_ready(s, full_batch);
    pthread_cond_signal(&s->cond);
    if (s->free_len > 0) {
        Batch *next = s->free_batches[--s->free_len];
        pthread_mutex_unlock(&s->mutex);
        batch_reset(next);
        return next;
    }
    Batch *b = (Batch *)malloc(sizeof(Batch));
    assert(b != NULL);
    push_all(s, b);
    pthread_mutex_unlock(&s->mutex);
    batch_reset(b);
    return b;
}

static Task sched_complete_batch_and_next(WorkScheduler *s, Batch *finished_batch, Batch **local_batch) {
    pthread_mutex_lock(&s->mutex);
    if (finished_batch != NULL) {
        push_free(s, finished_batch);
        assert(s->active_workers > 0);
        s->active_workers--;
    }

    for (;;) {
        if (s->ready_len > 0) {
            Batch *b = s->ready_batches[--s->ready_len];
            s->active_workers++;
            pthread_mutex_unlock(&s->mutex);
            return (Task){ .kind = TASK_BATCH, .batch = b };
        }
        if (s->dirs_len > 0) {
            DirEntry d = s->dirs[--s->dirs_len];
            s->active_workers++;
            pthread_mutex_unlock(&s->mutex);
            return (Task){ .kind = TASK_DIR, .dir = d };
        }
        if (*local_batch != NULL) {
            Batch *lb = *local_batch;
            *local_batch = NULL;
            if (lb->count > 0) {
                s->active_workers++;
                pthread_mutex_unlock(&s->mutex);
                return (Task){ .kind = TASK_BATCH, .batch = lb };
            } else {
                push_free(s, lb);
            }
        }
        if (s->active_workers == 0) {
            s->done = true;
            pthread_cond_broadcast(&s->cond);
            pthread_mutex_unlock(&s->mutex);
            return (Task){ .kind = TASK_DONE };
        }
        pthread_cond_wait(&s->cond, &s->mutex);
        if (s->done && s->ready_len == 0 && s->dirs_len == 0) {
            pthread_mutex_unlock(&s->mutex);
            return (Task){ .kind = TASK_DONE };
        }
    }
}

static void sched_finish_dir(WorkScheduler *s, const DirEntry *subdirs, size_t subdirs_len, Batch **local_batch) {
    pthread_mutex_lock(&s->mutex);
    if (subdirs_len > 0) {
        while (s->dirs_len + subdirs_len > s->dirs_cap) {
            s->dirs_cap *= 2;
            s->dirs = (DirEntry *)realloc(s->dirs, s->dirs_cap * sizeof(DirEntry));
            assert(s->dirs != NULL);
        }
        memcpy(s->dirs + s->dirs_len, subdirs, subdirs_len * sizeof(DirEntry));
        s->dirs_len += subdirs_len;
    }

    // Flush local_batch if it has >= 16 items, OR if there are no remaining directories
    // queued and no other active workers so we never broadcast `done = true` while holding
    // unsubmitted files in `local_batch`.
    Batch *flush_batch = NULL;
    if (*local_batch != NULL && (*local_batch)->count > 0) {
        if ((*local_batch)->count >= 16 || (s->dirs_len == 0 && s->active_workers == 1)) {
            flush_batch = *local_batch;
            *local_batch = NULL;
            push_ready(s, flush_batch);
        }
    }

    assert(s->active_workers > 0);
    s->active_workers--;

    size_t new_work = subdirs_len + (flush_batch != NULL ? 1U : 0U);
    if (new_work > 1) {
        pthread_cond_broadcast(&s->cond);
    } else if (new_work == 1) {
        pthread_cond_signal(&s->cond);
    } else if (s->active_workers == 0 && s->dirs_len == 0 && s->ready_len == 0) {
        assert(*local_batch == NULL || (*local_batch)->count == 0);
        s->done = true;
        pthread_cond_broadcast(&s->cond);
    }
    pthread_mutex_unlock(&s->mutex);
}

typedef struct {
    uint8_t *data;
    size_t   cap;
    size_t   len;
} FileBuf;

static bool read_file_into(long dirfd, const char *path, FileBuf *fb) {
    assert(path != NULL);
    assert(fb != NULL);
    long fd = sys_openat(dirfd, path, O_RDONLY_VAL | O_NONBLOCK_VAL | O_CLOEXEC_VAL);
    if (fd < 0) return false;

    if (fb->cap < 131072) {
        fb->cap = 131072;
        fb->data = (uint8_t *)realloc(fb->data, fb->cap);
        assert(fb->data != NULL);
    }

    size_t total = 0;
    for (;;) {
        if (total == fb->cap) {
            fb->cap *= 2;
            fb->data = (uint8_t *)realloc(fb->data, fb->cap);
            assert(fb->data != NULL);
        }
        size_t avail = fb->cap - total;
        long n = sys_read(fd, fb->data + total, avail);
        if (n < 0) {
            if (n == EINTR_VAL) continue;
            sys_close(fd);
            fb->len = 0;
            return false;
        }
        if (n == 0) break;
        total += (size_t)n;
    }

    sys_close(fd);
    fb->len = total;
    return true;
}

typedef struct {
    ParseResult parse_result;
    int64_t     files_found;
    int64_t     files_processed;
} WorkerResult;

typedef struct {
    WorkScheduler *sched;
    WorkerResult   result;
} WorkerContext;

static void worker_result_add(WorkerResult *dst, const WorkerResult *src) {
    assert(dst != NULL && src != NULL);
    for (int i = 0; i < KEYWORD_COUNT; i++) {
        dst->parse_result.keyword_counts[i] += src->parse_result.keyword_counts[i];
    }
    dst->parse_result.total_keywords += src->parse_result.total_keywords;
    dst->parse_result.lines_processed += src->parse_result.lines_processed;
    dst->files_found += src->files_found;
    dst->files_processed += src->files_processed;
}

static void process_single_path(long dirfd, const char *path, WorkerResult *res, FileBuf *fb) {
    res->files_found++;
    if (read_file_into(dirfd, path, fb)) {
        res->files_processed++;
        parse_into(fb->data, fb->len, &res->parse_result);
    }
}

static void *unified_worker_proc(void *arg) {
    WorkerContext *ctx = (WorkerContext *)arg;
    assert(ctx != NULL && ctx->sched != NULL);
    WorkScheduler *sched = ctx->sched;

    FileBuf fb = { .data = NULL, .cap = 0, .len = 0 };
    uint8_t dents_buf[32768] __attribute__((aligned(8)));

    size_t subdirs_cap = 32;
    size_t subdirs_len = 0;
    DirEntry *subdirs = (DirEntry *)malloc(subdirs_cap * sizeof(DirEntry));
    assert(subdirs != NULL);

    Batch *local_batch = NULL;
    Batch *prev_finished_batch = NULL;

    for (;;) {
        Task task = sched_complete_batch_and_next(sched, prev_finished_batch, &local_batch);
        prev_finished_batch = NULL;

        if (task.kind == TASK_DONE) {
            break;
        } else if (task.kind == TASK_BATCH) {
            Batch *b = task.batch;
            assert(b != NULL);
            long b_dirfd = b->dirfd;
            if (b_dirfd >= 0) {
                for (size_t i = 0; i < b->count; i++) {
                    const char *name = batch_get_rel_name(b, i);
                    process_single_path(b_dirfd, name, &ctx->result, &fb);
                }
                sys_close(b_dirfd);
                b->dirfd = AT_FDCWD_VAL;
            } else {
                for (size_t i = 0; i < b->count; i++) {
                    const char *path = batch_get_path(b, i);
                    process_single_path(AT_FDCWD_VAL, path, &ctx->result, &fb);
                }
            }
            prev_finished_batch = b;
        } else if (task.kind == TASK_DIR) {
            DirEntry dir = task.dir;
            if (local_batch == NULL) {
                local_batch = sched_acquire_batch(sched);
            }
            bool batch_started_empty = (local_batch->count == 0);

            long fd = sys_openat(AT_FDCWD_VAL, dir.ptr, O_RDONLY_VAL | O_DIRECTORY_VAL | O_CLOEXEC_VAL);
            if (fd >= 0) {
                subdirs_len = 0;

                for (;;) {
                    long written = sys_getdents64(fd, dents_buf, sizeof(dents_buf));
                    if (written <= 0) break;

                    size_t offset = 0;
                    size_t end = (size_t)written;
                    while (offset < end) {
                        uint16_t reclen;
                        memcpy(&reclen, dents_buf + offset + 16, sizeof(uint16_t));
                        assert(reclen >= 19);
                        uint8_t d_type = dents_buf[offset + 18];
                        const char *name = (const char *)(dents_buf + offset + 19);
                        offset += reclen;

                        if (name[0] == '.' && (name[1] == '\0' || (name[1] == '.' && name[2] == '\0'))) {
                            continue;
                        }
                        size_t name_len = strlen(name);

                        bool is_dir = (d_type == DT_DIR_VAL);
                        bool is_reg = (d_type == DT_REG_VAL);
                        if (d_type == DT_UNKNOWN_VAL) {
                            struct c_statx stx;
                            long s_res = sys_statx(fd, name, AT_SYMLINK_NOFOLLOW_V, STATX_TYPE_VAL, &stx);
                            if (s_res == 0) {
                                uint16_t ftype = stx.stx_mode & S_IFMT_VAL;
                                is_dir = (ftype == S_IFDIR_VAL);
                                is_reg = (ftype == S_IFREG_VAL);
                            }
                        }

                        if (is_dir) {
                            bool has_slash = dir.len > 0 && dir.ptr[dir.len - 1] == '/';
                            size_t sep_len = has_slash ? 0 : 1;
                            size_t sub_len = dir.len + sep_len + name_len;
                            char *sub = (char *)malloc(sub_len + 1);
                            assert(sub != NULL);
                            memcpy(sub, dir.ptr, dir.len);
                            if (!has_slash) sub[dir.len] = '/';
                            memcpy(sub + dir.len + sep_len, name, name_len);
                            sub[sub_len] = '\0';

                            if (subdirs_len == subdirs_cap) {
                                subdirs_cap *= 2;
                                subdirs = (DirEntry *)realloc(subdirs, subdirs_cap * sizeof(DirEntry));
                                assert(subdirs != NULL);
                            }
                            subdirs[subdirs_len++] = (DirEntry){ .ptr = sub, .len = sub_len };
                        } else if (is_reg && is_c_extension((const uint8_t *)name, name_len)) {
                            bool too_long = false;
                            if (!batch_append(local_batch, dir.ptr, dir.len, name, name_len, &too_long)) {
                                if (too_long) {
                                    // Path exceeds 8KB batch buffer; open directly relative to `fd`!
                                    process_single_path(fd, name, &ctx->result, &fb);
                                } else {
                                    if (batch_started_empty) {
                                        long dup_fd = sys_dup(fd);
                                        if (dup_fd >= 0) local_batch->dirfd = dup_fd;
                                    }
                                    local_batch = sched_submit_batch_and_swap(sched, local_batch);
                                    batch_started_empty = true;
                                    bool ok = batch_append(local_batch, dir.ptr, dir.len, name, name_len, NULL);
                                    assert(ok);
                                    (void)ok;
                                }
                            }
                        }
                    }
                }

                // If `local_batch` is going to be flushed (>= 16 files) and contains ONLY files from
                // this directory (`batch_started_empty`), attach a dup of `fd` so the worker that
                // processes the batch can `openat(dirfd, rel_name)` without kernel path resolution!
                if (batch_started_empty && local_batch != NULL && local_batch->count >= 16) {
                    long dup_fd = sys_dup(fd);
                    if (dup_fd >= 0) local_batch->dirfd = dup_fd;
                }
                sys_close(fd);
                sched_finish_dir(sched, subdirs, subdirs_len, &local_batch);
                subdirs_len = 0;
            } else {
                sched_finish_dir(sched, NULL, 0, &local_batch);
            }

            free(dir.ptr);
        }
    }

    free(subdirs);
    free(fb.data);
    return NULL;
}

static const char *format_number(int64_t num, char buf[32]) {
    char tmp[32];
    int n = snprintf(tmp, sizeof(tmp), "%" PRId64, num);
    assert(n > 0 && n < (int)sizeof(tmp));
    if (num < 0 || n <= 3) {
        memcpy(buf, tmp, (size_t)n + 1);
        return buf;
    }
    int commas = (n - 1) / 3;
    int total_len = n + commas;
    assert(total_len < 32);
    buf[total_len] = '\0';
    int dest = total_len;
    int src = n;
    int count = 0;
    while (src > 0) {
        if (count == 3) {
            buf[--dest] = ',';
            count = 0;
        }
        buf[--dest] = tmp[--src];
        count++;
    }
    return buf;
}

typedef struct {
    const char *keyword;
    int64_t     count;
    double      percentage;
} KeywordStat;

static int cmp_stat_desc(const void *a, const void *b) {
    const KeywordStat *sa = (const KeywordStat *)a;
    const KeywordStat *sb = (const KeywordStat *)b;
    if (sb->count != sa->count) {
        return (sb->count > sa->count) - (sb->count < sa->count);
    }
    return strcmp(sa->keyword, sb->keyword);
}

static void print_results(const WorkerResult *totals, double processing_time, int num_workers, int cpu_count) {
    char out[8192];
    size_t pos = 0;
    char nb1[32], nb2[32], nb3[32], nb4[32];

    pos += (size_t)snprintf(out + pos, sizeof(out) - pos,
        "Processed %s files with %d worker threads (CPU cores: %d)...\n"
        "C Keyword Analyzer (C Ultra - Unified Work-Stealing)\n"
        "Files processed: %s\n"
        "Total lines: %s\n"
        "Total keywords: %s\n"
        "Processing time: %.3f seconds\n",
        format_number(totals->files_found, nb1),
        num_workers,
        cpu_count,
        format_number(totals->files_processed, nb2),
        format_number(totals->parse_result.lines_processed, nb3),
        format_number(totals->parse_result.total_keywords, nb4),
        processing_time);

    KeywordStat stats[KEYWORD_COUNT];
    size_t stats_count = 0;
    int64_t total_kw = totals->parse_result.total_keywords;

    for (int i = 0; i < KEYWORD_COUNT; i++) {
        int64_t c = totals->parse_result.keyword_counts[i];
        if (c > 0) {
            double pct = total_kw > 0 ? ((double)c / (double)total_kw) * 100.0 : 0.0;
            stats[stats_count++] = (KeywordStat){
                .keyword = KEYWORD_NAMES[i],
                .count = c,
                .percentage = pct
            };
        }
    }

    qsort(stats, stats_count, sizeof(KeywordStat), cmp_stat_desc);

    pos += (size_t)snprintf(out + pos, sizeof(out) - pos,
        "\nKeyword frequencies (most frequent first):\n"
        "Rank Keyword         Count Percentage\n"
        "---- -------         ----- ----------\n");

    for (size_t i = 0; i < stats_count; i++) {
        pos += (size_t)snprintf(out + pos, sizeof(out) - pos,
            "%4zu %-15s %10s %6.2f%%\n",
            i + 1,
            stats[i].keyword,
            format_number(stats[i].count, nb1),
            stats[i].percentage);
    }

    sys_write(1, out, pos);
}

static inline int64_t get_monotonic_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000000000LL + (int64_t)ts.tv_nsec;
}

static int get_cpu_count(void) {
    cpu_set_t set;
    CPU_ZERO(&set);
    if (sched_getaffinity(0, sizeof(set), &set) == 0) {
        int c = CPU_COUNT(&set);
        if (c > 0) return c;
    }
    return 1;
}

#ifndef C_ULTRA_TEST
int main(int argc, char **argv) {
    int64_t start_ns = get_monotonic_ns();
    init_tables();

    if (argc < 2) {
        const char *prog = argc > 0 ? argv[0] : "c_ultra";
        printf("Usage: %s <file_or_directory>\nAnalyzes C keyword frequencies in source files\n", prog);
        return 1;
    }

    const char *target = argv[1];
    size_t target_len = strlen(target);

    struct c_statx stx;
    if (sys_statx(AT_FDCWD_VAL, target, 0, STATX_TYPE_VAL, &stx) != 0) {
        fprintf(stderr, "Error: Cannot access %s\n", target);
        return 1;
    }
    uint16_t ftype = stx.stx_mode & S_IFMT_VAL;
    bool is_dir = (ftype == S_IFDIR_VAL);
    bool is_reg = (ftype == S_IFREG_VAL);

    int cpu_count = get_cpu_count();
    WorkerResult totals;
    memset(&totals, 0, sizeof(totals));
    int num_workers = cpu_count;

    if (is_dir) {
        WorkScheduler sched;
        sched_init(&sched, target, target_len);

        int extra_threads = cpu_count - 1;
        pthread_t *threads = extra_threads > 0 ? (pthread_t *)malloc((size_t)extra_threads * sizeof(pthread_t)) : NULL;
        WorkerContext *ctxs = (WorkerContext *)calloc((size_t)cpu_count, sizeof(WorkerContext));
        assert(ctxs != NULL);
        for (int i = 0; i < cpu_count; i++) {
            ctxs[i].sched = &sched;
        }

        pthread_attr_t attr;
        pthread_attr_init(&attr);
        pthread_attr_setstacksize(&attr, 512 * 1024);

        for (int i = 0; i < extra_threads; i++) {
            int rc = pthread_create(&threads[i], &attr, unified_worker_proc, &ctxs[i + 1]);
            assert(rc == 0);
            (void)rc;
        }
        pthread_attr_destroy(&attr);

        // Main thread participates as worker 0!
        unified_worker_proc(&ctxs[0]);
        worker_result_add(&totals, &ctxs[0].result);

        for (int i = 0; i < extra_threads; i++) {
            pthread_join(threads[i], NULL);
            worker_result_add(&totals, &ctxs[i + 1].result);
        }

        free(threads);
        free(ctxs);
        sched_deinit(&sched);
    } else {
        if (!is_reg || !is_c_extension((const uint8_t *)target, target_len)) {
            fprintf(stderr, "Error: %s is not a C source file\n", target);
            return 1;
        }
        num_workers = 1;
        totals.files_found = 1;
        FileBuf fb = { .data = NULL, .cap = 0, .len = 0 };
        if (read_file_into(AT_FDCWD_VAL, target, &fb)) {
            totals.files_processed = 1;
            parse_into(fb.data, fb.len, &totals.parse_result);
        }
        free(fb.data);
    }

    if (totals.files_found == 0) {
        printf("No C source files found\n");
        return 1;
    }

    int64_t end_ns = get_monotonic_ns();
    double processing_time = (double)(end_ns - start_ns) / 1000000000.0;

    print_results(&totals, processing_time, num_workers, cpu_count);
    return 0;
}
#endif
