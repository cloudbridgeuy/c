# c - Interact with Chat AI APIs

Interact with AI chatbots through the command-line interface (CLI).

## Commands

### `anthropic`

Uses Anthropic's Claude API.

```
c anthropic [OPTIONS] --anthropic-api-key <KEY> [PROMPT]
```

#### Arguments

| Argument | Description |
|-|-|
| `PROMPT` | The prompt text to send to Claude. |

#### Options

| Option | Description |
|-|-|
| `--session` | Chat session name to store context. |
| `--model` | Claude model to use (claude-v1, claude2, etc). |
| `--max-tokens` | Max tokens to generate. |
| `--temperature` | Randomness of response. |
| `--top-k` | Only sample from top k tokens. |
| `--top-p` | Nucleus sampling top-p. |
| `--stop-sequences` | Strings to stop generation. |
| `--anthropic-api-key` | Anthropic API key. |
| `--silent` | Silent mode. |
| `--stream` | Stream response incrementally. |
| `--pin` | Pin message to history. |
| `--format` | Output format (raw, json, yaml). |
| `-h, --help` | Print help. |


### `openai`

Uses OpenAI's GPT API.

```
c openai [OPTIONS] --openai-api-key <KEY> [PROMPT]
```

#### Arguments

| Argument | Description |
|-|-|
| `PROMPT` | The prompt text to send GPT. |

#### Options

| Option | Description |
|-|-|
| `--session` | Chat session name to store context. |
| `--model` | GPT model to use (gpt3, gpt4, etc). |
| `--max-tokens` | Max tokens to generate. |
| `--temperature` | Randomness of response. |
| `--top-p` | Nucleus sampling top-p. |
| `--stop` | Sequences to stop generation. |
| `--openai-api-key` | OpenAI API key. |
| `--silent` | Silent mode. |
| `--stream` | Stream response incrementally. |
| `--pin` | Pin message to history. |
| `--format` | Output format (raw, json, yaml). |
| `-h, --help` | Print help. |

### `vertex`

Uses Google Vertex AI Code API.

```
c vertex [OPTIONS] --google-api-key <KEY> [PROMPT]
```

#### Arguments

| Argument | Description |
|-|-|
| `PROMPT` | The prompt text to send Vertex AI. |

#### Options

| Option | Description |
|-|-|
| `--session` | Chat session name to store context. |
| `--model` | Vertex model to use. |
| `--max-tokens` | Max tokens to generate. |
| `--temperature` | Randomness of response. |
| `--top-k` | Only sample from top k tokens. |
| `--top-p` | Nucleus sampling top-p. |
| `--stop-sequences` | Strings to stop generation. |
| `--google-api-key` | Google Vertex API key. |
| `--silent` | Silent mode. |
| `--stream` | Stream response incrementally. |
| `--pin` | Pin message to history. |
| `--format` | Output format (raw, json, yaml). |
| `-h, --help` | Print help. |

