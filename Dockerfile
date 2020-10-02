FROM alpine:latest

COPY target/debug/warp-ws-example /entrypoint

RUN chmod +x /entrypoint

EXPOSE 3030

ENTRYPOINT /entrypoint