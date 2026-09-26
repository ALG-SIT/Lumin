#!/usr/bin/env python3
"""Stage pinned official ORT/WebGPU native libraries, without a Python runtime dependency.

Only the build uses Python's standard library. Wheels are verified archives;
only native libraries and licenses are copied, never Python modules or executables.
"""
import argparse
import hashlib
import json
import platform
from pathlib import Path
import shutil
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "scripts/onnx-runtime-lock.json"


def digest(path):
    with path.open("rb") as source:
        value = hashlib.sha256()
        for block in iter(lambda: source.read(1024 * 1024), b""):
            value.update(block)
        return value.hexdigest()


def is_shared_library(name):
    return name.endswith((".dll", ".dylib", ".so")) or ".so." in name


def is_platform_shared_library(name, platform):
    if platform.startswith("windows-"):
        return name.endswith(".dll")
    if platform.startswith("darwin-"):
        return name.endswith(".dylib")
    if platform.startswith("linux-"):
        return name.endswith(".so") or ".so." in name
    return False


def host():
    machine = platform.machine().lower()
    arch = {"amd64": "x86_64", "aarch64": "arm64"}.get(machine, machine)
    return f"{platform.system().lower()}-{arch}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", default=host())
    parser.add_argument("--output", type=Path, default=ROOT / "src-tauri/runtime")
    args = parser.parse_args()
    spec = json.loads(LOCK.read_text())
    if args.platform not in spec["platforms"]:
        parser.error(f"No pinned native WebGPU package for {args.platform}. Supported: {', '.join(spec['platforms'])}")
    if not any(
        args.platform.startswith(prefix) for prefix in ("windows-", "darwin-", "linux-")
    ):
        parser.error(f"No shared-library suffix configured for {args.platform}")
    target = args.output.resolve()
    fingerprint = hashlib.sha256(LOCK.read_bytes() + args.platform.encode()).hexdigest()
    stamp = target / "manifest.json"
    required_libraries = set()
    if args.platform.startswith("windows-"):
        # Dawn's D3D12 backend loads these by name at runtime. They are shipped
        # in the official WebGPU wheel alongside the EP plugin.
        required_libraries.update(
            {
                "onnxruntime.dll",
                "onnxruntime_providers_webgpu.dll",
                "dxcompiler.dll",
                "dxil.dll",
            }
        )
    if target.exists() and any(target.iterdir()) and not stamp.is_file():
        raise RuntimeError("Refusing to replace a non-generated, non-empty output directory")
    if stamp.exists():
        previous = json.loads(stamp.read_text())
        files = previous.get("files", {})
        files_are_valid = all(
            (target / name).is_file() and digest(target / name) == value
            for name, value in files.items()
        )
        files_are_platform_specific = all(
            not is_shared_library(name)
            or is_platform_shared_library(name, args.platform)
            for name in files
        )
        if (
            previous.get("lock") == fingerprint
            and files
            and required_libraries.issubset(files)
            and files_are_valid
            and files_are_platform_specific
        ):
            print(f"ONNX Runtime {spec['onnxruntime']} + WebGPU {spec['webgpu']}: verified")
            return
    target.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=target.parent, prefix=".ort-stage-") as work:
        work = Path(work)
        staged = work / "runtime"
        staged.mkdir()
        for index, item in enumerate(spec["platforms"][args.platform]):
            archive = work / f"{index}.whl"
            request = urllib.request.Request(item["url"], headers={"User-Agent": "Lumin-runtime-builder"})
            with urllib.request.urlopen(request, timeout=120) as response, archive.open("wb") as output:
                shutil.copyfileobj(response, output)
            if digest(archive) != item["sha256"]:
                raise RuntimeError("Official runtime archive SHA-256 mismatch")
            with zipfile.ZipFile(archive) as package:
                for entry in package.infolist():
                    name = Path(entry.filename).name
                    native = (
                        is_platform_shared_library(name, args.platform)
                        and "pybind" not in name
                    )
                    license_file = name.lower().startswith(("license", "thirdpartynotices"))
                    if not native and not license_file:
                        continue
                    if license_file:
                        name = f"{index}-{name}"
                    elif name.startswith("libonnxruntime."):
                        name = "libonnxruntime.dylib" if ".dylib" in name else "libonnxruntime.so"
                    with package.open(entry) as source, (staged / name).open("wb") as output:
                        shutil.copyfileobj(source, output)
        libraries = list(staged.glob("*webgpu*"))
        if not libraries:
            raise RuntimeError("WebGPU native library missing in official archive")
        if args.platform.startswith("windows-") and not all(
            (staged / name).is_file() for name in ("dxcompiler.dll", "dxil.dll")
        ):
            raise RuntimeError(
                "Dawn D3D12 dependencies dxcompiler.dll/dxil.dll missing in official Windows archive"
            )
        if args.platform == "linux-x86_64" and not list(staged.glob("*providers_cuda*")):
            raise RuntimeError("CUDA EP missing in official Linux archive")
        metadata = {"lock": fingerprint, "platform": args.platform, "onnxruntime": spec["onnxruntime"], "webgpu": spec["webgpu"], "cuda": args.platform == "linux-x86_64", "files": {p.name: digest(p) for p in staged.iterdir()}}
        (staged / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n")
        # The directory is generated build output; replace only after full verification.
        if target.exists():
            shutil.rmtree(target)
        shutil.move(str(staged), str(target))
    print(f"Staged ONNX Runtime and WebGPU for {args.platform}: {target}")


if __name__ == "__main__":
    main()
