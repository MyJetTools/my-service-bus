FROM rust:slim
COPY ./target/release/my-service-bus-main-node ./target/release/my-service-bus-main-node
COPY ./wwwroot ./wwwroot 
ENTRYPOINT ["./target/release/my-service-bus-main-node"]
