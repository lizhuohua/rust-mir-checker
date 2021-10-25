import os
import sys
import signal
import itertools
import subprocess
import threading
from multiprocessing.pool import ThreadPool

class bcolors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'


# Only use the first three numerical abstract domain for now, to speed up the analysis
abstract_domains = ["ppl_linear_congruences"]
# "linear_equalities", "ppl_polyhedra",
# "ppl_linear_congruences", "pkgrid_polyhedra_linear_congruences"]

root_dir = os.path.dirname(os.path.abspath(__file__))  # path to the current script
executable = os.path.abspath(os.path.join(root_dir, "../../target/release/cargo-mir-checker"))  # path to the cargo sub-command
output_dir = os.path.join(root_dir, "output")  # path to the output directory
# paths to the all test cases
test_cases = [os.path.abspath(i) for i in os.listdir(root_dir)
              if os.path.isdir(os.path.join(root_dir, i)) and os.path.abspath(i) != output_dir]

# Lock for the global counters
lock = threading.Lock()
total_count = 0
success_count = 0
fail_count = 0
timeout_count = 0


def evaluate(task):
    crate_dir = task["crate_dir"]
    crate_name = os.path.basename(crate_dir)
    domain = task["domain"]
    entry = task["entry"]

    print("Evaluating", crate_name, "with domain type:", domain, "entry function:", entry)
    if lock.acquire():
        global total_count
        total_count += 1
        lock.release()

    build_dir = os.path.abspath(os.path.join(crate_name, entry + "_" + domain + "_build"))
    mkdir(build_dir)

    # Run `cargo clean` to make sure it does not use cache
    p = subprocess.Popen(["cargo", "clean", "--target-dir", build_dir], cwd=crate_dir)
    p.wait()
    # try:
    with subprocess.Popen([executable, "mir-checker", "--target-dir", build_dir, "--", "--entry_def_id_index", entry, "--domain", domain],
                          cwd=crate_dir, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, preexec_fn=os.setsid) as process:
        try:
            out = process.communicate(timeout=300)[0]

            if process.returncode == 0:
                print(bcolors.OKBLUE, "Finish analyzing crate", crate_name, "entry function:", entry, "domain type:", domain, bcolors.ENDC)
                output_file_path = os.path.join(crate_name, entry + "_" + domain)
                # The output file
                out_str = out.decode("utf-8")
                # Only write file if the output contains diagnosis from our tool
                if "[MirChecker]" in out_str:
                    f = open(output_file_path, "w")
                    f.write(out_str)
                    f.close()

                if lock.acquire():
                    global success_count
                    success_count += 1
                    lock.release()
            else:
                print(bcolors.FAIL, "Error while analyzing crate", crate_name, "entry function:", entry, "domain type:", domain, bcolors.ENDC)
                if lock.acquire():
                    global fail_count
                    fail_count += 1
                    lock.release()
        except subprocess.TimeoutExpired:
            print(bcolors.FAIL, "Timeout while analyzing crate", crate_name, "entry function:", entry, "domain type:", domain, bcolors.ENDC)
            os.killpg(process.pid, signal.SIGTERM)  # send signal to the process group
            if lock.acquire():
                global timeout_count
                timeout_count += 1
                lock.release()

    # Clean up
    print("Cleaning up", build_dir)
    # p = subprocess.Popen(["cargo", "clean", "--target-dir", build_dir], cwd=crate_dir)
    p = subprocess.Popen(["rm", "-rf", build_dir], cwd=crate_dir)
    p.wait()

# Create a directory (if it does not exist) in the current directory
def mkdir(dir_name):
    if not os.path.exists(dir_name):
        os.makedirs(dir_name)


# Lock for the global task list
task_list_lock = threading.Lock()
task_list = []

def get_task_list(crate_dir):
    if not os.path.exists(os.path.join(crate_dir, "Cargo.toml")):
        return None
    crate_name = os.path.basename(crate_dir)
    # First run `cargo clean` to make sure it does not use cache
    p = subprocess.Popen(["cargo", "clean"], cwd=crate_dir)
    p.wait()

    # Get a list of entry functions
    p = subprocess.Popen([executable, "mir-checker", "--", "--show_entries_index"],
                         cwd=crate_dir, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    entry_functions, _ = p.communicate()
    entry_functions = list(map(lambda x: str(x, "utf-8"), entry_functions.split()))

    if len(entry_functions) == 0:
        # The crate being analyzed has no usable entry functions, just ignore
        print(bcolors.WARNING, crate_name, "has no usable entry points, ignored", bcolors.ENDC)
        return None
    else:
        result = []

        for (entry, domain) in itertools.product(entry_functions, abstract_domains):
            task = {}
            task["crate_dir"] = crate_dir
            task["entry"] = entry
            task["domain"] = domain
            result.append(task)

        if task_list_lock.acquire():
            global task_list
            if result is not None:
                task_list += result
                mkdir(crate_name)
            task_list_lock.release()


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Need an argument to specify the size of the thread pool")
        exit(1)

    num_thread = int(sys.argv[1])

    mkdir(output_dir)
    os.chdir(output_dir)

    with ThreadPool(num_thread) as p:
        p.map(get_task_list, test_cases)

    print(len(task_list), "tasks in total")

    with ThreadPool(num_thread) as p:
        p.map(evaluate, task_list)

    print("Done with success:", success_count, ", fail:", fail_count, ", timeout:", timeout_count, ", total:", total_count)
