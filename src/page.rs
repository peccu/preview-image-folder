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
      .image {
          text-align: center;
      }
      .image img {
          max-width: 100%;
          max-height: 90vh;
      }
      .name {
          margin-bottom: 5px;
          padding-left: 3px;
          border-left: solid 10px gray;
          border-bottom: solid 1px gray;
          max-height: 10vh;
      }
      .item {
          margin-bottom: 15px;
      }
    </style>
</head>
<body>
    <h1>Preview Images in folder.</h1>
    <pre id="messages"></pre>
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
          // append(event.data + "\n");
          fetch_images();
        };
        socket.onerror = function (event) {
            append("error: " + JSON.stringify(event, null, 2) + "\n")
        };
        var show_images = (data) => {
            // append("images: " + JSON.stringify(data, null, 2) + "\n")
            var list = data
                .filter(e=>e.match(/\.png$/))
                .map(e=>`
        <div class="item" id="${e}">
            <div class="name">${e}</div>
            <div class="image">
                <img alt="${e}" src="${e}"/>
            </div>
        </div>`).join("\n");
            console.log(list);
            var images = document.getElementById("images");
            images.innerHTML = list;
        };
        var scrollToFirst = (data) => {
            var list = data.filter(e=>e.match(/\.png$/));
            document.getElementById(list[0]).scrollIntoView()
        };
        var images = [];
        var fetch_images = () => {
            fetch("./images.json")
            .then((response) => response.json())
            .then((data) => {
                images = data;
                console.log(data);
                show_images(data);
                scrollToFirst(data);
            });
        };
        fetch_images();
    </script>
  </body>
</html>
"#;

pub fn genpage() -> Vec<u8> {
    [INDEX_HTML_HEAD, INDEX_HTML_TAIL].concat().to_vec()
}
