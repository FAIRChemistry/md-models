FROM node:22-slim

WORKDIR /app

COPY tests/envs/typescript/ /app/
COPY tests/data/expected_typescript_zod.ts /app/generated.ts

RUN npm install
RUN npm install -g typescript
RUN tsc
RUN node test.js
