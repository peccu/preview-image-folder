# preview-image-folder

Start server by

```
cd build
npm i
npm run dev
```

 and open http://localhost:3000 or other browser-sync outputs.

Then, you can add images into ./images and the browser should be auto-reloaded and show images in the directory.

You can change directory or image extensions at `start.sh`

## docker-compose version

change local image path in the `docker-compose.yml`, and the run these command.

```
docker-compose up -d
```

## Using

watch directory by nodemon and reload browser by browser-sync.
