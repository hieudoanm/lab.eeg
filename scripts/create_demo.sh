#!/usr/bin/env bash
#
# Start EEG Lab, automate a complete browser workflow, and save one MP4 demo.
#
# Requirements: Docker Compose, pnpm/Node.js, ffmpeg, and Chromium
# (Playwright installs Chromium on demand).
#
# Usage:
#   ./scripts/create_demo.sh
#
# The Compose services remain running after the recording is created. Stop them
# separately with `docker compose down`.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
demo_dir="${repo_root}/demo"
work_dir="${demo_dir}/.create-demo-$$"
recorder="${repo_root}/packages/frontend/e2e/create-demo.ts"
output="${demo_dir}/eeg-lab-demo.mp4"
recording="${demo_dir}/fixtures/resting-state-eeg.npy"
webm=""

cleanup() {
  rm -rf "${work_dir}"
}
trap cleanup EXIT

for command in docker pnpm ffmpeg ffprobe; do
  if ! command -v "${command}" >/dev/null 2>&1; then
    echo "Required command not found: ${command}" >&2
    exit 1
  fi
done

# Ensure tsx is available for running TypeScript demo recorder
echo "Installing tsx for TypeScript execution..."
pnpm --dir "${repo_root}/packages/frontend" add -D tsx

mkdir -p "${demo_dir}" "${work_dir}"

if [[ ! -s "${recording}" ]]; then
  echo "The fixed demo recording is missing or empty: ${recording}" >&2
  exit 1
fi

echo "Starting EEG Lab with Docker Compose..."
docker compose -f "${repo_root}/docker-compose.yml" -p eeg-lab up --build -d

wait_for_url() {
  local url="$1"
  local description="$2"
  local attempt
  for attempt in $(seq 1 120); do
    if curl --fail --silent --show-error "${url}" >/dev/null 2>&1; then
      echo "${description} is ready."
      return 0
    fi
    sleep 2
  done
  echo "Timed out waiting for ${description} at ${url}." >&2
  docker compose -f "${repo_root}/docker-compose.yml" -p eeg-lab ps >&2
  exit 1
}

wait_for_url "http://localhost:8000/health/ready" "EEG Lab API"
wait_for_url "http://localhost:3000/login" "EEG Lab frontend"

echo "Using fixed synthetic EEG recording: ${recording}"

echo "Ensuring the Playwright Chromium browser is installed..."
pnpm --dir "${repo_root}/packages/frontend" exec playwright install chromium

echo "Recording the application walkthrough..."
(
  cd "${repo_root}/packages/frontend"
  DEMO_RECORDING="${recording}" \
    DEMO_VIDEO_DIRECTORY="${work_dir}" \
    pnpm exec tsx "${recorder}"
)

webm="$(find "${work_dir}" -type f -name '*.webm' -print -quit)"
if [[ -z "${webm}" || ! -f "${webm}" ]]; then
  echo "Playwright completed without producing a browser recording." >&2
  exit 1
fi

echo "Encoding MP4..."
ffmpeg -hide_banner -loglevel error -y \
  -i "${webm}" \
  -an \
  -c:v libx264 \
  -preset medium \
  -crf 22 \
  -pix_fmt yuv420p \
  -movflags +faststart \
  "${output}"

if ! ffprobe -v error -select_streams v:0 -show_entries stream=codec_name \
  -of default=noprint_wrappers=1:nokey=1 "${output}" | grep -qx h264; then
  echo "The generated MP4 does not contain an H.264 video stream." >&2
  exit 1
fi

echo "Demo video created: ${output}"
echo "Frontend: http://localhost:3000"
echo "API:      http://localhost:8000"
echo "Services remain running; use 'docker compose -p eeg-lab down' to stop them."