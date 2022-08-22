#!/bin/bash
cat head.html \
  <(ls -t1 public/images | xargs -I{} echo '    <img src="./images/{}">') \
  <(echo ' </body>') \
  <(echo '</html>') \
  > public/index.html
