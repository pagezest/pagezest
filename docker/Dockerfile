#FROM alpine:3.21.3
FROM debian:bookworm as base
RUN apt-get update && apt-get install -y libsqlite3-dev

FROM base AS app
WORKDIR /app

COPY ./pagezest /app/pagezest
COPY ./plugins/ /app/plugins
COPY ./pz-admin/ /app/pz-admin

CMD ["./pagezest"]
