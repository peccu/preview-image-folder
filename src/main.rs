extern crate clap;
extern crate env_logger;
extern crate notify;
extern crate preview_image_folder;
extern crate ws;

use preview_image_folder::{files, spawn_server};
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

// v5
// use notify::{Watcher, RecursiveMode, Event};
// use notify::EventKind::{Create, Modify, Remove};
// v4
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use notify::{watcher, RecursiveMode, Watcher};
use ws::{connect, CloseCode};

fn main() {
    // Setup logging
    env_logger::init();

    // Parse command line arguments
    let matches = preview_image_folder::parse_arg();

    let target = matches.value_of("directory").unwrap();
    println!("watching: {}", target);

    println!(
        "{:?}",
        std::str::from_utf8(&files::list_images(target)).unwrap()
    );

    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap();
    let url: String = format!("{}:{}", host, port);
    let server_url = url.clone();
    println!(
        "Listening on http://{}/ (If this is running in the container, you should change url)",
        server_url
    );

    // Server thread
    // Listen on an address and call the closure for each connection
    let server_target = format!("{}", target);
    let server = spawn_server(server_url, server_target);

    // Give the server a little time to get going
    sleep(Duration::from_millis(10));

    // send refresh message
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

    // v5
    // let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
    //     println!("event {:?}", res);
    //     res.ok().and_then(|event| {
    //         println!("event {:?}", event);
    //         let client_url = url.clone();
    //         match event.kind {
    //             Create(_) => Some(refresh(client_url)),
    //             Modify(_) => Some(refresh(client_url)),
    //             Remove(_) => Some(refresh(client_url)),
    //             _ => Some(Ok(())),
    //         }
    //     }).unwrap().unwrap()
    // }).unwrap();
    // let client_target = format!("{}", target);
    // let client = thread::spawn(move || {
    //     watcher.watch(Path::new(&client_target), RecursiveMode::Recursive).unwrap();
    // });

    // v4
    let (sender, receiver) = channel();
    let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();
    watcher.watch(target, RecursiveMode::NonRecursive).unwrap();
    let client = thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => {
                println!("event {:?}", event);
                let client_url = url.clone();
                match event {
                    Create(_) => refresh(client_url),
                    Write(_) => refresh(client_url),
                    Remove(_) => refresh(client_url),
                    Rename(_, _) => refresh(client_url),
                    _ => Ok(()),
                }
                .unwrap()
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    });

    let _ = server.join();
    let _ = client.join();

    println!("All done.")
}

// curl -Lo 1.png https://picsum.photos/200/300
// cargo run -- --host 0.0.0.0 ./src
