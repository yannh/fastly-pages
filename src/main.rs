use std::path::Path;

use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};
use rust_embed::RustEmbed;
use std::ffi::OsStr;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

fn file_mimetype(filename: &str, default: mime::Mime) -> mime::Mime {
    let extension = Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase());
    match extension {
        Some(ext) => match ext.as_str() {
            "css" => mime::TEXT_CSS_UTF_8,
            "gif" => mime::IMAGE_GIF,
            "html" | "htm" => mime::TEXT_HTML_UTF_8,
            "jpeg" | "jpg" => mime::IMAGE_JPEG,
            "js" => mime::TEXT_JAVASCRIPT,
            "svg" => mime::IMAGE_SVG,
            _ => default,
        },
        _ => default,
    }
}

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    // Filter request methods...
    match req.get_method() {
        // Allow GET and HEAD requests.
        &Method::GET | &Method::HEAD => (),

        // Deny anything else.
        _ => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD")
                .with_body_text_plain("This method is not allowed\n"))
        }
    };

    const DEFAULT_MIMETYPE: mime::Mime = mime::APPLICATION_OCTET_STREAM;
    let mut filename = req.get_path().trim_start_matches("/");
    if filename == "" {
        filename = "index.html";
    }

    match Asset::get(filename) {
        Some(asset) => Ok(Response::from_status(StatusCode::OK)
            .with_body_bytes(asset.data.as_ref())
            .with_content_type(file_mimetype(filename, DEFAULT_MIMETYPE))),

        None => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain(&*format!("404 error, {} not found!", req.get_path()))),
    }
}
