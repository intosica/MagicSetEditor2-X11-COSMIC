# MagicSetEditor2-X11-COSMIC

Native Rust/libcosmic rewrite of Magic Set Editor 2.

* `mse_core` – parsing (`.mse-set`), templates, symbol + card rendering
* `mse_app`  – libcosmic GUI (`cargo run -p mse_app -- my.mse-set`)

Test the core without any GUI deps: `cargo test -p mse_core`

License: GPL-2.0-or-later (compatible with upstream MSE, from which formats/logic are ported).
