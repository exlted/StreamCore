FROM node:16-alpine

WORKDIR /usr/src/app
COPY package*.json ./
RUN npm ci --only=production

COPY . .

CMD [ "node", "src/main.js"]