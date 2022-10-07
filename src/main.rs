extern crate clap;
extern crate env_logger;
extern crate notify;
extern crate preview_image_folder;
extern crate ws;

use preview_image_folder::{files, spawn_server, watch};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Setup logging
    env_logger::init();

    // Parse command line arguments
    let app = preview_image_folder::AppParam::new();

    let target = app.get_target();
    println!("Watching: {}", target);
    println!(
        "Current file list: {:?}",
        std::str::from_utf8(&files::list_images(&target)).unwrap()
    );

    let url: String = app.get_url();
    println!();
    println!("Listening on http://{}/", url);
    println!("(If this is running in the container, you should change url)");

    // Server thread
    // Listen on an address and call the closure for each connection
    let server_url = url.clone();
    let server_target = target.clone();
    spawn_server(server_url, server_target);

    // Give the server a little time to get going
    sleep(Duration::from_millis(10));

    // watch files
    if let Err(e) = watch(target, url) {
        println!("error: {:?}", e)
    }
    println!("All done.")
}

// curl -Lo 1.png https://picsum.photos/200/300
// cargo run -- --host 0.0.0.0 ./src

// which rsync 2>/dev/null || apk add rsync
// mkdir -p /tmp/public/images
// while true; do rsync -az --delete ./images/
// cargo run -- --host 0.0.0.0 /tmp/public/images/
