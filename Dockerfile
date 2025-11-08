FROM rust:1.91.0

# ENV ROCKET_ADDRESS=0.0.0.0
# ENV ROCKET_PORT=8080

WORKDIR /app
COPY . .


# EXPOSE 8088

RUN cargo build

CMD ["cargo", "run"]
