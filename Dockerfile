FROM rust:1.65 as builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=builder /app/target/release/your-leptos-app /usr/local/bin/

CMD ["your-leptos-app"]

EXPOSE 3000
