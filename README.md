c
=

A CLI application to interact with different LLM models from multiple vendors while maintaining an
ongoing session.

# Introduction

I started this project after taking a look at [`ddddddeon/a`](https://github.com/ddddddeon/a) which
used the OpenAI completion API to generate code from a text prompt that conform to the following
form:

```bash
a bash function to record the voice of a human
```

It was the first time I saw the combination of code with prompts to implement a powerful feature,
and I understood that with the proper manipulation of prompts you could be able to create different
application using the same code base. Moreover, you would be able to take advantage of different
models, that better adapt to your use case, without needing to use a separate tool.

Since this was my first project written in `rust` I wanted to dip my toes by trying to rewrite `a`
using `clap`, you can still find a version of this crate. It works exactly the same as the one
crated by [`ddddddeon`](https://github.com/ddddddeon) only it uses `clap` for better `--help`
output.

Then I created a new CLI that would let you configure any prompt, while maintaining a history of
all previous messages. I also added the ability to estimate the number of tokens your prompt would
consume so that the app would remove messages to ensure your prompt is not to large for the model.
If you wanted to ensure a message would not be removed, you could `pin` it, and so the program
would avoid removing it. Some pre-build binaries and the crate code can be found on the `crates/`
directory.

This new iteration had most of what I wanted, only it made it hard to add new vendors. I also added
support for almost all endpoints exposed by OpenAI, which ended up being useless, given that I only
used the `chat` API. It's important to note that I've been dogfooding this tool since its inception,
helping me out as I code along.

`c` (I'm not good with names) is my first attempt at this tool, and I think it's the best one. It's
much easy to extend, exposes a single `Session` object to store the history of every vendor, and
exposes multiple different chat API's from a single `cli`:

- OpenAI
- Anthropic
- Google VertexAI

I plan to add more in the future.

## Getting started

If you have `macOS` you can download the latest release from the [`Releases`](https://github.com/cloudbridgeuy/c/releases) page.

> All compiled binaries are only for `macOS`. More coming soon!

If don't you'll have to compile the project for your platform. I haven't tested it on any other
platform other than `macOS` but the following steps should work:

1. Clone the repository. If you have access to the `GPT-4` api you can use the `main` branch. If not
   clone the repository from any of the tagged commits.
2. Run `cargo xtask install --name c --path $CARGO_HOME/bin`.

You can substitute `$CARGO_HOME` for any other directory. The `cargo xtask install` command will
build the `c` binary, give it write permissions, and move it into the folder you provide.

> You can run `cargo xtask build --name c --release` if you want to move the binary yourself later.

### OpenAI Key

To use the OpenAI chat interface you must provide your `OPEN_AI_KEY` as an environment variable. You can get your API
key [here](https://platform.openai.com/account/api-keys). Just sign-in with your credentials and click
`Create new secret key`. Copy the key and load it into a terminal session.

```bash
export OPEN_AI_KEY=<YOUR_API_KEY>
```

Or you may provide it through the `--openai-api-key` option when calling the `c openai` command.

I suggest that you include this command in your `dotfiles` so it gets loaded automatically on all
terminal sessions.

### Anthropic Key

Same as with OpenAI, you need your own `ANTHROPIC_AI_KEY` in order to use the Anthropic chat API
endpoint. Follow [these](https://docs.anthropic.com/claude/reference/getting-started-with-the-api)
to get yours.

```bash
export ANTHROPIC_API_KEY=<YOUR_API_KEY>
```

Or you may provide it through the `--anthropic-api-key ` option when calling the `c anthropic` command.

I suggest that you include this command in your `dotfiles` so it gets loaded automatically on all
terminal sessions.

### Google Vertex AI

I have been using Google Cloud for a few years, and I still get triped by their authentication
methods. To use this API you need to enable the Vertex AI endpoint on your Google Cloud project
and configure the `gcp_region`, `gcp_project`, and `gcp_key` values. I suggest you configure the
first two as environment variables like this:

```bash
export C_GCP_REGION="<YOUR_GCP_REGION>"
export C_GCP_PROJECT="<YOUR_GCP_PROJECT>"
```

Then you can provide the key on each command by running:

```bash
gcloud auth print-access-token
```

And passing that value to the `--gcp-key` option of the `c vertext` command. Here's an example:

```bash
c vertex \
    --gcp-key="$(gcloud auth print-access-token)" \
    'Give me a function in rust that returns the `n` number in a fibonnacci series using mnemoization'
```

Output:

```
fn fibonacci(n: usize) -> usize {
    let mut memo = vec![0; n + 1];
    memo[0] = 0;
    memo[1] = 1;

    for i in 2..n + 1 {
        memo[i] = memo[i - 1] + memo[i - 2];
    }

    memo[n]
}
```

> I have no idea if that code works.

# Usage

Once you have all the necessary permissions, you can execute the `c` command on any of the supported
LLM models:

```bash
❯ c --help
Interact with OpenAI's ChatGPT through the terminal

Usage: c [COMMAND]

Commands:
  anthropic  Anthropic Chat AI API
  openai     OpenAi Chat AI API
  vertex     Google Vertex AI Chat Code API
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Stdin

The prompt is the only positional argument supported by each `command` but you can also pass your it
through `stdin`.

> You need to pass a `-` as prompt for `c` to read from `stdin`, just like you would do when using
> the `kubectl` cli.

```bash
# Notice the `-` in place of the prompt.
❯ cat <<-'EOF' | c vertex -
I need a function to record the user voice using the default microphone.
EOF
```

Output

```python
import pyaudio
import wave

# Set the audio format.
FORMAT = pyaudio.paInt16
CHANNELS = 1
RATE = 44100

# Create an audio object.
audio = pyaudio.PyAudio()

# Open the default microphone.
stream=$(audio.open(format=FORMAT, channels=CHANNELS, rate=RATE, input=True, frames_per_buffer=1024))

# Start recording.
print("Recording...")
frames = []
for i in range(0, int(RATE / 1024 * 5)):
    data = stream.read(1024)
    frames.append(data)

# Stop recording.
print("Done recording.")
stream.stop_stream()
stream.close()

# Save the audio file.
wavefile = wave.open("output.wav", "wb")
wavefile.setnchannels(CHANNELS)
wavefile.setsampwidth(audio.get_sample_size(FORMAT))
wavefile.setframerate(RATE)
wavefile.writeframes(b"".join(frames))
wavefile.close()

# Close the audio object.
audio.terminate()
```

> The Vertex AI is the best one at returning code, but it doesn't work for general purpouse
> questions, and it defaults to `python` when you don't clarify the language.

Another good tool to use to write blocks of code in the terminal is [`gum`](https://github.com/charmbracelet/gum).
Here's how you would use it:

```bash
# o is an alias for the openai commans.
c o <<<"$(gum write --placeholder "Details of this change (CTRL+D to finish)" --width=80 --height=20)"
```

If you feel is to verbose you can wrap it in a function:

```bash
function co() {
  c o - <<<"$(gum write --placeholder "Details of this change (CTRL+D to finish)" --width=80 --height=20)"
}
```

Then you can just type `co` and get directly to creating your prompt.

## Whisper

On the first iteration of this app, `a`, I added a command to be able to record the user message
using the microphone, transcode it into text using the `whisper` API, and then returning the result.
I never used it so I didn't port it to `b` or `c`. But it's there if you want to try it.

```bash
a whisper
```

**NOTE**: If you want your output to include syntax highlight, start your recordings with the name
of the programming language you want. Just like if you were writing your prompt.

> You need `rec`, `ffmpeg`, and `curl` for the `whisper` command to work. I haven't found a way
> to create 100% native `rust` implementation of the recording mechanism.

## Sessions

Evere command takes a `--session` option. This creates a YAML file at `$HOME/.c/sessions` that will
hold information about the messages exchanged and the configuration options. A useful thing that the
session files provide is that you can edit yours or the LLM previous response to make it return a
better answer. For example, let's say we want to craft a prompt using `Clade` from anthropic that
will simulate flipping a coin. It sounds trivial so we might try:

```bash
c a 'Flip a coin'
```

But it's answer would look like this:

```
 I apologize, but I do not actually have a physical coin to flip. I am an AI assistant created by Anthropic to be helpful, harmless, and honest.
```

Let's do the same using the session name `coin`, this will create a file at
`~/.c/sessions/coin.yaml` that we can edit.

```bash
c a --session coin 'Flip a coin'
```

Open the file and edit it like so:

```yaml
id: coin
vendor: Anthropic
history:
- content: Flip a coin
  role: human
  pin: true
- content: tails
  role: assistant
  pin: true
options:
  model: claude-v1
  max_tokens_to_sample: 1000
max_supported_tokens: 8000
```

We did two changes to this file:

1. We changed the output of the previous message.
2. We set `pin` to true on both messages, so that these messages would never be removed from the
   context if the conversation gets too long.

Now, let's ask it again to flip a coin.

```bash
c a --session coin 'Again'
heads
```

Anthropic's Claude Model is not as good as OpenAI GPT models at folowing these kinds of orders, so
we could run this with `gpt4` instead.

> As of today, I haven't added the necessary logic to automatically migrate from one vendor's
> session to another, but it's something I'll definetely work on.

We need to edit the file again to do so:

```yaml
id: coin
vendor: OpenAI
history:
- content: Flip a coin
  role: human
  pin: true
- content: tails
  role: assistant
  pin: true
- content: Again
  role: human
  pin: false
options:
  model: gpt-4
max_supported_tokens: 8000
```

This time we:

1. Changed the `vendor` to `OpenAI`.
2. Changed the `options` to use the `gpt4` model.

And now we can run it as many times as we want and get the result we expect.

We could also change other properties on subsequent commands. Let's set the `max_tokens` count to
4, so OpenAI never returns more than the word we want.

```bash
c o --session coin --max-tokens 4 'Again'
tails
```

If you take a look at the `sessions` file you'll see the new value has been saved for the next
execution.

# Unix Style

One of my goals with this tool was to make it in a way that it was compatible with other tools I use
on a daily basis on the CLI, so I could craft custom and more complex mechanics without having to
implement them in `c`, given that they may only be useful to me. Here are some example of what I
mean.

I have multiple sessions that I use to ask question about different programming languages, but I
don't want to track what `vendor` I'm using for each. I also find myself looking at the `session`
file constanlty, so I wanted an easy way to open it in my editor. Lastly, I realized that most of
the prompts I write are multiline, so I wanted to be able to load my editor to write this long
prompts, but also be able to see what the previous answer was.

I took all of these requirements and I build this `bash` function called `chat`:

```bash
# Handy function to interact with `c` My custom LLM chat cli.
function chat() {
	if [ -z "$1" ]; then
		session="$(ls ~/.c/sessions | awk -F'.' '{print $1}' | fzf)"
	else
		session="$1"
		shift
	fi

	if [[ "$session" == "" ]]; then
		echo "No session selected"
		return 1
	fi

	if [[ "$1" == "" ]]; then
		tmp="$(mktemp)"

		if [ ! -f "~/.c/sessions/${session}.yaml" ]; then
			one="$(yq '.history[-2]' ~/.c/sessions/$session.yaml)"
			two="$(yq '.history[-1]' ~/.c/sessions/$session.yaml)"

			if [[ -n "$one" ]]; then
				echo "# $(yq '.role' <<<"$one")" >>"$tmp"
				echo >> "$tmp"
				echo "$(yq '.content' <<<"$one")" >>"$tmp"
			fi

				echo >> "$tmp"

			if [[ -n "$two" ]]; then
				echo "# $(yq '.role' <<<"$two")" >>"$tmp"
				echo >> "$tmp"
				echo "$(yq '.content' <<<"$two")" >>"$tmp"
			fi

			echo >> "$tmp"
			echo '<EOF/>' >> "$tmp"
			echo >> "$tmp"
			echo >> "$tmp"
		else
			echo "Can't find file ~/.c/sessions/$session.yaml"
		fi

		nvim +'normal Gzt' +'set filetype=markdown' +'startinsert' "$tmp"

		if [[ $! -ne 0 ]]; then
			return $!
		fi

		prompt="$(grep -n '<EOF/>' "$tmp" | awk -F':' '{ print $1 }' | xargs -n1 -I{} expr 2 + {} | xargs -n1 -I{} tail -n +{} "$tmp")"
		c anthropic --session "$session" --stream "$prompt"
		return $!
	fi

	if [[ "$1" == "edit" ]]; then
		nvim ~/.c/sessions/"$session".yaml
		return 0
	fi

	vendor="$(yq '.vendor' ~/.c/sessions/"$session".yaml)"

	case "$vendor" in
		Anthropic)
			subcommand="anthropic"
			;;
		OpenAI)
			subcommand="openai"
			;;
		Google)
			subcommand="vertex"
			;;
		*)
			echo "Unknown vendor $vendor"
			return 1
			;;
	esac

	c $subcommand --session "$session" --stream "$@"
}
```

It's long and sloppy but it does exactly what I wanted, and it works for me. You'll probably have
some other requirements but you'll be able to easily integrate `c` to your workflow.

# Anonymous sessions

All prompts create a session object that its used to generate the completion. By default, all of
these anonymous sessions are stored at `~/.c/session/anonymous`.
