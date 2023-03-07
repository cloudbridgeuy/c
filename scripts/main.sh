#!/usr/bin/env bash
# shellcheck disable=SC2154
# @describe GPT project main script.
# @author Guzmán Monné
# @version 1.0.0

self="$0"

# @cmd Runs the project in development mode
run() {
  cargo run "${argc__args[@]}"
}

# @cmd Build the project
build() {
  cargo build
}

# @cmd Run the project tests.
test() {
  cargo test
}

# @cmd Create a new project release.
release() {
  cargo build --release
}

# @cmd Copy the build binary into a directory on your path.
# @option -p --path! Directory where to send the binary.
install() {
  $self release
  cp target/release/a "$argc_path/a"
  chmod +x "$argc_path/a"
}

# @cmd Publish a new version of the project.
publish() {
  cargo publish
}

# Run `run` as the default command.
case "$1" in
  --help) ;;
  -*)
    set -- "run" "$@"
    ;;
esac

eval "$(argc "$0" "$@")"
