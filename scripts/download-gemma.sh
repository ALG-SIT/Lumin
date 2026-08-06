#!/bin/zsh
set -euo pipefail

repo_name="litert-community/Gemma3-1B-IT"
file_name="gemma3-1b-it-int4.litertlm"
target_dir="${0:A:h:h}/LuminApp/Models"
target_path="${target_dir}/${file_name}"
expected_sha256="1325ae366d31950f137c9c357b9fa89448b176d76998180c08ceaca78bba98be"
token_value="${HF_TOKEN:-${HUGGING_FACE_HUB_TOKEN:-}}"

if [[ -z "${token_value}" ]]; then
  print -u2 "HF_TOKEN または HUGGING_FACE_HUB_TOKEN を設定してください。"
  exit 1
fi

mkdir -p "${target_dir}"
if [[ -f "${target_path}" ]]; then
  print "Gemma model already exists: ${target_path}"
  exit 0
fi

print "Downloading ${repo_name}/${file_name} (about 584 MB)..."
curl --fail --location --progress-bar \
  -H "Authorization: Bearer ${token_value}" \
  "https://huggingface.co/${repo_name}/resolve/main/${file_name}" \
  --output "${target_path}.partial"

actual_sha256="$(shasum -a 256 "${target_path}.partial" | awk '{print $1}')"
if [[ "${actual_sha256}" != "${expected_sha256}" ]]; then
  print -u2 "Checksum verification failed. The partial file was kept for inspection."
  exit 1
fi
mv "${target_path}.partial" "${target_path}"
print "Saved: ${target_path}"
