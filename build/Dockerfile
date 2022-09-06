FROM alpine:latest

RUN apk add --update --no-cache rsync
ENV version=TEST-0.0.11
ENV APP=preview-image-folder
ENV PKG=${APP}-${version}-x86_64-unknown-linux-musl
ADD https://github.com/peccu/${APP}/releases/download/${version}/${PKG}.tar.gz /app/

WORKDIR /app
RUN tar xzf ${PKG}.tar.gz \
        && mv ${PKG}/${APP} ./ \
        && rm -rf ${PKG}.tar.gz ${PKG}
COPY start.sh /app/
RUN chmod +x start.sh

ADD . ./
RUN chmod 755 *.sh

VOLUME /app/images
EXPOSE 8000
CMD ["./start.sh"]
