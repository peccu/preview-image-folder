extern crate clap;
extern crate env_logger;
extern crate notify;
extern crate preview_image_folder;
extern crate ws;

use preview_image_folder::{files, spawn_server};
use std::thread::sleep;
use std::time::Duration;

use std::path::Path;

// v5
use notify::event::CreateKind;
use notify::event::DataChange;
use notify::event::ModifyKind;
use notify::EventKind::{Create, Modify, Remove};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use ws::{connect, CloseCode};

fn watch<P: AsRef<Path>>(path: P, url: String) -> notify::Result<()> {
    fn refresh(url: String) -> ws::Result<()> {
        connect(format!("ws://{}/ws", url), |out| {
            out.send("refresh!").unwrap();

            move |msg| {
                println!("Client got message '{}'. ", msg);
                out.close(CloseCode::Normal)
            }
        })
        .unwrap();
        Ok(())
    }

    fn refresh_by_event(event: notify::Event, url: String) {
        match event.kind {
            Create(CreateKind::File) => Some(refresh(url)),
            Modify(ModifyKind::Data(DataChange::Content)) => Some(refresh(url)),
            Remove(_) => Some(refresh(url)),
            _ => Some(Ok(())),
        };
        println!("changed: {:?}", event);
    }

    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    for res in rx {
        let client_url = url.clone();
        match res {
            Ok(event) => refresh_by_event(event, client_url),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

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
