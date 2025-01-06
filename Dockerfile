# Use an official Rust image for development
FROM rust:latest

# Install necessary tools
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Set the working directory
WORKDIR /app

# Copy only the Cargo.toml and Cargo.lock for caching
COPY Cargo.toml Cargo.lock ./

# Build dependencies to speed up subsequent builds
RUN cargo build --release

# Copy the source code
COPY . .

RUN cargo install cargo-watch
CMD ["cargo", "watch", "-x", "run"]

# Build the app
RUN cargo build --release

# Expose the port Leptos will use (default is 3000)
EXPOSE 3000

# Command to run the app
CMD ["cargo", "run", "--release"]
