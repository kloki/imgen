# imgen

A cli wrapper around OpenAI image generation that works for me.
Give multiple prompts and request them concurrently.

Install

```
cargo install imgen
```

# Config

OpenAI api key should be set using envar `OPENAI_API_KEY`.

# Run

```
imgen "First prompt" "Second prompt"
```

If a prompt is `.` imgen will reuse the previous prompt. This way you can easily request variations like this.

```
imgen "prompt" . .
```
