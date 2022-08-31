#!/bin/sh
SCPATH=$(cd $(dirname $0);pwd)
cd $SCPATH

mkdir -p /tmp/public/images

/app/preview-image-folder --host 0.0.0.0 /tmp/public/images &

while true
do
    rsync -az --delete $SCPATH/images/ /tmp/public/images/
    sleep 1
done
