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
    target = args.output.resolve()
    fingerprint = hashlib.sha256(LOCK.read_bytes() + args.platform.encode()).hexdigest()
    stamp = target / "manifest.json"
    if target.exists() and any(target.iterdir()) and not stamp.is_file():
        raise RuntimeError("Refusing to replace a non-generated, non-empty output directory")
    if stamp.exists():
        previous = json.loads(stamp.read_text())
        if previous.get("lock") == fingerprint and all((target / name).is_file() and digest(target / name) == value for name, value in previous.get("files", {}).items()) and previous.get("files"):
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
                    native = (name.startswith(("libonnxruntime", "onnxruntime")) and (name.endswith((".dll", ".dylib", ".so")) or ".so." in name) and "pybind" not in name)
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
        metadata = {"lock": fingerprint, "platform": args.platform, "onnxruntime": spec["onnxruntime"], "webgpu": spec["webgpu"], "files": {p.name: digest(p) for p in staged.iterdir()}}
        (staged / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n")
        # The directory is generated build output; replace only after full verification.
        if target.exists():
            shutil.rmtree(target)
        shutil.move(str(staged), str(target))
    print(f"Staged ONNX Runtime and WebGPU for {args.platform}: {target}")


if __name__ == "__main__":
    main()
