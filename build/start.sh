#!/bin/sh
SCPATH=$(cd $(dirname $0);pwd)
cd /tmp
mkdir -p public/images
nodemon \
  --watch public/images \
  -e png,jpg \
  --exec $SCPATH/make.sh &

cd public
browser-sync start \
  --server \
  --no-open \
  --files './index.html' &

while true
do
    rsync -az --delete $SCPATH/images/ /tmp/public/images/
    sleep 1
done
