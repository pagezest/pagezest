#FROM alpine:3.21.3
FROM debian:bookworm as base
RUN apt-get update && apt-get install -y libsqlite3-dev

FROM base AS app
WORKDIR /app

COPY ./pagezest /app/pagezest

FROM app AS plugins

COPY ./plugins/ /app/plugins

FROM plugins AS admin
COPY ./pz-admin/ /app/pz-admin
COPY ./assets/ /app/assets

CMD ["./pagezest"]
