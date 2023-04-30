./jtd-codegen game.jtd.json --typescript-out .
mv index.ts ../blokus-ui/src/game.ts

./jtd-codegen game.jtd.json --rust-out ../server/src/game/
