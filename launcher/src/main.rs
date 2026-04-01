use mime_guess::from_path;
use rust_embed::Embed;
use std::io::Cursor;
use tiny_http::{Header, Response, Server};

#[derive(Embed)]
#[folder = "../dist/"]
struct Assets;

const PORT: u16 = 7979;

fn main() {
    let addr = format!("127.0.0.1:{PORT}");
    let server = Server::http(&addr).expect("failed to bind");
    let url = format!("http://{addr}");

    println!("Kanban running at {url}");
    println!("Press Ctrl+C to stop.");

    // Open the browser after a brief moment so the server is ready
    let url_clone = url.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        if let Err(e) = open::that(&url_clone) {
            eprintln!("Could not open browser: {e}");
            eprintln!("Open manually: {url_clone}");
        }
    });

    for req in server.incoming_requests() {
        let raw_path = req.url().to_string();
        // Strip query string, leading slash; default to index.html
        let path = raw_path
            .split('?')
            .next()
            .unwrap_or("/")
            .trim_start_matches('/')
            .to_string();
        let path = if path.is_empty() { "index.html".to_string() } else { path };

        match Assets::get(&path) {
            Some(asset) => {
                let mime = from_path(&path).first_or_octet_stream();
                let content_type =
                    Header::from_bytes("Content-Type", mime.as_ref()).unwrap();
                let data: Vec<u8> = asset.data.into_owned();
                let len = data.len();
                let response = Response::new(
                    tiny_http::StatusCode(200),
                    vec![content_type],
                    Cursor::new(data),
                    Some(len),
                    None,
                );
                let _ = req.respond(response);
            }
            None => {
                // Fall back to index.html for SPA-style routing
                if let Some(index) = Assets::get("index.html") {
                    let content_type =
                        Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap();
                    let data: Vec<u8> = index.data.into_owned();
                    let len = data.len();
                    let response = Response::new(
                        tiny_http::StatusCode(200),
                        vec![content_type],
                        Cursor::new(data),
                        Some(len),
                        None,
                    );
                    let _ = req.respond(response);
                }
            }
        }
    }
}
