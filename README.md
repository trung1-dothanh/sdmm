<div align="center">

Stable Diffusion Models Manager
===============================

[![pipeline status](https://gitlab.com/kimtinh/sdmm/badges/master/pipeline.svg)](https://gitlab.com/kimtinh/sdmm/-/commits/master)

[![Gitlab](https://img.shields.io/badge/gitlab-%23181717.svg?style=for-the-badge&logo=gitlab&logoColor=white)](https://gitlab.com/kimtinh/sdmm)
[![Github](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/trung1-dothanh/sdmm)

  
[![demo video](https://img.youtube.com/vi/85oTHZkGkZU/maxresdefault.jpg)](https://youtu.be/85oTHZkGkZU)

</div>

Standalone application to manage your local Stable Diffusion models. This app is web-based so you can run it on
your cloud machine like runpod.

Features:
* [x] Manage model with tag.
* [x] Get preview image and model info from Civitai by hash.
* [x] Download from Civitai

How to run
----------

See the sample config at [sdmm-config-sample.ron](./sdmm-config-sample.ron) and update to your need.

Run the web server:
```shell
./sdmm -c ./path/to/config.ron
```

> Note: Put the [res](./res) folder in same directory with binary `sdmm`.

Now you can access it at http://localhost:9696 or http://your_ip_address:9696

How to build
------------

Get the prebuilt binary in Release page or build it with `cargo`.

Update CSS:
```shell
cd res
npm install tailwindcss @tailwindcss/cli 
npx @tailwindcss/cli -i ./css/tailwind_input.css -o ./css/tailwind_output.min.css --build --minify
```

Migrate database:
```shell
# Create sqlite db file if not exist (only used for building)
touch sdmm.sqlite

sqlx migrate run
```

Build the application

* Normal build (for running on the same machine):
    ```shell
    cargo build --release
    ```
    Output: `target/release/sdmm`.

* Statically build for Linux so you can copy binary to another machine and run without worrying about dependencies:
    ```shell
    rustup target add x86_64-unknown-linux-musl    # run once if not installed
    cargo build --target=x86_64-unknown-linux-musl --release
    ```
    Output: `target/x86_64-unknown-linux-musl/release/sdmm`

* Cross build in Linux for Windows target:
    ```shell
    rustup target add x86_64-pc-windows-gnu    # run once if not installed
    cargo build --target=x86_64-pc-windows-gnu --release
    ```

------

<div align="center">

![sdmm](https://count.getloli.com/@git_sdmm?name=git_sdmm&theme=random&padding=9&offset=0&align=top&scale=1&pixelated=1&darkmode=auto)

</div>
