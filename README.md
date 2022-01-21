# bevy_wordle_viz

## Instructions

- install [wasm-server-runner](https://github.com/jakobhellermann/wasm-server-runner)
- build

```
cargo run --target=wasm32-unknown-unknown [--release]
```

- Use [wordle](https://www.powerlanguage.co.uk/wordle/) "share" button to copy data to clipboard
- Open browser to http://127.0.0.1:1334 and paste the data

## Notes

Your favorite social media platform or chat app might mangle the output of wordle's share feature by replacing glyphs with their own icons.

This only works with the raw unicode output from wordle.

```
Wordle 216 4/6

🟨⬛⬛⬛⬛
🟨⬛⬛⬛⬛
⬛🟩🟩⬛🟩
🟩🟩🟩🟩🟩
```
