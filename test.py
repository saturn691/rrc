#!/usr/bin/env python3

"""
A wrapper script to run the compiler test suite.
"""

__version__ = "0.1.0"
__author__ = "William Huynh (@saturn691)"

from pathlib import Path
from dataclasses import dataclass
from typing import List, Optional
import os
import json
import shutil
import subprocess
import sys
import argparse
import time


RED = "\033[31m"
GREEN = "\033[32m"
RESET = "\033[0m"

# Do not show colours when not in a TTY
if not sys.stdout.isatty():
    RED = ""
    GREEN = ""
    RESET = ""

PROJECT_LOCATION = Path(__file__).resolve().parent
OUTPUT_FOLDER = PROJECT_LOCATION.joinpath("bin").resolve()
COMPILER_TEST_FOLDER = PROJECT_LOCATION.joinpath("tests").resolve()
COMPILER_FILE = PROJECT_LOCATION.joinpath("target/debug/rrc").resolve()
RESULTS_FILE = COMPILER_TEST_FOLDER.joinpath("expected.json").resolve()
TEST_DRIVER = COMPILER_TEST_FOLDER.joinpath("test.rs").resolve()

BUILD_TIMEOUT_SECONDS = 60
RUN_TIMEOUT_SECONDS = 15
TIMEOUT_RETURNCODE = 124


@dataclass
class Result:
    test_case_name: str
    return_code: int
    passed: bool
    timeout: bool
    error_log: Optional[str]

    def to_log(self) -> str:
        timeout = "[TIMED OUT] " if self.timeout else ""
        if self.passed:
            return f'{self.test_case_name}\n\t> {GREEN}Pass{RESET}\n'

        return f'{self.test_case_name}\n{RED}{timeout + self.error_log}{RESET}\n'


def run_test(test_file: Path) -> Result:
    """
    Runs an instance of a test case.
    """
    test_name = test_file.relative_to(PROJECT_LOCATION)

    # Relative path w.r.t. COMPILER_TEST_FOLDER
    relative_path = test_file.relative_to(COMPILER_TEST_FOLDER)
    log_path = Path(OUTPUT_FOLDER).joinpath(
        relative_path.parent, test_file.stem, test_file.stem
    ).resolve()

    def relevant_files(component):
        return f"{log_path}.{component}.stderr.log \n\t {log_path}.{component}.stdout.log"

    # Recreate the directory
    shutil.rmtree(log_path.parent, ignore_errors=True)
    log_path.parent.mkdir(parents=True, exist_ok=True)

    custom_env = os.environ.copy()
    custom_env["ASAN_OPTIONS"] = "exitcode=0"

    # Compile
    return_code, _, timed_out = run_subprocess(
        cmd=[COMPILER_FILE, "--input", test_file, "--output", f"{log_path}.ll"],
        timeout=RUN_TIMEOUT_SECONDS,
        log_path=f"{log_path}.compiler",
        env=custom_env,
    )
    compiler_log_file_str = f"{relevant_files('compiler')}"
    if return_code != 0:
        msg = f"\t> Failed to compile testcase: \n\t {compiler_log_file_str}"
        return Result(
            test_case_name=test_name, return_code=return_code, passed=False,
            timeout=timed_out, error_log=msg)

    # Generate assembly
    return_code, _, timed_out = run_subprocess(
        cmd=[
            "llc", f"{log_path}.ll"
        ],
        timeout=RUN_TIMEOUT_SECONDS,
        log_path=f"{log_path}.llvm",
    )
    if return_code != 0:
        msg = f"\t> Failed to LLVM: \n\t {compiler_log_file_str} \n\t {relevant_files('llvm')}"
        return Result(
            test_case_name=test_name, return_code=return_code, passed=False,
            timeout=timed_out, error_log=msg)

    # Intermediate steps to get turn into `.a` file
    subprocess.run(
        ["llvm-as", f"{log_path}.ll", "-o", f"{log_path}.bc"], check=True)
    subprocess.run(
        ["llc", f"{log_path}.bc", "-filetype=obj", "-o", f"{log_path}.o"],
        check=True)
    subprocess.run(["ar", "rcs", log_path.parent.joinpath(
        "libfoo.a").resolve(), f"{log_path}.o"], check=True)

    # Link
    subprocess.run(["rustc", TEST_DRIVER, "-L", log_path.parent,
                   "-o", f"{log_path}.out"], check=True)

    # Run
    expected_result = 0
    with open(RESULTS_FILE, "r") as f:
        results = json.load(f)
        key = str(test_file.relative_to(COMPILER_TEST_FOLDER))
        expected_result = results[key]

    return_code, _, timed_out = run_subprocess(
        cmd=[f"{log_path}.out"],
        timeout=RUN_TIMEOUT_SECONDS,
        log_path=f"{log_path}.sim",
    )
    if return_code != expected_result:
        msg = (
            f"\t> Failed to run (return code: {return_code}, "
            f"expected: {expected_result}): \n\t {compiler_log_file_str} "
            f"\n\t {relevant_files('sim')}"
        )
        return Result(
            test_case_name=test_name, return_code=return_code, passed=False,
            timeout=timed_out, error_log=msg)

    return Result(
        test_case_name=test_name, return_code=return_code, passed=True,
        timeout=False, error_log=""
    )


def run_subprocess(
    cmd: List[str],
    timeout: int,
    env: Optional[dict] = None,
    log_path: Optional[str] = None,
    silent: bool = False,
) -> tuple[int, str, bool]:
    """
    Wrapper for subprocess.run(...) with common arguments and error handling.

    Returns tuple of (return_code: int, error_message: str, timed_out: bool)
    """
    # None means that stdout and stderr are handled by parent
    # i.e., they go to console by default
    stdout = None
    stderr = None

    if silent:
        stdout = subprocess.DEVNULL
        stderr = subprocess.DEVNULL
    elif log_path:
        stdout = open(f"{log_path}.stdout.log", "w")
        stderr = open(f"{log_path}.stderr.log", "w")

    try:
        subprocess.run(cmd, env=env, stdout=stdout,
                       stderr=stderr, timeout=timeout, check=True)
    except subprocess.CalledProcessError as e:
        return e.returncode, f"{e.cmd} failed with return code {e.returncode}", False
    except subprocess.TimeoutExpired as e:
        return TIMEOUT_RETURNCODE, f"{e.cmd} took more than {e.timeout}", True

    return 0, "", False


def build(silent: bool) -> bool:
    """
    Wrapper for `cargo build`
    """
    print(GREEN + "Building compiler..." + RESET)

    return_code, error_message, _ = run_subprocess(
        cmd=["cargo", "build"],
        timeout=BUILD_TIMEOUT_SECONDS,
        silent=silent,
    )

    if return_code != 0:
        print(RED + "Error when making:" + error_message + RESET)
        return False

    return True


def run_tests(args):
    """
    Runs tests and prints the results.
    """
    tests = list(Path(args.dir).resolve().rglob("*.rs"))
    tests = sorted(tests, key=lambda x: (x.parent.name, x.name))

    results = []

    for test in tests:
        if test.name == "test.rs":
            continue

        result = run_test(test)
        results.append(result.passed)
        if not args.short or not result.passed:
            print(result.to_log())

    passing = sum(results)
    total = len(results)

    print(
        "\n>> Test Summary: " + GREEN + f"{passing} Passed, " + RED +
        f"{total-passing} Failed" + RESET
    )


def test(args):
    """
    Handles the test subcommand.
    """
    shutil.rmtree(OUTPUT_FOLDER, ignore_errors=True)
    Path(OUTPUT_FOLDER).mkdir(parents=True, exist_ok=True)

    if not build(args.short):
        return

    run_tests(args)


def update(args):
    """
    Handles the update subcommand.
    """
    # Get the result of the test
    tests = list(Path(args.dir).resolve().rglob("*.rs"))
    tests = sorted(tests, key=lambda x: (x.parent.name, x.name))

    # Load the pre-existing data
    if os.path.exists(RESULTS_FILE):
        with open(RESULTS_FILE, "r") as f:
            results = json.load(f)
    else:
        results = {}

    start_time = time.time()
    for test in tests:
        if test.name == "test.rs":
            continue

        print(f"Checking {test.relative_to(PROJECT_LOCATION)}")

        # Create a static library
        subprocess.run(["rustc", "--crate-type=staticlib",
                       "--crate-name=foo", test], timeout=BUILD_TIMEOUT_SECONDS,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # Create the executable
        subprocess.run(
            ["rustc", TEST_DRIVER, "-L", "."],
            BUILD_TIMEOUT_SECONDS
        )

        # Run the executable
        return_code, _, _ = run_subprocess(
            ["./test"],
            RUN_TIMEOUT_SECONDS
        )
        results[str(test.relative_to(COMPILER_TEST_FOLDER))] = return_code

    # Write the results to the file, overwriting the old one
    with open(RESULTS_FILE, "w") as f:
        json.dump(results, f, indent=4, sort_keys=True)

    print(f"{GREEN}Updated results in {time.time() - start_time:.2f} seconds{RESET}")


def parse_args() -> argparse.Namespace:
    """
    Wrapper for argument parsing.
    """
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--version",
        action="version",
        version=f"BetterTesting {__version__}"
    )

    subparsers = parser.add_subparsers(required=True)

    test_subparser = subparsers.add_parser(
        'test', help='Run the compiler tests.')
    test_subparser.set_defaults(func=test)
    test_subparser.add_argument(
        "dir", nargs="?", default=COMPILER_TEST_FOLDER, type=Path,
        help="(Optional) paths to the compiler test folders. Use this to select "
        "certain tests. Leave blank to run all tests.")
    test_subparser.add_argument(
        "-m", "--multithreading",
        action="store_true",
        default=False,
        help="Use multiple threads to run tests. This will make it faster, "
        "but order is not guaranteed. Should only be used for speed."
    )
    test_subparser.add_argument(
        "-s", "--short", action="store_true", default=False,
        help="Disable verbose output into the terminal. Note that all logs will "
        "be stored automatically into log files regardless of this option.")

    update_subparser = subparsers.add_parser(
        'update', help='Update the expected results file.')
    update_subparser.add_argument(
        "dir", nargs="?", default=COMPILER_TEST_FOLDER, type=Path,
        help="(Optional) paths to the compiler test folders. Use this to select "
        "certain tests. Leave blank to run all tests.")
    update_subparser.set_defaults(func=update)

    return parser.parse_args()


def main():
    args = parse_args()
    args.func(args)

    # Cleanup
    try:
        os.remove(PROJECT_LOCATION / "test")
    except FileNotFoundError:
        pass

    try:
        os.remove(PROJECT_LOCATION / "libfoo.a")
    except FileNotFoundError:
        pass


if __name__ == "__main__":
    try:
        main()
    finally:
        print(RESET, end="")
        if sys.stdout.isatty():
            os.system("stty echo")
