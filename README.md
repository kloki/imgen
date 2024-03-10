# imgen

A cli wrapper around OpenAI imgage generation that works for me.

# Run

```
cargo run --release -- "First prompt" "Second prompt"
```

if a prompt is `.` is will reuse the previous prompt. So the create 3 variations run:

```
cargo run --release -- "prompt" . .
```
