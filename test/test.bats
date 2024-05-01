#!/usr/bin/env bats

setup() {
    load '../test/test_helper/bats-support/load'
    load '../test/test_helper/bats-assert/load'
}

exit_if_fail() {
    if [ ! -z "${TEST_FAILURE}" ]; then
        echo "Previous test failed. Aborting."
        teardown_once
        exit 1
    fi
}

@test "Setup" {
    # Define docker-compose.yml using a Heredoc
    dockerfile=$(
        cat <<'EOF'
FROM rust:alpine as builder
WORKDIR /usr/src
RUN apk add --no-cache musl-dev

# The standard way to build
#COPY . .
#RUN cargo build --release

# A workaround to avoid redownloading and recompling everything when only the source code changes
# This is in leui of a cargo build --deps-only command
COPY ./Cargo.toml ./Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {println!(\"if you see this, the build failed\")}" > src/main.rs \
    && cargo build
RUN rm src/main.rs

# Copy the source code and build the release binary
COPY ./src ./src
RUN cargo build --release

FROM alpine:latest
WORKDIR /root
COPY --from=builder /usr/src/target/release/resh .
CMD tail -f /dev/null
EOF
    )

    # Write the config to a docker-compose.yml file
    echo "$dockerfile" >Dockerfile
}

@test "Cache" {
    # Cache the Docker images
    run docker pull alpine:latest
    [ $status -eq 0 ]
    run docker pull rust:alpine
    [ $status -eq 0 ]
    exit_if_fail
}

@test "Build" {
    run docker build -t resh-test .
    [ $status -eq 0 ]
    exit_if_fail
}

@test "Up" {
    run docker run -d --name resh-test resh-test
    [ $status -eq 0 ]
    exit_if_fail
}

@test "resh-test-finds-toml" {
    resh_toml=$(
        cat <<'EOF'
[commands]
ls = "ls -l"
[user_commands.root]
lsa = "ls -lah"
EOF
    )

    echo "$resh_toml" >resh.toml
    run docker cp resh.toml resh-test:/etc/resh.toml
    [ $status -eq 0 ]
    run docker exec -it resh-test ./resh -c
    [ $status -ne 0 ]
    assert_output --partial 'a value is required for'
}

@test "resh-test-toml-global-valid-command" {
    run docker exec -it resh-test ./resh -c ls
    assert_output --partial 'root'
}

@test "resh-test-toml-global-unavailable-command" {
    run docker exec -it resh-test ./resh -c touch
    assert_output --partial 'Undefined command alias'
}

@test "resh-test-toml-global-user-command" {
    run docker exec -it resh-test ./resh -c lsa
    assert_output --partial '..'
}

@test "Teardown" {
    teardown_once
}

teardown_once() {
    rm Dockerfile >/dev/null 2>&1
    rm resh.toml >/dev/null 2>&1
    docker kill resh-test >/dev/null 2>&1
    docker rm resh-test >/dev/null 2>&1
}
