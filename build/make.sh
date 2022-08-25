#!/bin/sh
SCPATH=$(cd $(dirname $0);pwd)
cat $SCPATH/head.html \
  <(ls -t1 public/images | xargs -I{} echo '    <img src="./images/{}">') \
  <(echo ' </body>') \
  <(echo '</html>') \
  > public/index.html
