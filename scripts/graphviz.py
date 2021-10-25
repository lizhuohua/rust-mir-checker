# Run graphviz more easily
import sys
import subprocess

dotfile = "main.dot"
pngfile = "main.png"

if len(sys.argv) == 1:
    subprocess.check_call("cargo clean", shell=True)
    subprocess.check_call("cargo rustc -- -C panic=abort -Zunpretty=mir-cfg -Zalways_encode_mir > " + dotfile, shell=True)
else:
    inputfile = sys.argv[1]
    subprocess.check_call("rustc -- -C panic=abort -Zunpretty=mir-cfg -Zalways_encode_mir ", shell=True)

subprocess.check_call("dot " + dotfile + " -T png -o " + pngfile, shell=True)

if sys.platform == "linux":
    subprocess.check_call("feh " + pngfile + "&", shell=True)
elif sys.platform == "darwin":
    subprocess.check_call("open " + pngfile + "&", shell=True)

subprocess.check_call("rm " + dotfile, shell=True)
