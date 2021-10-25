# This script runs examples under various options
# This is mainly used to gather coverage information

import os
import sys
import subprocess
import time

unit_tests_list = [
    {"name": "alloc-test", "entry": "main"},
    {"name": "annotation", "entry": "main"},
    {"name": "arith", "entry": "main"},
    {"name": "array", "entry": "main"},
    {"name": "assignment", "entry": "main"},
    {"name": "big-loop", "entry": "main"},
    {"name": "cast", "entry": "main"},
    {"name": "crate-bin-test", "entry": "main"},
    {"name": "crate-lib-test", "entry": "foo"},
    {"name": "empty", "entry": "main"},
    {"name": "enum-test", "entry": "main"},
    {"name": "function-call", "entry": "main"},
    {"name": "index", "entry": "main"},
    {"name": "iterator", "entry": "main"},
    {"name": "loop-test", "entry": "main"},
    {"name": "negation", "entry": "main"},
    {"name": "recursion", "entry": "main"},
    {"name": "size-of", "entry": "main"},
    {"name": "struct-test", "entry": "main"},
    {"name": "vector", "entry": "main"},
    {"name": "widen-narrow", "entry": "main"},
]

safe_bugs_list = [
    {"name": "division-by-zero", "entry": "main"},
    {"name": "incorrect-boundary-check", "entry": "main"},
    {"name": "incorrect-cast", "entry": "main"},
    {"name": "integer-overflow", "entry": "main"},
    {"name": "out-of-bound-index", "entry": "main"},
    {"name": "unreachable", "entry": "main"},
]

unsafe_bugs_list = [
    {"name": "double-free", "entry": "main"},
    {"name": "offset", "entry": "main"},
    {"name": "use-after-free(CVE-2019-15551)", "entry": "main"},
    {"name": "use-after-free(CVE-2019-16140)", "entry": "main"},
]

abstract_domains = ["interval", "octagon", "polyhedra", "linear_equalities", "ppl_polyhedra",
                    "ppl_linear_congruences", "pkgrid_polyhedra_linear_congruences"]

executable = os.path.abspath("../target/debug/cargo-mir-checker")


def run_test(test_list, test_dir, allow_error):
    # for (test, domain_type) in itertools.product(test_list, abstract_domains):
    for test in test_list:
        ok = False
        for domain_type in abstract_domains:
            # First run cargo clean to make sure that cargo does not use cache
            p = subprocess.Popen(["cargo", "clean"], cwd=os.path.join(test_dir, test["name"]))
            p.wait()

            my_env = os.environ.copy()
            # Disabling logging will save a lot of execution time!
            # my_env["RUST_LOG"] = "rust_mir_checker"

            # Customized options
            p = subprocess.Popen([executable, "mir-checker", "--", "--domain", domain_type, "--entry", test["entry"], "--widening_delay", "5",
                                 "--narrowing_iteration", "5", "--deny_warnings"], cwd=os.path.join(test_dir, test["name"]), env=my_env)
            p.communicate()[0]
            rc = p.returncode
            if rc == 0:
                ok = True

        if not ok and not allow_error:
            raise Exception("All abstract domains cannot reason about the verification conditions for \"{}\"".format(test["name"]))


# Run tests
try:
    start = time.time()
    run_test(unit_tests_list, "unit-tests", False)
    run_test(safe_bugs_list, "safe-bugs", True)
    run_test(unsafe_bugs_list, "unsafe-bugs", True)
    end = time.time()
    print("All tests are passed! Elapsed time:", end - start)
except Exception as e:
    print(e)
    sys.exit(1)  # This error code will cause a failure in CI service, so we know there are some problems
