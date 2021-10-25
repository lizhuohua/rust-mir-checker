import os

# Prepare for (absolute) paths
root_dir = os.path.dirname(os.path.abspath(__file__))  # path to the current script
output_dir = os.path.join(root_dir, "output")  # path to the output directory
cases_dir = [os.path.join(output_dir, i) for i in os.listdir(output_dir)]  # paths to the all test cases


def count_warning(case_dir):
    warning_set = set()
    output_files = [os.path.join(case_dir, i) for i in os.listdir(case_dir)]
    for output in output_files:
        with open(output) as f:
            lines = [line.rstrip() for line in f]
            for i in range(len(lines)):
                if '[MirChecker]' in lines[i]:
                    warning_set.add(lines[i]+lines[i+1])
    print(len(warning_set))


if __name__ == "__main__":
    # For precision, run each evaluation sequentially instead of in parallel
    for case_dir in cases_dir:
        print(case_dir)
        count_warning(case_dir)
