
all: build

build:
	cargo build --examples

results:
	cd maelstrom && ./maelstrom serve

echo:
	cd maelstrom && ./maelstrom test -w echo --bin ../target/debug/examples/echo --node-count 5 --time-limit 10 --rate 50

unique-ids:
	cd maelstrom && ./maelstrom test -w unique-ids --bin ../target/debug/examples/unique-ids --time-limit 10 --rate 15000 --node-count 3 --availability total --nemesis partition

broadcast:
	cd maelstrom && ./maelstrom test -w broadcast --bin ../target/debug/examples/broadcast --node-count 1 --time-limit 10 --rate 30

broadcast-b:
	cd maelstrom && ./maelstrom test -w broadcast --bin ../target/debug/examples/broadcast --node-count 5 --time-limit 10 --rate 10

broadcast-c:
	cd maelstrom && ./maelstrom test -w broadcast --bin ../target/debug/examples/broadcast --node-count 5 --time-limit 10 --rate 10 --nemesis partition

broadcast-d:
	cd maelstrom && ./maelstrom test -w broadcast --bin ../target/debug/examples/broadcast --node-count 25 --time-limit 10 --rate 100 --latency 100 
broadcast-d2:
	cd maelstrom && ./maelstrom test -w broadcast --bin ../target/debug/examples/broadcast --node-count 25 --time-limit 10 --rate 100 --latency 100 --nemesis partition