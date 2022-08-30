#!/bin/sh
SCPATH=$(cd $(dirname $0);pwd)
cat $SCPATH/head.html \
  <(ls -t1 public/images | xargs -I{} echo '    <div class="item"><div class="name">{}</div><div class="image"><img src="./images/{}"/></div></div>') \
  <(echo ' </body>') \
  <(echo '</html>') \
  > public/index.html
