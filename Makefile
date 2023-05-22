
all: build

build: 
	cargo build
	cargo build --examples

results:
	cd maelstrom && ./maelstrom serve

echo:
	cd maelstrom && ./maelstrom test -w echo --bin ../target/debug/examples/echo --node-count 5 --time-limit 10

unique-ids:
	cd maelstrom && ./maelstrom test -w unique-ids --bin ../target/debug/examples/unique-ids --time-limit 10 --rate 15000 --node-count 3 --availability total --nemesis partition