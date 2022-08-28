extern crate clap;
extern crate env_logger;

/// An example of a chat web application server
extern crate ws;
use ws::{listen, Handler, Message, Request, Response, Result, Sender};

use clap::{App, Arg};

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
                .value_name("DIRECTORIES")
                .help("Directories which include images.")
                .multiple(true),
        )
        .get_matches();

    let host = matches.value_of("host").unwrap();
    let port = matches.value_of("port").unwrap();
    println!(
        "Listening on http://{}:{}/ (If this is running in the container, you should change url)",
        host, port
    );
    // Listen on an address and call the closure for each connection
    listen(format!("{}:{}", host, port), |out| Server { out }).unwrap();
}
