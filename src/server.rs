use std::thread;
use urlencoding::decode;
use ws::{listen, Handler, Message, Request, Response, Sender};

use crate::files;
use crate::page::genpage;

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
                files::list_images(self.target),
                "application/json; charset=UTF-8".into(),
            )),

            other => Ok(Response::new(
                200,
                "OK",
                files::read_file(
                    format!("{}{}", self.target, decode(other).expect("UTF-8")).as_str(),
                ),
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

pub fn spawn_server(url: String, target: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        listen(url, |out| Server {
            out,
            target: &target,
        })
        .unwrap()
    })
}
