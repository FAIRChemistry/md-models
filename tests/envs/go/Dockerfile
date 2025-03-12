FROM golang:1.23-alpine

WORKDIR /app

COPY tests/envs/go/ /app/

RUN go test -v