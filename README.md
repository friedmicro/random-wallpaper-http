# random-wallpaper-http

A small http server which will send a random image from a folder when requested. Supports requesting a minimum size.

## Build\Run
To build\run: `cargo build --release; cp ./target/release/random-wallpaper-http; ./random-wallpaper-http`.

## API
Responds to POST requests on /image, for example:
curl -X POST `http://localhost:3500/image -H "Content-Type: application/json" -d '{"min_x": 2560, "min_y": 1440, "device": "desktop"}' > ./Wallpaper.jpg`
