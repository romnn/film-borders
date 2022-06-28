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

WASM_MODULE = "filmborders"
ROOT_DIR = Path(__file__).parent
WWW_DIR = ROOT_DIR / "www"
WASM_NODE_MODULE = WWW_DIR / "node_modules" / WASM_MODULE
WWW_PUBLIC_DIR = WWW_DIR / "public"
WWW_PUBLIC_WASM_DIR = WWW_PUBLIC_DIR / "wasm"
SOURCE_DIR = ROOT_DIR / "src"


@task(help={"check": "Checks if source is formatted without applying changes"})
def format(c, check=False):
    """Format code"""
    pass


@task
def pack(c):
    """Compile, pack and upgrade the wasm module package"""
    # node module first
    c.run("rm -rf {}".format(WASM_NODE_MODULE))
    c.run(
        "wasm-pack build --target web --release {} -- --features wasm ".format(
            ROOT_DIR
        ),
        pty=True,
    )
    c.run("yarn --cwd {} upgrade {}".format(WWW_DIR, WASM_MODULE), pty=True)

    # public wasm second
    c.run(
        "wasm-pack build --target no-modules --release {} -- --features wasm".format(
            ROOT_DIR
        ),
        pty=True,
    )
    c.run("mkdir -p {}".format(WWW_PUBLIC_WASM_DIR))
    c.run("rm -rf {}".format(WWW_PUBLIC_WASM_DIR))
    c.run("cp -R {} {}".format("pkg", WWW_PUBLIC_WASM_DIR))


@task
def install_wasm_pack(c):
    """Download and install wasm-pack"""
    c.run("cargo install wasm-pack --force", pty=True)


@task
def lint(c):
    """Lint code"""
    pass


@task
def clean_build(c):
    """Clean up files from package building"""
    c.run("rm -fr www/build/")
    c.run("rm -fr target/")
    c.run("rm -fr pkg/")


@task(pre=[clean_build])
def clean(c):
    """Runs all clean sub-tasks"""
    pass
