FROM node:16-alpine

RUN apk add --update --no-cache rsync
WORKDIR /app

ADD package.json ./package.json
RUN npm i

ADD . ./
RUN chmod 755 *.sh

VOLUME /app/images
CMD ["npm", "run", "dev"]
