# Zero2prod

## Prerequisites
* rust, postgres, docker, sqlx
* install sqlx: 
```
cargo install sqlx-cli --no-default-features \                                          
features rustls,postgres
```

## Install and Run locally

initialize the postgres database and redis session store
```
scripts/init_db.sh 
scripts/init_redis.sh

```
build the project: 
```
cargo build
```

run locally
```
cargo run
```

## Run test suite
``` cargo test```
to test and print tracing logs:
```TEST_LOG=true RUST_LOG="sqlx=error,info" cargo test | bunyan```

## Install and Run using Docker
```
docker build --tag zero2prod --file Dockerfile .
docker run -p 8000:8000 zero2prod
```

## Query Locally
### HealthCheck
```
curl -v "127.0.0.1:8000/health_check"
```

### New Subscription
```
curl --request POST \                                                       
  -d "name=john&email=johndoe@email.com" \
  127.0.0.1:8000/subscriptions --verbose
```

## Doc to be Added:
* Change Password
* Publish Newsletter