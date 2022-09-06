# preview-image-folder


## local version

Start server by

```
cd build
npm i
npm run dev
```

 and open http://localhost:3000 or other browser-sync outputs.

Then, you can add images into ./images and the browser should be auto-reloaded and show images in the directory.

You can change directory or image extensions at `start.sh`

## docker version

You can use docker image [peccu/preview-image-folder](https://hub.docker.com/r/peccu/preview-image-folder) from Docker Hub.

You need to mount the image folder into `/app/images` and listening port is 8000.

```
docker run --name preview-image-folder --rm -d -p 8000:3000 -v $(pwd):/app/images peccu/preview-image-folder:latest
```

## docker-compose version

change local image path in the `docker-compose.yml`, and the run these command.

```
docker-compose up -d
```

## Using

watch directory by nodemon and reload browser by browser-sync.
