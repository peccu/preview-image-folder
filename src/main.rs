extern crate clap;
extern crate env_logger;
extern crate notify; // 5.0.0 looks good but not released. this is 4.0.17.
extern crate ws;

use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use clap::{App, Arg};
use notify::DebouncedEvent::{Create, Write};
use notify::{watcher, RecursiveMode, Watcher};
use ws::{connect, listen, CloseCode, Handler, Message, Request, Response, Result, Sender};

// This can be read from a file
static INDEX_HTML_HEAD: &'static [u8] = br#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Preview images</title>
    <style>
      *, html, body {
          margin: 0px;
          padding: 0px;
      }
      h1 {
          padding: 1em 0px;
      }
      .image, .image img {
          max-width: 100%;
      }
      .name {
          margin-bottom: 5px;
          padding-left: 3px;
          border-left: solid 10px gray;
          border-bottom: solid 1px gray;
      }
      .item {
          margin-bottom: 15px;
      }
    </style>
</head>
<body>
    <h1>Preview Images in folder.</h1>

      <pre id="messages"></pre>
      <form id="form">
        <input type="text" id="msg">
        <input type="submit" value="Send">
      </form>
            <div id="images"><div>
    "#;

static INDEX_HTML_TAIL: &'static [u8] = br#"
      <script>
        const append = (message) => {
            var messages = document.getElementById("messages");
            messages.append(message);
        };
        var proto = !!location.protocol.match(/s:$/) ? "wss://" : "ws://";
        var socket = new WebSocket(proto + window.location.host + "/ws");
        socket.onmessage = function (event) {
          append(event.data + "\n");
          fetch_images();
        };
        socket.onerror = function (event) {
            append("error: " + JSON.stringify(event, null, 2) + "\n")
        };
        var form = document.getElementById("form");
        form.addEventListener('submit', function (event) {
          event.preventDefault();
          var input = document.getElementById("msg");
          socket.send(input.value);
          input.value = "";
        });
        var show_images = (data) => {
            append("images: " + JSON.stringify(data, null, 2) + "\n")
            var list = data.filter(e=>e.match(/\.png$/)).map(e=>`<div class="item"><div class="name">${e}</div><div class="image"><img src="${e}"/></div></div>`).join("\n")
            console.log(list);
            var images = document.getElementById("images");
            images.innerHTML = list;
        };
        var images = fetch("./images.json").then(r => r.json()).then(show_images)
        var fetch_images = () => {
            fetch("./images.json")
            .then((response) => response.json())
            .then((data) => {
                console.log(data);
                show_images(data);
            });
        }
    </script>
  </body>
</html>
    "#;

fn genpage() -> Vec<u8> {
    [INDEX_HTML_HEAD, INDEX_HTML_TAIL].concat().to_vec()
}

fn list_images(target: &str) -> Vec<u8> {
    // https://stackoverflow.com/a/31226040
    let entries = fs::read_dir(target)
        .unwrap()
        .filter_map(|res| {
            res.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
            })
        })
        .collect::<Vec<String>>();
    // TODO sort by descending

    // https://gist.github.com/jimmychu0807/9a89355e642afad0d2aeda52e6ad2424
    format!("[\"{}\"]", entries.join("\",\""))
        .as_bytes()
        .to_vec()
}

// Server web application handler
struct Server<'a> {
    out: Sender,
    target: &'a str,
}

impl Handler for Server<'_> {
    //
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        // Using multiple handlers is better (see router example)
        match req.resource() {
            // The default trait implementation
            "/ws" => Response::from_request(req),

            // Create a custom response
            "/" => Ok(Response::new(200, "OK", genpage())),

            "/index.html" => Ok(Response::new(200, "OK", genpage())),

            "/images.json" => Ok(Response::new(200, "OK", list_images(self.target))),

            other => Ok(Response::new(
                200,
                "OK",
                read_file(format!(".{}", other).as_str()),
            )),
            // _ => Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec())),
        }
    }

    // Handle messages recieved in the websocket (in this case, only on /ws)
    fn on_message(&mut self, msg: Message) -> Result<()> {
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
    let matches = App::new("Preview Image Folder with auto refresh.")
        .version("0.1")
        .about("Show images in specified folder and refresh when images in folder is updated.")
        .arg(
            Arg::with_name("host")
                .long("host")
                .default_value("127.0.0.1")
                .value_name("HOST")
                .help("Set the host to listen for web page. If you use in container, you should set this into 0.0.0.0"),
        )
        .arg(
            Arg::with_name("port")
                .short('p')
                .long("port")
                .default_value("8000")
                .value_name("PORT")
                .help("Set the port to listen for web page."),
        )
        .arg(
            Arg::with_name("directory")
                .value_name("DIRECTORY")
                .default_value(".")
                .help("Directory which include images."), //.multiple(true),
        )
        .get_matches();

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

    let (sender, receiver) = channel();
    let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();
    watcher.watch(target, RecursiveMode::NonRecursive).unwrap();

    // send refresh message
    fn refresh(url: String) -> Result<()> {
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

    let client = thread::spawn(move || loop {
        match receiver.recv() {
            Ok(event) => {
                println!("event {:?}", event);
                let client_url = url.clone();
                match event {
                    Create(_) => refresh(client_url),
                    Write(_) => refresh(client_url),
                    _ => Ok(()),
                };
            }
            Err(e) => println!("watch error: {:?}", e),
        };
    });

    let _ = server.join();
    let _ = client.join();

    println!("All done.")
}

// curl -Lo 1.png https://picsum.photos/200/300
