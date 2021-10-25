import os
import sys
import signal
import itertools
import subprocess
import threading
import csv
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


class EvaluationResult:
    def __init__(self, name, domain, entry, elasptime, peakmem):
        self.name = name
        self.domain = domain
        self.entry = entry
        self.elasptime = elasptime
        self.peakmem = peakmem

    def __str__(self):
        return f'name: {self.name}, domain: {self.domain}, entry: {self.entry}, elasptime: {self.elasptime}, peakmem: {self.peakmem}'


# Only use the first three numerical abstract domain for now, to speed up the analysis
abstract_domains = ["interval", "octagon", "polyhedra", "ppl_linear_congruences"]
# "linear_equalities", "ppl_polyhedra",
# "ppl_linear_congruences", "pkgrid_polyhedra_linear_congruences"]

root_dir = os.path.dirname(os.path.abspath(__file__))  # path to the current script
crate_dir = os.path.join(root_dir, "crates")  # path to the crate directory
output_dir = os.path.join(root_dir, "outputs")  # path to the output directory
test_cases_dir = [os.path.join(crate_dir, i) for i in os.listdir(crate_dir)]  # paths to the all test cases
executable = os.path.abspath(os.path.join(root_dir, "../../target/release/cargo-mir-checker"))  # path to the cargo sub-command

# Lock for the result
lock = threading.Lock()
result = []

lock2 = threading.Lock()
count = 0
total_count = 0
failed_cases = set()

def evaluate(task):
    crate_dir = task["crate_dir"]
    crate_name = os.path.basename(crate_dir)
    domain = task["domain"]
    entry = task["entry"]
    global count
    global result
    global cleaning_delay
    print("Evaluating", crate_name, "with domain type:", domain, "entry function:", entry, "cleaning delay:", cleaning_delay)

    build_dir = os.path.abspath(os.path.join(crate_name, entry + "_" + domain + "_build"))
    mkdir(build_dir)

    # Run `cargo clean` to make sure it does not use cache
    subprocess.Popen(["cargo", "clean", "--target-dir", build_dir], cwd=crate_dir).wait()

    timeout_sec = 60

    # Use `time` command to get execution time and peak memory usage
    with subprocess.Popen(["/usr/bin/time", "-f", "%M\n%e", executable, "mir-checker", "--quiet",
                           "--target-dir", build_dir, "--", "--entry", entry, "--domain", domain, "--cleaning_delay",
                           str(cleaning_delay)], cwd=crate_dir, stderr=subprocess.PIPE, preexec_fn=os.setsid) as process:
        try:
            out = process.communicate(timeout=timeout_sec)[1]

            if process.returncode == 0:
                out_str = out.decode("utf-8").split()
                elasp_time = float(out_str[-1])
                peak_mem = int(out_str[-2])

                if lock.acquire():
                    result.append(EvaluationResult(crate_name, domain, entry, elasp_time, peak_mem))
                    lock.release()

                if lock2.acquire():
                    count += 1
                    print("Progress:", count, "/", total_count)
                    lock2.release()

                print(bcolors.OKBLUE, "Finish analyzing crate", crate_name, "entry function:", entry,
                      "domain type:", domain, "peak memory:", peak_mem, "elasp time:", elasp_time, bcolors.ENDC)

            else:
                print(bcolors.FAIL, "Error while analyzing crate", crate_name, "entry function:", entry, "domain type:", domain, bcolors.ENDC)
                if lock2.acquire():
                    count += 1
                    print("Progress:", count, "/", total_count)
                    failed_cases.add((crate_name, domain, entry))
                    lock2.release()

        except subprocess.TimeoutExpired:
            print(bcolors.FAIL, "Timeout while analyzing crate", crate_name, "entry function:", entry, "domain type:", domain, bcolors.ENDC)

            if lock.acquire():
                result.append(EvaluationResult(crate_name, domain, entry, timeout_sec, 0))
                lock.release()

            if lock2.acquire():
                count += 1
                print("Progress:", count, "/", total_count)
                lock2.release()
            os.killpg(process.pid, signal.SIGTERM)  # send signal to the process group

    # Clean up
    print("Cleaning up", build_dir)
    subprocess.Popen(["cargo", "clean", "--target-dir", build_dir], cwd=crate_dir).wait()
    # p = subprocess.Popen(["rm", "-rf", build_dir], cwd=crate_dir)


# Create a directory (if it does not exist) in the current directory
def mkdir(dir_name):
    if not os.path.exists(dir_name):
        os.makedirs(dir_name)


# Lock for the global task list
task_list_lock = threading.Lock()
task_list = []

def get_task_list(crate_dir):
    crate_name = os.path.basename(crate_dir)
    # First run `cargo clean` to make sure it does not use cache
    subprocess.Popen(["cargo", "clean"], cwd=crate_dir).wait()

    # Get a list of entry functions
    p = subprocess.Popen([executable, "mir-checker", "--", "--show_entries"],
                         cwd=crate_dir, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    entry_functions, _ = p.communicate()
    entry_functions = list(map(lambda x: str(x, "utf-8"), entry_functions.split()))

    result = []

    for (entry, domain) in itertools.product(entry_functions, abstract_domains):
        task = {}
        task["crate_dir"] = crate_dir
        task["entry"] = entry
        task["domain"] = domain
        result.append(task)

    if task_list_lock.acquire():
        global task_list
        task_list += result
        mkdir(crate_name)
        task_list_lock.release()


def process_result(result):
    output = {}
    for r in result:
        if (r.name, r.domain, r.entry) not in failed_cases:
            if r.name not in output:
                output[r.name] = {}
                for domain in abstract_domains:
                    output[r.name][domain] = (0, 0)

    for r in result:
        if (r.name, r.domain, r.entry) not in failed_cases:
            (time, mem) = output[r.name][r.domain]
            output[r.name][r.domain] = (time+r.elasptime, max(mem, r.peakmem))

    return output


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Need an argument to specify the size of the thread pool")
        exit(1)

    num_thread = int(sys.argv[1])

    mkdir(output_dir)
    os.chdir(output_dir)

    with ThreadPool(num_thread) as p:
        p.map(get_task_list, test_cases_dir)

    total_count = len(task_list)*2

    print(len(task_list)*2, "tasks in total, run in", num_thread, "threads")

    # 1st run
    cleaning_delay = 0
    with ThreadPool(num_thread) as p:
        p.map(evaluate, task_list)

    with open('eval_result_nocleanup.csv', 'w', newline='') as csvfile:
        spamwriter = csv.writer(csvfile)
        spamwriter.writerow([''] + abstract_domains + abstract_domains)
        result_dict = process_result(result)
        for (name, domain_dict) in result_dict.items():
            res = []
            for domain in abstract_domains:
                (t, m) = domain_dict[domain]
                res.append(str(t))
            for domain in abstract_domains:
                (t, m) = domain_dict[domain]
                res.append(str(m))
            spamwriter.writerow([name] + res)

    result = []

    # 2nd run
    cleaning_delay = 1
    with ThreadPool(num_thread) as p:
        p.map(evaluate, task_list)

    with open('eval_result.csv', 'w', newline='') as csvfile:
        spamwriter = csv.writer(csvfile)
        spamwriter.writerow([''] + abstract_domains + abstract_domains)
        result_dict = process_result(result)
        for (name, domain_dict) in result_dict.items():
            res = []
            for domain in abstract_domains:
                (t, m) = domain_dict[domain]
                res.append(str(t))
            for domain in abstract_domains:
                (t, m) = domain_dict[domain]
                res.append(str(m))
            spamwriter.writerow([name] + res)

    print(failed_cases)
