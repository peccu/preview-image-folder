#!/bin/bash
nodemon \
  --watch public/images \
  -e png,jpg \
  --exec make.sh &

cd public
browser-sync start \
  --server \
  --no-open \
  --files './index.html'
