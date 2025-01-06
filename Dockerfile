# Use the official Rust image as the base
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy Cargo files first to leverage Docker caching
COPY Cargo.toml Cargo.lock ./

# Create an empty src directory to run `cargo fetch`
RUN mkdir src

# Fetch dependencies
RUN cargo fetch

# Copy the rest of the application source code
COPY src ./src

# Build the application
RUN cargo build --release

# Define the command to run the app
CMD ["./target/release/my-leptos-app"]
