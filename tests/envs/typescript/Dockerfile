FROM node:22-slim

WORKDIR /app

COPY tests/envs/typescript/ /app/

RUN npm install
RUN npm install -g typescript
RUN tsc
RUN node test.js
