# Dev files

For the browser preview (`pnpm dev`) and the README graphics, never shipped in the app.

- `samples/`: eight product photos made with image generation. They aren't real products or real shoots.
- `recipes.json`: the starter recipes as JSON, for the browser preview's stand-in engine. Written with the TypeScript types: `cargo test -p wrangler-engine --features ts export_bindings`.
