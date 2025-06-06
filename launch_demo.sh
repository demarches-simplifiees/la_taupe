NODE_LOG=/tmp/node_log
LA_TAUPE_LOG=/tmp/la_taupe_log

echo 'compiling la_taupe'
cargo build --release

echo 'building simple node server which mimics a backend storage'
npm install --prefix tests/fixtures/static_server

echo 'launching fake backend storage with node listenning on 3333'
DEBUG=express:* node tests/fixtures/static_server/server.js > "$NODE_LOG" 2>&1 &

echo 'launching la_taupe listenning on 8080'
RUST_LOG=info ./target/release/la_taupe > "$LA_TAUPE_LOG" 2>&1 &

sleep 2

echo 'fetching a demo rib'
curl -X POST localhost:8080/analyze -H 'Content-Type: application/json' -d '{"url": "http://localhost:3333/RIB_AMU.pdf", "hint": {"type": "rib"}}'

wait
