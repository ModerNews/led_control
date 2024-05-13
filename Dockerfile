FROM rust:1-alpine as builder

WORKDIR /opt/app
COPY src /opt/app/src
COPY Cargo.toml /opt/app/Cargo.toml

RUN apk add --no-cache musl-dev
RUN cargo build --release

FROM alpine:latest as runtime

RUN apk add --no-cache bash curl sqlite 

COPY --from=builder /opt/app/target/release/led_control /usr/local/bin/led-control
RUN chmod +x /usr/local/bin/led-control
COPY ./default-config.yaml /etc/led-control/config.yaml

EXPOSE 8000 

ENTRYPOINT [ "/usr/local/bin/led-control" ]
CMD [ "" ]
