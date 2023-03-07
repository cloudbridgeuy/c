#!/usr/bin/env bash
# @describe Interact with OpenAI Whisper's API

set -x

# @cmd Record an mp3 audio file, send it to OpenAI's whisper API, and print the response content to stdout.
# @option -p --path Record file path.
record() {
  if [[ -z $argc_path ]]; then
    argc_path="$(mktemp).mp3"
  fi

  # Record an mp3 audio file
  rec -c 1 -r 16000 -b 16 -e signed-integer -t raw - |
    sox -t raw -r 16000 -b 16 -e signed-integer - -t mp3 "$argc_path"

  # Send the audio file to OpenAI's whisper API
  response=$(
    curl -X POST "https://api.openai.com/v1/audio/transcriptions" \
      -H "Content-Type: multipart/form-data" \
      -H "Authorization: Bearer $OPENAI_API_KEY" \
      --form file=@"$argc_path" \
      --form model=whisper-1
  )

  # Print the response content to stdtut
  a "$(jq -r '.text' <<<"$response" | tr '[:upper:]' '[:lower:]')"
}

if [[ -z $1 ]] || [[ $1 == -* ]]; then
  set -- "record" "$@"
fi

eval "$(argc "$0" "$@")"
