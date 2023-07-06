<h1 align="center">
    iconify-rs
</h1>

<p align="center">
    This crate provides a macro to embed SVGs from
    <a href="https://iconify.design/">Iconify</a>.
    For a list of icons, see
    <a href="https://icon-sets.iconify.design/">Iconify Icon Sets</a>.
</p>

<div align="center">
    <a href="https://crates.io/crates/iconify">
        <img alt="Crates.io" src="https://img.shields.io/crates/v/iconify.svg" />
    </a>
    <a href="./LICENSE">
        <img alt="Crates.io" src="https://img.shields.io/badge/license-MIT%2FApache-blue.svg" />
    </a>
    <a href="https://docs.rs/iconify/latest/iconify/">
        <img alt="docs.rs" src="https://img.shields.io/docsrs/iconify/latest" />
    </a>
</div>


## 📝 Usage 

```jsx
let svg = iconify::svg!("mdi:home", color = "red")
```
`iconify::svg!` will download and embed an SVG as a string. It will also cache the request,
so it won't download the same SVG twice.
```rust
let svg = "<svg>...</svg>"
```

### Options

You can pass options to the macro to customize the SVG.
```rust
let svg = iconify::svg!("mdi:home",
    width = "24",
    height = "24",
    color = "red",
    // ... and more.
)
```

All options from the [Iconify API](https://iconify.design/docs/api/svg.html) are supported. You can
find the documentation for the options for the `svg!` macro [here](https://docs.rs/iconify/latest/iconify/macro.svg.html).

### Templating
It can also be used directly in rsx, or any compile-time template engine.

Maud:
```rust
html! {
    body {
        .card {
            (PreEscaped(iconify::svg!("mdi:home")))
            p { "Hello!" }
        }
    }
}
```

Askama (Currently, a bug prevents you from using the full macro path. See [Issue #836](https://github.com/djc/askama/issues/836))

```jsx
<body>
  <div class="card">
    {{ svg!("mdi:home")|safe }}
    <p>Hello!</p>
</body>
```

## ✨ Features

* Directly embed SVGs from Iconify
* Caches requests (default feature)
* Offline mode
* SVG transforms (through API)
* (Soon) CSS fetching

## 🔌 Offline Mode

If you don't want iconify-rs to make requests at compile-time in CI (or other reasons), you can use offline mode with prepared icons.

1. Enable the `offline` feature.
2. Prepare icons by setting `ICONIFY_PREPARE=true` and running `cargo check`. This will generate a directory for you in `CARGO_MANIFEST_DIR` called `icons` with all the icons you invoked.
3. Now you're ready to go! Just run `cargo build` and it will use the icons you prepared.

If you want to set a custom directory, you can also set `ICONIFY_OFFLINE_DIR`.

