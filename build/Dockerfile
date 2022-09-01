FROM alpine:latest

RUN apk add --update --no-cache rsync
ENV version=TEST-0.0.10
ADD https://github.com/peccu/preview-image-folder/releases/download/${version}/preview-image-folder-${version}-x86_64-unknown-linux-musl.tar.gz /app/

WORKDIR /app
RUN tar xzf preview-image-folder-${version}-x86_64-unknown-linux-musl.tar.gz \
        && mv preview-image-folder-${version}-x86_64-unknown-linux-musl/preview-image-folder ./ \
        && rm -rf preview-image-folder-${version}-x86_64-unknown-linux-musl.tar.gz preview-image-folder-${version}-x86_64-unknown-linux-musl
ADD start.sh /app/

VOLUME /app/images
CMD ["./start.sh"]
