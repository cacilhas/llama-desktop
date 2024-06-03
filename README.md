[MIT]: https://github.com/cacilhas/llama-desktop/blob/master/COPYING
[Llama]: https://raw.githubusercontent.com/cacilhas/llama-desktop/master/src/assets/logo.png
[Ollama]: https://ollama.ai/

# Llamma Desktop

![Llama][]

Desktop app to connect to [Ollama][] and send queries.

Llama Desktop reads the Ollama service URI from the environment variable
`OLLAMA_HOST`, defaults to `http://localhost:11434`.

## Installation

### Ollama

In case you have an NVIDIA GPU and want to run Ollama locally:

```sh
curl -fsSL https://ollama.com/install.sh | sh
```

### Last stable release

```sh
cargo install llama-desktop
```

### Development version

```sh
cargo install git@github.com:cacilhas/llama-desktop.git
```

## License

- [MIT][]
