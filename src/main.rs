extern crate clap;
extern crate env_logger;
extern crate notify;
extern crate ws;
extern crate preview_image_folder;

use preview_image_folder::genpage;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
// use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use urlencoding::decode;

// v5
// use notify::{Watcher, RecursiveMode, Event};
// use notify::EventKind::{Create, Modify, Remove};
// v4
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use notify::{watcher, RecursiveMode, Watcher};
use ws::{connect, listen, CloseCode, Handler, Message, Request, Response, Sender};

fn list_files_by_reverse_modified(target: &str) -> Vec<fs::DirEntry> {
    let paths: fs::ReadDir = fs::read_dir(target).unwrap();
    let mut sorted = paths.filter_map(|e| e.ok()).collect::<Vec<fs::DirEntry>>();
    sorted.sort_by(|a, b| {
        b.metadata()
            .unwrap()
            .modified()
            .unwrap()
            .cmp(&a.metadata().unwrap().modified().unwrap())
    });
    sorted
}

fn vec_to_json(vec: Vec<fs::DirEntry>) -> Vec<u8> {
    let entries = vec
        .into_iter()
        .map(|e| e.file_name().into_string().ok())
        .map(|s| String::from(s.unwrap()))
        .collect::<Vec<String>>();

    // https://gist.github.com/jimmychu0807/9a89355e642afad0d2aeda52e6ad2424
    format!("[\"{}\"]", entries.join("\",\""))
        .as_bytes()
        .to_vec()
}

fn list_images(target: &str) -> Vec<u8> {
    vec_to_json(list_files_by_reverse_modified(target))
}

fn response_with_contenttype<R>(
    status: u16,
    reason: R,
    body: Vec<u8>,
    contenttype: Vec<u8>,
) -> Response
where
    R: Into<String>,
{
    let mut res = Response::new(status, reason, body);
    res.headers_mut().push(("Content-type".into(), contenttype));
    res
}

// Server web application handler
struct Server<'a> {
    out: Sender,
    target: &'a str,
}

impl Handler for Server<'_> {
    //
    fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
        // Using multiple handlers is better (see router example)
        match req.resource() {
            // The default trait implementation
            "/ws" => Response::from_request(req),

            // Create a custom response
            "/" => Ok(response_with_contenttype(
                200,
                "OK",
                genpage(),
                "text/html; charset=UTF-8".into(),
            )),

            "/index.html" => Ok(response_with_contenttype(
                200,
                "OK",
                genpage(),
                "text/html; charset=UTF-8".into(),
            )),

            "/images.json" => Ok(response_with_contenttype(
                200,
                "OK",
                list_images(self.target),
                "application/json; charset=UTF-8".into(),
            )),

            other => Ok(Response::new(
                200,
                "OK",
                read_file(format!("{}{}", self.target, decode(other).expect("UTF-8")).as_str()),
            )),
            // _ => Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec())),
        }
    }

    // Handle messages recieved in the websocket (in this case, only on /ws)
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        // Broadcast to all connections
        self.out.broadcast(msg)
    }
}

fn _read_file(name: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn read_file(name: &str) -> Vec<u8> {
    let path = env::current_dir();
    println!("pwd: {:?} -> {:?}", path, name);
    match _read_file(name) {
        Ok(buf) => buf,
        _ => b"Error".to_vec(),
    }
}

fn main() {
    // Setup logging
    env_logger::init();

    // Parse command line arguments
    let matches = preview_image_folder::parse_arg();

    let target = matches.value_of("directory").unwrap();
    println!("watching: {}", target);

    println!("{:?}", std::str::from_utf8(&list_images(target)).unwrap());

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
    let server = thread::spawn(move || {
        listen(server_url, |out| Server {
            out,
            target: &server_target,
        })
        .unwrap()
    });

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
