# A simple browser bevy game 

TODO:

    * Add more levels

# Build yourself

- install Rust🦀 and Python🐍

```shell
rustup target add wasm32-unknown-unknown
```

- you can whether build bin yourself or the wasm version by

```shell
wasm-pack build --target web --release
```

- additional

    - build parameters:
        * `--debug` for debug
        * `--profiling` optimized but with logs
        * `--release` full optimized but takes longer time

    - scaffolding files are already in root dir
    
    - just build and serve `index.html` to play 

# Simple Level Editor

- ⚠ by default the Win32 API is used to make window transparent

- install the dependencies in requirements.txt

```shell
## if you need virtual env
python3 -m venv .venv
source .venv/bin/activate

pip install -r requirements.txt
```

- run `simple_level_editor.py`

# ♥

> For my family.
