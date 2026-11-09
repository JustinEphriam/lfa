// Comprehensive Test Suite for C Keyword Analyzer (c_ultra)
#define C_ULTRA_TEST
#include "main.c"
#include <sys/stat.h>
#include <unistd.h>

static int find_kw_id(const char *name) {
    for (int i = 0; i < KEYWORD_COUNT; i++) {
        if (strcmp(KEYWORD_NAMES[i], name) == 0) {
            return i;
        }
    }
    return -1;
}

static void test_keyword_table_and_mph(void) {
    init_tables();
    assert(TABLES_INITIALIZED);
    assert(KEYWORD_COUNT == 59);

    bool seen_slot[256] = { false };
    for (int i = 0; i < KEYWORD_COUNT; i++) {
        const char *kw = KEYWORD_NAMES[i];
        size_t len = strlen(kw);
        assert(len == KEYWORD_LENS[i]);
        assert(len >= 2 && len <= KW_MAX_LEN);

        uint32_t h = kw_hash((uint8_t)kw[0], (uint8_t)kw[len - 1], len);
        assert(!seen_slot[h] && "Duplicate hash slot detected!");
        seen_slot[h] = true;
        assert(KW_TABLE[h] == (uint8_t)i);
        assert(kw_match_len_2_to_14((const uint8_t *)kw, (const uint8_t *)kw, len));

        // Verify parsing every single keyword in isolation
        ParseResult res;
        memset(&res, 0, sizeof(res));
        parse_into((const uint8_t *)kw, len, &res);
        assert(res.total_keywords == 1);
        assert(res.keyword_counts[i] == 1);
        assert(res.lines_processed == 0);
    }

    // Check that non-keywords of lengths 1..20 with same first/last byte do not false-match
    const char *non_keywords[] = {
        "a", "i", "if_", "_if", "iff", "inta", "int_t", "returning",
        "autoo", "breaks", "caser", "chars", "consts", "continues",
        "defaults", "d0", "doubles", "elses", "enums", "externs",
        "floats", "forr", "gotos", "longs", "registers", "shorts",
        "signeds", "sizeofs", "statics", "structs", "switchs",
        "typedefs", "unions", "unsigneds", "voids", "volatiles", "whiles",
        "inlines", "restricts", "_Bools", "boolean", "nullptrs",
        "constexprs", "thread_locals", "static_asserts", "_Static_asserts",
        // C++ only keywords must NOT be counted as C keywords
        "class", "namespace", "template", "typename", "virtual", "override",
        "final", "private", "protected", "public", "friend", "using",
        "new", "delete", "try", "catch", "throw", "noexcept", "mutable",
        "explicit", "operator", "dynamic_cast", "static_cast", "reinterpret_cast",
        "const_cast", "typeid", "concept", "requires", "co_await", "co_yield", "co_return"
    };
    for (size_t k = 0; k < sizeof(non_keywords) / sizeof(non_keywords[0]); k++) {
        ParseResult res;
        memset(&res, 0, sizeof(res));
        const char *s = non_keywords[k];
        parse_into((const uint8_t *)s, strlen(s), &res);
        assert(res.total_keywords == 0);
    }
    // Exhaustively verify that `skip_non_tokens_avx2` and `skip_ident_cont_avx2` match
    // `TOKEN_KIND` and `IDENT_CONT` for all 256 byte values with zero false positives/negatives!
    for (int b = 0; b < 256; b++) {
        uint8_t block[64];
        memset(block, ' ', sizeof(block));
        block[19] = (uint8_t)b;
        size_t pos = skip_non_tokens_avx2(block, sizeof(block), 0);
        if (TOKEN_KIND[b] != TOK_NONE) {
            assert(pos == 19);
        } else {
            assert(pos == sizeof(block));
        }

        memset(block, 'a', sizeof(block));
        block[19] = (uint8_t)b;
        size_t id_pos = skip_ident_cont_avx2(block, sizeof(block), 0);
        if (!IDENT_CONT[b]) {
            assert(id_pos == 19);
        } else {
            assert(id_pos == sizeof(block));
        }
    }
    printf("[PASS] test_keyword_table_and_mph (all 59 C89-C23 keywords + C++ non-keyword rejection + 256-byte AVX2 LUT verification)\n");
}

static void test_extensions(void) {
    // Valid C extensions (.c, .h, .C, .H)
    assert(is_c_extension((const uint8_t *)"main.c", 6));
    assert(is_c_extension((const uint8_t *)"header.h", 8));
    assert(is_c_extension((const uint8_t *)"UPPER.C", 7));
    assert(is_c_extension((const uint8_t *)"UPPER.H", 7));
    assert(is_c_extension((const uint8_t *)"a/b/c/foo.bar.c", 15));

    // Invalid / C++ extensions must be rejected
    assert(!is_c_extension((const uint8_t *)"main.cpp", 8));
    assert(!is_c_extension((const uint8_t *)"main.cc", 7));
    assert(!is_c_extension((const uint8_t *)"main.cxx", 8));
    assert(!is_c_extension((const uint8_t *)"main.c++", 8));
    assert(!is_c_extension((const uint8_t *)"header.hpp", 10));
    assert(!is_c_extension((const uint8_t *)"header.hxx", 10));
    assert(!is_c_extension((const uint8_t *)"header.h++", 10));
    assert(!is_c_extension((const uint8_t *)" header.hh", 10));
    assert(!is_c_extension((const uint8_t *)"script.py", 9));
    assert(!is_c_extension((const uint8_t *)"Makefile", 8));
    assert(!is_c_extension((const uint8_t *)"c", 1));
    assert(!is_c_extension((const uint8_t *)"", 0));
    printf("[PASS] test_extensions (strict .c/.h/.C/.H only, C++ rejected)\n");
}

static void test_comments_and_splicing(void) {
    const char *src =
        "// single line comment with if int return while\n"
        "int a = 1;\n"
        "// spliced line comment \\\n"
        "if (a) return 0;\n"
        "// multi-spliced line comment \\\r\n"
        "while (1) \\\n"
        "for (;;)\n"
        "return a;\n"
        "/* block comment with\n"
        "   struct union enum typedef static const\n"
        "   * / ** / ***/\n"
        "char b = 2;\n"
        "/* unclosed block comment at EOF: goto switch case default";

    ParseResult res;
    memset(&res, 0, sizeof(res));
    parse_into((const uint8_t *)src, strlen(src), &res);

    // Only `int`, `return`, and `char` should be counted!
    if (res.total_keywords != 3) {
        for (int i = 0; i < KEYWORD_COUNT; i++) {
            if (res.keyword_counts[i] > 0) {
                printf("Found kw %s: %" PRId64 "\n", KEYWORD_NAMES[i], res.keyword_counts[i]);
            }
        }
    }
    assert(res.total_keywords == 3);
    assert(res.keyword_counts[find_kw_id("int")] == 1);
    assert(res.keyword_counts[find_kw_id("return")] == 1);
    assert(res.keyword_counts[find_kw_id("char")] == 1);
    assert(res.keyword_counts[find_kw_id("if")] == 0);
    assert(res.keyword_counts[find_kw_id("while")] == 0);
    assert(res.keyword_counts[find_kw_id("for")] == 0);
    assert(res.keyword_counts[find_kw_id("goto")] == 0);
    assert(res.lines_processed == 12);
    printf("[PASS] test_comments_and_splicing (single-line, backslash-spliced, block, unclosed)\n");
}

static void test_literals_and_digit_separators(void) {
    const char *src =
        "const char *s1 = \"if while for return struct\";\n"
        "const char *s2 = \"escaped \\\"if return\\\" still in string\";\n"
        "const char *s3 = \"escaped backslash at end \\\\\"; if (1) {}\n"
        "char c1 = '\"'; if (1) {}\n"
        "char c2 = '\\''; while (0) {}\n"
        "char c3 = '\\\\'; for (;;) {}\n"
        "int x = 1'000'000; return x;\n"
        "unsigned y = 0xDEAD'BEEF'CAFEu; break;\n"
        "int z = 1'2'3; if (z) continue;\n";

    ParseResult res;
    memset(&res, 0, sizeof(res));
    parse_into((const uint8_t *)src, strlen(src), &res);

    assert(res.keyword_counts[find_kw_id("const")] == 3);
    assert(res.keyword_counts[find_kw_id("char")] == 6);
    assert(res.keyword_counts[find_kw_id("if")] == 3);
    assert(res.keyword_counts[find_kw_id("while")] == 1);
    assert(res.keyword_counts[find_kw_id("for")] == 1);
    assert(res.keyword_counts[find_kw_id("int")] == 2);
    assert(res.keyword_counts[find_kw_id("return")] == 1);
    assert(res.keyword_counts[find_kw_id("unsigned")] == 1);
    assert(res.keyword_counts[find_kw_id("break")] == 1);
    assert(res.keyword_counts[find_kw_id("continue")] == 1);
    assert(res.keyword_counts[find_kw_id("struct")] == 0);
    assert(res.total_keywords == 20);
    printf("[PASS] test_literals_and_digit_separators (strings, escapes, char literals, C23 digit separators)\n");
}

static void test_avx2_boundaries(void) {
    // Test buffers of various sizes around 32-byte AVX2 lane boundaries and > 255*32 (8160) SAD blocks
    size_t len = 32 * 300 + 17;
    uint8_t *buf = (uint8_t *)malloc(len);
    assert(buf != NULL);

    // Fill with spaces and newlines at known positions
    memset(buf, ' ', len);
    size_t expected_nl = 0;
    for (size_t i = 0; i < len; i += 7) {
        buf[i] = '\n';
        expected_nl++;
    }
    assert(count_newlines_avx2(buf, len) == expected_nl);

    // Place keywords across 32-byte boundaries
    memset(buf, ' ', len);
    const char *kw = "_Static_assert"; // 14 bytes
    size_t kw_len = strlen(kw);
    size_t placed = 0;
    for (size_t off = 25; off + kw_len < len; off += 64) {
        memcpy(buf + off, kw, kw_len);
        placed++;
    }
    ParseResult res;
    memset(&res, 0, sizeof(res));
    parse_into(buf, len, &res);
    assert(res.total_keywords == (int64_t)placed);
    assert(res.keyword_counts[find_kw_id("_Static_assert")] == (int64_t)placed);

    free(buf);
    printf("[PASS] test_avx2_boundaries (unaligned 32-byte crossings & multi-block vpsadbw)\n");
}

static void test_directory_traversal_and_edge_cases(void) {
    char tmp_dir[] = "/tmp/c_ultra_test_XXXXXX";
    assert(mkdtemp(tmp_dir) != NULL);

    // 1. Create a valid .c file and .H file
    char path_c[512], path_h[512], path_cpp[512], path_fifo[512], sub_dir[512], sub_c[512];
    snprintf(path_c, sizeof(path_c), "%s/valid.c", tmp_dir);
    snprintf(path_h, sizeof(path_h), "%s/UPPER.H", tmp_dir);
    snprintf(path_cpp, sizeof(path_cpp), "%s/ignored.cpp", tmp_dir);
    snprintf(path_fifo, sizeof(path_fifo), "%s/named_pipe.c", tmp_dir);
    snprintf(sub_dir, sizeof(sub_dir), "%s/subdir", tmp_dir);
    snprintf(sub_c, sizeof(sub_c), "%s/subdir/nested.c", tmp_dir);

    FILE *f = fopen(path_c, "wb");
    assert(f != NULL);
    fprintf(f, "int main(void) { if (true) return 0; }\n");
    fclose(f);

    f = fopen(path_h, "wb");
    assert(f != NULL);
    fprintf(f, "typedef struct { constexpr int x; } Foo;\n");
    fclose(f);

    // C++ file that must be ignored
    f = fopen(path_cpp, "wb");
    assert(f != NULL);
    fprintf(f, "int while for return if switch case default;\n");
    fclose(f);

    // Named FIFO named `named_pipe.c` - must be skipped without hanging!
    assert(mkfifo(path_fifo, 0600) == 0);

    // Subdirectory with nested.c (testing < 16 files partial batch flush on exit!)
    assert(mkdir(sub_dir, 0700) == 0);
    f = fopen(sub_c, "wb");
    assert(f != NULL);
    fprintf(f, "static inline _Bool check(void) { return false; }\n");
    fclose(f);

    // Run full multi-threaded WorkScheduler on tmp_dir
    WorkScheduler sched;
    sched_init(&sched, tmp_dir, strlen(tmp_dir));

    int num_threads = 4;
    pthread_t threads[3];
    WorkerContext ctxs[4];
    memset(ctxs, 0, sizeof(ctxs));
    for (int i = 0; i < num_threads; i++) {
        ctxs[i].sched = &sched;
    }
    for (int i = 0; i < num_threads - 1; i++) {
        assert(pthread_create(&threads[i], NULL, unified_worker_proc, &ctxs[i + 1]) == 0);
    }
    unified_worker_proc(&ctxs[0]);
    WorkerResult totals;
    memset(&totals, 0, sizeof(totals));
    worker_result_add(&totals, &ctxs[0].result);
    for (int i = 0; i < num_threads - 1; i++) {
        pthread_join(threads[i], NULL);
        worker_result_add(&totals, &ctxs[i + 1].result);
    }
    sched_deinit(&sched);

    // Verify results:
    // valid.c: int, void, if, true, return (5 keywords, 1 line)
    // UPPER.H: typedef, struct, constexpr, int (4 keywords, 1 line)
    // nested.c: static, inline, _Bool, void, return, false (6 keywords, 1 line)
    // ignored.cpp and named_pipe.c: ignored!
    assert(totals.files_found == 3);
    assert(totals.files_processed == 3);
    assert(totals.parse_result.lines_processed == 3);
    assert(totals.parse_result.total_keywords == 15);
    assert(totals.parse_result.keyword_counts[find_kw_id("int")] == 2);
    assert(totals.parse_result.keyword_counts[find_kw_id("void")] == 2);
    assert(totals.parse_result.keyword_counts[find_kw_id("return")] == 2);
    assert(totals.parse_result.keyword_counts[find_kw_id("true")] == 1);
    assert(totals.parse_result.keyword_counts[find_kw_id("false")] == 1);
    assert(totals.parse_result.keyword_counts[find_kw_id("constexpr")] == 1);

    // Cleanup
    unlink(sub_c);
    rmdir(sub_dir);
    unlink(path_fifo);
    unlink(path_cpp);
    unlink(path_h);
    unlink(path_c);
    rmdir(tmp_dir);
    printf("[PASS] test_directory_traversal_and_edge_cases (multi-thread, FIFO skip, .cpp skip, <16 partial batch flush)\n");
}

static void test_format_number(void) {
    char buf[32];
    assert(strcmp(format_number(0, buf), "0") == 0);
    assert(strcmp(format_number(999, buf), "999") == 0);
    assert(strcmp(format_number(1000, buf), "1,000") == 0);
    assert(strcmp(format_number(1234567, buf), "1,234,567") == 0);
    assert(strcmp(format_number(10000000000LL, buf), "10,000,000,000") == 0);
    assert(get_cpu_count() >= 1);
    (void)print_results;
    printf("[PASS] test_format_number (64-bit integer formatting with commas)\n");
}

int main(void) {
    printf("Running c_ultra test suite...\n");
    test_keyword_table_and_mph();
    test_extensions();
    test_comments_and_splicing();
    test_literals_and_digit_separators();
    test_avx2_boundaries();
    test_directory_traversal_and_edge_cases();
    test_format_number();
    printf("All tests passed successfully!\n");
    return 0;
}
