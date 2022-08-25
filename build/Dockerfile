FROM node:16-alpine

RUN apk add --update --no-cache rsync
ADD . /app/

WORKDIR /app
RUN chmod 755 *.sh
RUN npm i

VOLUME /app/images
CMD ["npm", "run", "dev"]
