# preview-image-folder

Preview images in folder with auto refresh when the new images added.

Main usecase is to check screen captured image.

## install

- download package from release page
- extract binary
- add it to PATH or call it with relative path
- `preview-image-folder --help` shows help

## local version

Start server by

```
cargo run -- --host 0.0.0.0 ./images
```

 and open http://localhost:8000 .

Then, you can add images into ./images and the browser should be auto-reloaded and show images in the directory.

You can change directory or image extensions by argument `--port` and last aragument.

Please see help for detail by `cargo run -- --help`.

## docker version

You can use docker image [peccu/preview-image-folder](https://hub.docker.com/r/peccu/preview-image-folder) from Docker Hub.

You need to mount the image folder into `/app/images` and listening port is 8000.

```
docker run --name preview-image-folder --rm -d -p 8000:8000 -v $(pwd):/app/images peccu/preview-image-folder:latest
```

## docker-compose version

change local image path in the `docker-compose.yml`, and the run these command.

```
docker-compose up -d
```
