./jtd-codegen board_state.jtd.json --typescript-out .
mv index.ts ../blokus-ui/src/board_state.ts

./jtd-codegen board_state.jtd.json --rust-out ../server/src/board_state/