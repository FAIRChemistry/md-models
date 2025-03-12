FROM golang:1.23-alpine

WORKDIR /app

COPY tests/data/expected_golang.go /app/generated.go
COPY tests/envs/go/ /app/

RUN go test -v