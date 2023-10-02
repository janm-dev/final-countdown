# Builder image
FROM rust:latest AS builder

# Install musl libc for static linking
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt upgrade -y && apt install -y musl-tools musl-dev

# Create user
ENV USER=countdown
ENV UID=1001

RUN adduser \
	--disabled-password \
	--gecos "" \
	--home "/nonexistent" \
	--shell "/sbin/nologin" \
	--no-create-home \
	--uid "${UID}" \
	"${USER}"

WORKDIR /countdown

COPY ./ .

# Install icu4x-datagen for cldr/icu data generation
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall icu_datagen --no-confirm

# Build with statically-linked musl libc
RUN cargo build --target x86_64-unknown-linux-musl --release

# Final image
FROM scratch

# Import from builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /countdown

# Copy the build
COPY --from=builder /countdown/target/x86_64-unknown-linux-musl/release/final-countdown ./

# Use an unprivileged user
USER countdown:countdown

# Expose HTTP port
EXPOSE 80

ENTRYPOINT [ "/countdown/final-countdown" ]
