use core::fmt;
use std::{env, str::FromStr};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};

use crate::attrs::{get_lit_bool, get_lit_str};

trait AppendQueryPair {
    fn append_query_pair(&mut self, key: &str, value: &Option<String>);
}

impl AppendQueryPair for url::form_urlencoded::Serializer<'_, url::UrlQuery<'_>> {
    fn append_query_pair(&mut self, key: &str, value: &Option<String>) {
        if let Some(value) = value {
            self.append_pair(key, &value.to_string());
        }
    }
}

enum IconifyRotation {
    Rotate90,
    Rotate180,
    Rotate270,
}

impl fmt::Display for IconifyRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IconifyRotation::Rotate90 => write!(f, "90deg"),
            IconifyRotation::Rotate180 => write!(f, "180deg"),
            IconifyRotation::Rotate270 => write!(f, "270deg"),
        }
    }
}

impl FromStr for IconifyRotation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "90" => Ok(IconifyRotation::Rotate90),
            "180" => Ok(IconifyRotation::Rotate180),
            "270" => Ok(IconifyRotation::Rotate270),
            _ => Err(()),
        }
    }
}

enum IconifyFlip {
    Horizontal,
    Vertical,
    Both,
}

impl fmt::Display for IconifyFlip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IconifyFlip::Horizontal => write!(f, "horizontal"),
            IconifyFlip::Vertical => write!(f, "vertical"),
            IconifyFlip::Both => write!(f, "horizontal,vertical"),
        }
    }
}

impl FromStr for IconifyFlip {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "horizontal" => Ok(IconifyFlip::Horizontal),
            "vertical" => Ok(IconifyFlip::Vertical),
            "both" | "horizontal,vertical" | "vertical,horizontal" => Ok(IconifyFlip::Both),
            _ => Err(()),
        }
    }
}

struct IconifyInput {
    pack: String,
    name: String,
    color: Option<String>,
    width: Option<String>,
    height: Option<String>,
    flip: Option<IconifyFlip>,
    rotate: Option<IconifyRotation>,
    view_box: bool,
}

impl IconifyInput {
    fn icon_url(&self) -> Result<String, url::ParseError> {
        let mut url = url::Url::parse(&iconify_url())?;

        // Set the pack and icon name in the url path.
        {
            let mut path_segments = url
                .path_segments_mut()
                .map_err(|_| url::ParseError::RelativeUrlWithoutBase)?;

            path_segments.push(&self.pack);
            path_segments.push(&format!("{}.svg", &self.name));
        }

        // Set the query parameters.
        {
            let mut query_pairs = url.query_pairs_mut();

            query_pairs.append_query_pair("color", &self.color);
            query_pairs.append_query_pair("width", &self.width);
            query_pairs.append_query_pair("height", &self.height);
            query_pairs.append_query_pair("flip", &self.flip.as_ref().map(IconifyFlip::to_string));
            query_pairs.append_query_pair(
                "rotate",
                &self.rotate.as_ref().map(IconifyRotation::to_string),
            );
            query_pairs.append_query_pair(
                "box",
                &self.view_box.then(|| Some("true".to_string())).flatten(),
            );
        }

        Ok(url.to_string())
    }

    #[cfg(feature = "cache")]
    fn hash_digest(&self) -> Result<String, syn::Error> {
        use hex::ToHex;

        let mut buf = [0u8; 8];
        let url = self.icon_url().map_err(|err| {
            syn::Error::new(Span::call_site(), format!("failed to parse url: {err}"))
        })?;

        blake3::Hasher::new()
            .update(url.as_bytes())
            .finalize_xof()
            .fill(&mut buf);

        Ok(buf.encode_hex::<String>())
    }
}

impl Parse for IconifyInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pack_name_lit = input.parse::<syn::LitStr>()?;
        let pack_name_string = pack_name_lit.value();

        let mut pack_name = pack_name_string.split(':');

        let error = || syn::Error::new(pack_name_lit.span(), "expected `pack_name:icon_name`");
        let pack = pack_name.next().ok_or_else(error)?.to_string();
        let name = pack_name.next().ok_or_else(error)?.to_string();

        if pack_name.next().is_some() {
            return Err(error());
        }

        let mut color = None;
        let mut width = None;
        let mut height = None;
        let mut flip = None;
        let mut rotate = None;
        let mut view_box = false;

        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let metas = input.parse_terminated(syn::Meta::parse, syn::Token![,])?;

            for meta in metas {
                use syn::Meta::NameValue;
                match meta {
                    // Parse syn!("...", color = ...)].
                    NameValue(m) if m.path.is_ident("color") => {
                        let value = get_lit_str("color", &m.value)?;
                        color = Some(value.value());
                    }
                    // Parse syn!("...", width = ...)].
                    NameValue(m) if m.path.is_ident("width") => {
                        let value = get_lit_str("width", &m.value)?;
                        width = Some(value.value());
                    }
                    // Parse syn!("...", height = ...)].
                    NameValue(m) if m.path.is_ident("height") => {
                        let value = get_lit_str("height", &m.value)?;
                        height = Some(value.value());
                    }
                    // Parse syn!("...", flip = ...)].
                    NameValue(m) if m.path.is_ident("flip") => {
                        let value = get_lit_str("flip", &m.value)?;
                        let flip_val = IconifyFlip::from_str(&value.value())
                            .map_err(|_| syn::Error::new(value.span(), "Invalid flip value"))?;
                        flip = Some(flip_val);
                    }
                    // Parse syn!("...", rotate = ...)].
                    NameValue(m) if m.path.is_ident("rotate") => {
                        let value = get_lit_str("rotate", &m.value)?;
                        let rotate_val = IconifyRotation::from_str(&value.value()).map_err(|_| {
                            syn::Error::new_spanned(
                            value,
                            "Invalid rotate value. Rotate can be one of \"90\", \"180\", or \"270\".",
                        )
                        })?;
                        rotate = Some(rotate_val);
                    }
                    // Parse syn!("...", view_box = ...)].
                    NameValue(m) if m.path.is_ident("view_box") => {
                        view_box = get_lit_bool("view_box", &m.value)?;
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "Not a name value pair: `foo = \"...\"`",
                        ));
                    }
                }
            }
        }

        Ok(Self {
            pack,
            name,
            color,
            width,
            height,
            flip,
            rotate,
            view_box,
        })
    }
}

fn iconify_url() -> String {
    env::var("ICONIFY_URL").unwrap_or("https://api.iconify.design".to_string())
}

#[cfg(feature = "cache")]
fn iconify_cache_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
    use directories::BaseDirs;

    if let Ok(dir) = env::var("ICONIFY_CACHE_DIR") {
        return PathBuf::from(dir);
    }

    let dir = PathBuf::from(BaseDirs::new().unwrap().cache_dir());

    // I didn't like the idea of having a cache dir in the root of %LOCALAPPDATA%.
    #[cfg(target_os = "windows")]
    dir.push("cache");

    dir.join("iconify-rs")
}

#[cfg(feature = "cache")]
fn iconify_cache_path(input: &IconifyInput) -> Result<std::path::PathBuf, syn::Error> {
    let digest = input.hash_digest()?;

    let mut path = iconify_cache_dir();
    path.push(&input.pack);
    path.push(format!("{}-{}", input.name, digest));
    path.set_extension("svg");
    Ok(path)
}

#[cfg(feature = "offline")]
fn offline_dir() -> PathBuf {
    if let Ok(dir) = env::var("ICONIFY_OFFLINE_DIR") {
        return PathBuf::from(dir);
    }

    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("icons");
    path
}

#[cfg(feature = "offline")]
fn offline_icon_path(input: &IconifyInput) -> Result<PathBuf, syn::Error> {
    let digest = input.hash_digest()?;

    let mut path = offline_dir();
    path.push(&input.pack);
    path.push(format!("{}-{}", input.name, digest));
    path.set_extension("svg");
    Ok(path)
}

#[cfg(feature = "offline")]
fn offline_svg(input: &IconifyInput) -> Result<String, syn::Error> {
    let path = offline_icon_path(input)?;

    fs::read_to_string(&path).map_err(|err| {
        syn::Error::new(
            Span::call_site(),
            format!("failed to read offline icon: {err}"),
        )
    })
}

#[cfg(feature = "offline")]
fn prepare_offline_icons() -> bool {
    env::var("ICONIFY_PREPARE").ok().as_deref() == Some("true")
}

fn fetch_svg(iconify_input: &IconifyInput) -> Result<String, syn::Error> {
    #[cfg(feature = "cache")]
    let path = {
        let path = iconify_cache_path(iconify_input)?;

        if let Ok(text) = std::fs::read_to_string(&path) {
            return Ok(text);
        }

        path
    };

    let url = iconify_input
        .icon_url()
        .map_err(|err| syn::Error::new(Span::call_site(), format!("couldn't parse url: {err}")))?;

    let request = reqwest::blocking::get(url).map_err(|err| {
        syn::Error::new(Span::call_site(), format!("failed to fetch icon: {err}"))
    })?;

    if !request.status().is_success() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("failed to fetch icon: {}", request.status()),
        ));
    }

    let text = request.text().map_err(|err| {
        syn::Error::new(Span::call_site(), format!("failed to fetch icon: {err}"))
    })?;

    #[cfg(feature = "cache")]
    {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
    }

    Ok(text)
}

pub fn iconify_svg_impl(input: TokenStream) -> TokenStream {
    let iconify_input = syn::parse_macro_input!(input as IconifyInput);

    // If we're using offline icons, we need to fetch them from the
    // iconify API during development. This is done by setting the
    // ICONIFY_PREPARE environment variable.
    #[cfg(feature = "offline")]
    let svg_result = if prepare_offline_icons() {
        fetch_svg(&iconify_input)
    } else {
        offline_svg(&iconify_input)
    };

    #[cfg(not(feature = "offline"))]
    let svg_result = fetch_svg(&iconify_input);

    let svg = match svg_result {
        Ok(svg) => svg,
        Err(err) => return err.to_compile_error().into(),
    };

    #[cfg(feature = "offline")]
    if prepare_offline_icons() {
        // Prepare offline icons
        let path = match offline_icon_path(&iconify_input) {
            Ok(path) => path,
            Err(err) => return err.to_compile_error().into(),
        };

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &svg).unwrap();
    }

    quote! {
        #svg
    }
    .into()
}
