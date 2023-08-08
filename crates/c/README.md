# c - Chatbot CLI

## Usage

```
c [COMMAND]
```

## Commands

### anthropic

Anthropic Chat API

```
c anthropic [OPTIONS] --anthropic-api-key <ANTHROPIC_API_KEY> [PROMPT]
```

Interact with Anthropic's Claude chatbot API. Allows specifying model, temperature, max tokens etc.

### openai

OpenAI Chat API

```
c openai [OPTIONS] --openai-api-key <OPENAI_API_KEY> [PROMPT]
```

Interact with OpenAI's chatbot API. Allows specifying model, temperature, max tokens etc.

### help

Print help for commands and options

```
c help [COMMAND]
```

## Options

**-h, --help**

Print help

## Anthropic Options

**--model**

Claude model to use (default: claude-v1)

**--temperature**

Randomness of response (default: 1)

**--max-tokens**

Maximum tokens to generate (default: 1000)

**--stop-sequences**

Strings to stop generation

**--anthropic-api-key**

Anthropic API key

## OpenAI Options

**--model**

OpenAI model to use

**--temperature**

Randomness of response (default: 1)

**--max-tokens**

Maximum tokens to generate

**--openai-api-key**

OpenAI API key

## Output Options

**-s, --silent**

Silent mode

**--stream**

Stream output

**-f, --format**

Output format (default: raw)
