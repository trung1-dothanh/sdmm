#!/usr/bin/env just --justfile

VERSION := `cargo pkgid | sed 's/.*#//'`

css:
    npx @tailwindcss/cli -i ./res/css/tailwind_input.css -o ./res/css/tailwind_output.min.css --watch --minify

windows:
    cargo build --target=x86_64-pc-windows-gnu --release
    rm -rf output/windows
    mkdir -p output/windows/sdmm
    cp ../target/x86_64-pc-windows-gnu/release/sdmm.exe output/windows/sdmm/
    cp -r res output/windows/sdmm/
    cp sdmm-config-sample.ron output/windows/sdmm/sdmm.ron
    cd output/windows && zip -r sdmm_{{VERSION}}.windows.x86-64.zip sdmm && mv sdmm_{{VERSION}}.windows.x86-64.zip ..

linux:
    cargo build --target=x86_64-unknown-linux-musl --release
    rm -rf output/linux
    mkdir -p output/linux/sdmm
    cp ../target/x86_64-unknown-linux-musl/release/sdmm output/linux/sdmm/
    cp -r res output/linux/sdmm/
    cp sdmm-config-sample.ron output/linux/sdmm/sdmm.ron
    cd output/linux && tar cJvf sdmm_{{VERSION}}.linux.x86-64.tar.xz sdmm && mv sdmm_{{VERSION}}.linux.x86-64.tar.xz ..

release: windows linux
