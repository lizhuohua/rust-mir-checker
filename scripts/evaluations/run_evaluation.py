import os
import signal
import subprocess

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


abstract_domains = ["interval",]
# "octagon", "polyhedra", "linear_equalities", "ppl_polyhedra",
#                   "ppl_linear_congruences", "pkgrid_polyhedra_linear_congruences"]

# Prepare for (absolute) paths
root_dir = os.path.dirname(os.path.abspath(__file__))  # path to the current script
executable = os.path.abspath(os.path.join(root_dir, "../../target/release/cargo-mir-checker"))  # path to the cargo sub-command
crate_dir = os.path.join(root_dir, "crates")  # path to the crate directory
output_dir = os.path.join(root_dir, "outputs")  # path to the output directory
test_cases_dir = [os.path.join(crate_dir, i) for i in os.listdir(crate_dir)]  # paths to the all test cases


class EvaluationResult:
    def __init__(self, name, nfunc, elasptime, peakmem):
        self.name = name
        self.nfunc = nfunc
        self.elasptime = elasptime
        self.peakmem = peakmem

def run_task(task):
    crate_dir = task["crate_dir"]
    crate_name = os.path.basename(crate_dir)
    domain = task["domain"]
    entry = task["entry"]

    print("Evaluating", crate_name, "with domain type:", domain, "entry function:", entry)

    build_dir = os.path.abspath(os.path.join(crate_name, entry + "_" + domain + "_build"))
    mkdir(build_dir)

    # Run `cargo clean` to make sure it does not use cache
    subprocess.Popen(["cargo", "clean", "--target-dir", build_dir], cwd=crate_dir).wait()
    # try:
    success = False
    with subprocess.Popen(["/usr/bin/time", "-f", "%M\n%e", executable, "mir-checker", "--quiet",
                           "--target-dir", build_dir, "--", "--entry", entry, "--domain", domain],
                          cwd=crate_dir, stderr=subprocess.PIPE, preexec_fn=os.setsid) as process:
        try:
            out = process.communicate(timeout=300)[1]

            if process.returncode == 0:
                out_str = out.decode("utf-8").split()
                elasp_time = float(out_str[-1])
                peak_mem = int(out_str[-2])

                success = True
                print(bcolors.OKBLUE, "Finish analyzing crate", crate_name, "entry function:", entry,
                      "domain type:", domain, "peak memory:", peak_mem, "elasp time:", elasp_time, bcolors.ENDC)

            else:
                print(bcolors.FAIL, "Error while analyzing crate", crate_name, "entry function:", entry, "domain type:", domain, bcolors.ENDC)
        except subprocess.TimeoutExpired:
            print(bcolors.FAIL, "Timeout while analyzing crate", crate_name, "entry function:", entry, "domain type:", domain, bcolors.ENDC)
            os.killpg(process.pid, signal.SIGTERM)  # send signal to the process group

    # Clean up
    print("Cleaning up", build_dir)
    subprocess.Popen(["cargo", "clean", "--target-dir", build_dir], cwd=crate_dir).wait()
    # subprocess.Popen(["rm", "-rf", build_dir], cwd=crate_dir).wait()

    if success:
        return (elasp_time, peak_mem)
    else:
        return None


def evaluate(crate_dir):
    entry_list = get_entry_list(crate_dir)
    num_entry = len(entry_list)
    # task_list = get_task_list(crate_dir)
    crate_name = os.path.basename(crate_dir)
    mkdir(crate_name)
    print("Evaluating", crate_name, ", # of functions:", num_entry)

    elasptime = 0
    peakmem = 0
    for entry in entry_list:
        time_of_this_entry = 0
        task_list = get_task_list(entry, crate_dir)
        num_of_success = 0
        for task in task_list:
            result = run_task(task)
            if result is not None:
                time_of_this_entry += result[0]
                peakmem = max(peakmem, result[1])
                num_of_success += 1
        if num_of_success != 0:
            time_of_this_entry /= num_of_success

        elasptime += time_of_this_entry

    return EvaluationResult(crate_name, num_entry, elasptime, peakmem)


# Create a directory (if it does not exist) in the current directory
def mkdir(dir_name):
    if not os.path.exists(dir_name):
        os.makedirs(dir_name)


def get_entry_list(crate_dir):
    # First run `cargo clean` to make sure it does not use cache
    subprocess.Popen(["cargo", "clean"], cwd=crate_dir).wait()

    # Get a list of entry functions
    p = subprocess.Popen([executable, "mir-checker", "--", "--show_entries"],
                         cwd=crate_dir, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    entry_functions, _ = p.communicate()
    entry_functions = list(map(lambda x: str(x, "utf-8"), entry_functions.split()))

    return entry_functions


def get_task_list(entry_function, crate_dir):
    task_list = []
    for domain in abstract_domains:
        task = {}
        task["crate_dir"] = crate_dir
        task["entry"] = entry_function
        task["domain"] = domain
        task_list.append(task)

    return task_list


def show_result(eval_result):
    for result in eval_result:
        print(result.name, result.nfunc, result.elasptime, result.peakmem)


if __name__ == "__main__":
    mkdir(output_dir)
    os.chdir(output_dir)

    eval_result = []
    # For precision, run each evaluation sequentially instead of in parallel
    for case_dir in test_cases_dir:
        result = evaluate(case_dir)
        eval_result.append(result)

    show_result(eval_result)
