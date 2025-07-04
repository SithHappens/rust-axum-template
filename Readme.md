
# Dev Setup

## Application

```shell
cargo watch -q -c -w src/ -w .cargo/ -x "run"
```

## QuickDev

```shell
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```
or

```shell
cargo watch -q -c -w src/ -w tests/ -x "test --target-dir target/tests -q quick_dev -- --nocapture"
```

## Test

```shell
cargo watch -q -c -x "test -- --nocapture"

```
more specific
```shell
cargo watch -q -c -x "test test_create_ok -- --nocapture"

```

## Database

```shell
docker run --rm --name pg -p 5432:5432 -e POSTGRES_PASSWORD=welcome postgres:15
```

start a psql terminal for postgres:
```shell
docker exec -it -u postgres pg psql
```
