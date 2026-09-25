#!/usr/bin/env bash
# Read-only diagnostics for native WebGPU/Vulkan and CUDA EP startup on Linux.
set -u

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runtime_dir="${LUMIN_RUNTIME_DIR:-${root_dir}/src-tauri/runtime}"

section() {
  printf '\n## %s\n' "$1"
}

section "Host"
uname -a
if command -v nvidia-smi >/dev/null 2>&1; then
  nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader
else
  printf 'nvidia-smi: unavailable\n'
fi

section "Vulkan loader and ICDs"
if command -v vulkaninfo >/dev/null 2>&1; then
  vulkan_summary="$(vulkaninfo --summary 2>&1 || true)"
  printf '%s\n' "${vulkan_summary}" | sed -n '1,180p'
  if grep -qiE 'llvmpipe|PHYSICAL_DEVICE_TYPE_CPU' <<<"${vulkan_summary}"; then
    printf '%s\n' 'WARNING: Vulkan exposes a CPU/software adapter; WebGPU cannot establish NVIDIA GPU inference.'
  fi
else
  printf 'vulkaninfo: unavailable (Ubuntu/WSL: sudo apt-get install vulkan-tools)\n'
fi
for icd_dir in /usr/share/vulkan/icd.d /etc/vulkan/icd.d; do
  if [[ -d "${icd_dir}" ]]; then
    find "${icd_dir}" -maxdepth 1 -type f -name '*.json' -printf '%p\n' | sort
  fi
done

section "Bundled ONNX Runtime"
for library in libonnxruntime.so libonnxruntime_providers_webgpu.so; do
  path="${runtime_dir}/${library}"
  if [[ -f "${path}" ]]; then
    printf '%s\n' "${path}"
    ldd "${path}" | grep -E 'not found|libcuda|libcudart|libcublas|libcudnn|libvulkan' || true
  else
    printf 'missing: %s\n' "${path}"
  fi
done

section "Next step"
printf '%s\n' 'Run with LUMIN_EXECUTION_PROVIDER=webgpu to isolate native WebGPU/Vulkan.'
