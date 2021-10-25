import requests
import os
from subprocess import Popen, DEVNULL

def make_crate_list(category, page_list):
    category_str = "" if category == "" else "category=" + category
    crate_list = []
    for page in page_list:
        request_page = requests.get('https://crates.io/api/v1/crates?{}&page={}&per_page=100&sort=downloads'.format(category_str, page))
        crate_list += request_page.json()['crates']
    return crate_list


repo_set = set()
def clone_repo(name, repo):
    # crates.io API may not always correctly return the repository address
    if repo is None:
        print("Warning:", name, "is ignored because its repository address is none")
        return False

    global repo_set
    if repo in repo_set:
        # Do not clone repositories that have already been cloned
        print("Warning:", name, "is ignored because it has already been cloned from", repo)
        return False
    else:
        repo_set.add(repo)
        print("Cloning repo: ", name, "from:", repo)
        my_env = os.environ.copy()
        my_env["GIT_TERMINAL_PROMPT"] = "0"  # Some repositories need username and password, use this to fail instead of prompting for credentials
        p = Popen(["git", "clone", "--depth=1", repo, name], stdout=DEVNULL, stderr=DEVNULL, env=my_env)
        p.communicate()[0]
        if p.returncode != 0:
            print("Warning: Error whiling cloning repo:", repo)
            return False
        return True

def should_ignore(name, description):
    description = description.lower()
    # Exclude crates that are related to FFI, macro/trait definitions, multi-threads, etc.
    keywords = ["ffi", "macro", "binding", "wrapper", "float", "api", "abi", "trait", "concurrent",
                "async", "pin", "mutex", "lock", "atomic", "thread", "string", "rational", "libm",
                "cortex", "hal", "simd", "asm", "sys", "stm32", "arch", "gpio"]
    if any([keyword in name + description for keyword in keywords]):
        print("Warning:", name, "is ignored because it is not our concern")
        return True
    return False


count = 0
print("Requesting the API of crates.io...")
crate_list = make_crate_list("no-std", [11, 12, 13, 14, 15, 16, 17, 18, 19, 20])
print("Got addresses of {} crates, start cloning...".format(len(crate_list)))
for crate in crate_list:
    name = crate['name']
    description = crate['description']
    repo = crate['repository']

    if not should_ignore(name, description):
        if clone_repo(name, repo):
            count += 1

print(count, "crates cloned")
