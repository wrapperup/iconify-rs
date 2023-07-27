//! <p>
//!     This crate provides a macro to embed SVGs from
//!     <a href="https://iconify.design/">Iconify</a>.
//!     For a list of icons, see
//!     <a href="https://icon-sets.iconify.design/">Iconify Icon Sets</a>.
//! </p>
//!
//! <div>
//!     <a href="https://crates.io/crates/iconify">
//!         <img alt="Crates.io" src="https://img.shields.io/crates/v/iconify.svg" />
//!     </a>
//!     <a href="./LICENSE">
//!         <img alt="Crates.io" src="https://img.shields.io/badge/license-MIT%2FApache-blue.svg" />
//!     </a>
//!     <a href="https://docs.rs/iconify/latest/iconify/">
//!         <img alt="docs.rs" src="https://img.shields.io/docsrs/iconify/latest" />
//!     </a>
//! </div>
//!
//!
//! ## 📝 Usage
//!
//! ```
//! let svg = iconify::svg!("mdi:home")
//! ```
//! `iconify::svg!` will download and embed an SVG as a string. It will also cache the request,
//! so it won't download the same SVG twice.
//! ```
//! let svg = "<svg>...</svg>"
//! ```
//!
//! #### Options
//!
//! You can pass options to the macro to customize the SVG.
//! ```
//! let svg = iconify::svg!("mdi:home",
//!    width = "24",
//!    height = "24",
//!    color = "red",
//!    // ... and more.
//! )
//! ```
//!
//! All options from the [Iconify API](https://iconify.design/docs/api/svg.html) are supported. You can
//! find the documentation for the options for the [svg!] macro [here](svg!).
//!
//! #### Templating
//! It can also be used directly in rsx, or any compile-time template engine.
//!
//! Maud:
//! ```
//! html! {
//!     body {
//!         .card {
//!             (PreEscaped(iconify::svg!("mdi:home")))
//!             p { "Hello!" }
//!         }
//!     }
//! }
//! ```
//!
//! Askama (Currently, a bug prevents you from using the full macro path. See [Issue #836](https://github.com/djc/askama/issues/836))
//!
//! ```
//! <body>
//!   <div class="card">
//!     {{ svg!("mdi:home")|safe }}
//!     <p>Hello!</p>
//! </body>
//! ```
//!
//! ## ✨ Features
//!
//! * Directly embed SVGs from Iconify
//! * Caches requests (default feature)
//! * Offline mode
//! * SVG transforms (through API)
//! * (Soon) CSS fetching
//!
//! ## 🔌 Offline Mode
//!
//! If you don't want iconify-rs to make requests at compile-time in CI (or other reasons), you can use offline mode with prepared icons.
//!
//! 1. Enable the `offline` feature.
//! 2. Prepare icons by setting `ICONIFY_PREPARE=true` and running `cargo check`. This will generate a directory for you in `CARGO_MANIFEST_DIR` called `icons` with all the icons you invoked.
//! 3. Now you're ready to go! Just run `cargo build` and it will use the icons you prepared.
//!
//! If you want to set a custom directory, you can also set `ICONIFY_OFFLINE_DIR`.

use proc_macro::TokenStream;

mod attrs;
mod svg;

/// Embeds an SVG from Iconify.
/// For a list of icons, see [Iconify Icon Sets](https://icon-sets.iconify.design/).
///
/// # Usage
/// The first argument is the icon's package and name, separated by a colon.
///
/// ```
///    let svg = iconify::svg!("mdi:home");
///    println!("{}", svg);
/// ```
///
/// Additional optional arguments can also be
/// passed to the macro to customize the SVG.
/// * `color = <str>`
///     * Sets the color of the SVG. This can be any valid CSS color.
/// * `width = <str>`
///    * Sets the width of the SVG. This can be any valid CSS width.
///    * If this is not set, the SVG will be rendered at 1em.
/// * `height = <str>`
///   * Sets the height of the SVG. This can be any valid CSS height.
///   * If this is not set, the SVG will be rendered at 1em.
/// * `flip = <str>`
///   * Flips the SVG horizontally, vertically, or both.
///   * Available values: "horizontal", "vertical", "both"
/// * `rotate = <str>`
///   * Rotates the SVG by the given amount.
///   * Available values: "90", "180", "270"
/// * `view_box = <bool>`
///   * If set to true, the SVG will include an invisible bounding box.
///
/// ```
/// iconify::svg!(
///     "pack:name",
///     color = "red",
///     width = "128px",
///     height = "128px",
///     flip = "horizontal",
///     rotate = "90",
///     view_box = true
/// )
/// ```
#[proc_macro]
pub fn svg(input: TokenStream) -> TokenStream {
    match svg::iconify_svg_impl(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
