#!/usr/bin/env bats

setup() {
    load '../test/test_helper/bats-support/load'
    load '../test/test_helper/bats-assert/load'
}

@test "Setup" {
    run docker build -f test/Dockerfile -t resh-test .
    [ $status -eq 0 ]
    run docker run -d --name resh-test resh-test
    [ $status -eq 0 ]
}

@test "resh-test-toml-found" {
    run docker cp test/resh.toml resh-test:/etc/resh.toml
    [ $status -eq 0 ]
    run docker exec -it resh-test ./resh -c
    [ $status -ne 0 ]
    assert_output --partial 'a value is required for'
}

@test "resh-test-toml-global-command-valid" {
    run docker exec -it resh-test ./resh -c ls
    assert_output --partial 'root'
}

@test "resh-test-toml-global-command-unavailable" {
    run docker exec -it resh-test ./resh -c touch
    assert_output --partial 'Undefined command alias'
}

@test "resh-test-toml-global-user-command" {
    run docker exec -it resh-test ./resh -c lsa
    assert_output --partial '..'
}

@test "resh-test-toml-global-user-command-with-args" {
    run docker exec -it resh-test ./resh -c 'echo hello world!'
    assert_output --partial 'hello world!'
}

@test "resh-test-toml-global-user-command-overrides-global" {
    run docker exec -it resh-test ./resh -c 'foo'
    assert_output --partial 'bar override'
}

@test "Teardown" {
    teardown_once
}

teardown_once() {
    docker kill resh-test >/dev/null 2>&1
    docker rm resh-test >/dev/null 2>&1
}
