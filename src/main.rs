extern crate clap;
extern crate env_logger;
extern crate notify; // 5.0.0 looks good but not released. this is 4.0.17.
extern crate ws;

use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use clap::{App, Arg};
use notify::{watcher, RecursiveMode, Watcher};
use notify::DebouncedEvent::{Write, Create};
use ws::{connect, listen, CloseCode, Handler, Message, Request, Response, Result, Sender};

// This can be read from a file
static INDEX_HTML: &'static [u8] = br#"
<!DOCTYPE html>
<html>
	<head>
		<meta charset="utf-8">
	</head>
	<body>
      <pre id="messages"></pre>
			<form id="form">
				<input type="text" id="msg">
				<input type="submit" value="Send">
			</form>
      <script>
        var proto = !!location.protocol.match(/s:$/) ? "wss://" : "ws://";
        var socket = new WebSocket(proto + window.location.host + "/ws");
        socket.onmessage = function (event) {
          var messages = document.getElementById("messages");
          messages.append(event.data + "\n");
        };
        socket.onerror = function (event) {
            var messages = document.getElementById("messages");
            messages.append("error: " + JSON.stringify(event, null, 2) + "\n");  
        };
        var form = document.getElementById("form");
        form.addEventListener('submit', function (event) {
          event.preventDefault();
          var input = document.getElementById("msg");
          socket.send(input.value);
          input.value = "";
        });
		</script>
	</body>
</html>
    "#;

// Server web application handler
struct Server {
    out: Sender,
}

impl Handler for Server {
    //
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        // Using multiple handlers is better (see router example)
        match req.resource() {
            // The default trait implementation
            "/ws" => Response::from_request(req),

            // Create a custom response
            "/" => Ok(Response::new(200, "OK", INDEX_HTML.to_vec())),

            "/index.html" => Ok(Response::new(200, "OK", INDEX_HTML.to_vec())),

            _ => Ok(Response::new(404, "Not Found", b"404 - Not Found".to_vec())),
        }
    }

    // Handle messages recieved in the websocket (in this case, only on /ws)
    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Broadcast to all connections
        self.out.broadcast(msg)
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
                .help("Set the host to listen for web page."),
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
    let server = thread::spawn(move || listen(server_url, |out| Server { out }).unwrap());

    // Give the server a little time to get going
    sleep(Duration::from_millis(10));

    let target = matches.value_of("directory").unwrap();
    println!("watching: {}", target);

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
                    _ => Ok(())
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
