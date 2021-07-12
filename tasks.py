"""
Tasks for maintaining the project.
Execute 'invoke --list' for guidance on using Invoke
"""
import shutil
import pprint

from invoke import task
import webbrowser
from pathlib import Path

Path().expanduser()

WASM_MODULE = "wasm-mod"
ROOT_DIR = Path(__file__).parent
WWW_DIR = ROOT_DIR / "www"
WWW_PUBLIC_DIR = WWW_DIR / "public"
WWW_PUBLIC_WASM_DIR = WWW_PUBLIC_DIR / "wasm"
SOURCE_DIR = ROOT_DIR / "src"


def _delete_file(file):
    try:
        file.unlink(missing_ok=True)
    except TypeError:
        # missing_ok argument added in 3.8
        try:
            file.unlink()
        except FileNotFoundError:
            pass


@task(help={"check": "Checks if source is formatted without applying changes"})
def format(c, check=False):
    """Format code"""
    python_dirs_string = " ".join(PYTHON_DIRS)
    black_options = "--diff" if check else ""
    c.run("pipenv run black {} {}".format(black_options, python_dirs_string))
    isort_options = "--check-only" if check else ""
    c.run("pipenv run isort {} {}".format(isort_options, python_dirs_string))


@task
def pack(c):
    """Compile, pack and upgrade the wasm module package"""
    c.run("wasm-pack build --release {}".format(ROOT_DIR), pty=True)
    c.run("yarn --cwd {} upgrade {}".format(WWW_DIR, WASM_MODULE), pty=True)
    c.run("wasm-pack build --target no-modules --release {}".format(ROOT_DIR), pty=True)
    c.run("mkdir -p {}".format(WWW_PUBLIC_WASM_DIR))
    c.run("rm -rf {}".format(WWW_PUBLIC_WASM_DIR))
    c.run("cp -R {} {}".format("pkg", WWW_PUBLIC_WASM_DIR))


@task
def lint(c):
    """Lint code"""
    c.run("pipenv run flake8 {}".format(SOURCE_DIR))


@task
def test(c, min_coverage=None):
    """Run tests"""
    pytest_options = "--cov-fail-under={}".format(min_coverage) if min_coverage else ""
    c.run("pipenv run pytest --cov={} {}".format(SOURCE_DIR, pytest_options))


@task
def install_hooks(c):
    """Install pre-commit hooks"""
    c.run("pipenv run pre-commit install -t pre-commit")
    c.run("pipenv run pre-commit install -t pre-push")


@task
def pre_commit(c):
    """Run all pre-commit checks"""
    c.run("pipenv run pre-commit run --all-files")


@task
def clean_build(c):
    """Clean up files from package building"""
    c.run("rm -fr build/")
    c.run("rm -fr dist/")
    c.run("rm -fr .eggs/")
    c.run("find . -name '*.egg-info' -exec rm -fr {} +")
    c.run("find . -name '*.egg' -exec rm -f {} +")


@task(pre=[clean_build])
def clean(c):
    """Runs all clean sub-tasks"""
    pass


@task(pre=[clean])
def release(c):
    """Make a release of the python package to pypi"""
    c.run("twine upload dist/*")
