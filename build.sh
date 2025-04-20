#!/bin/bash

export LLVM_LIB_CFG_PATH=${LLVM_LIB_CFG_PATH:-/usr/lib/llvm-16/lib}
cargo build --release
cp target/release/pagezest docker

pushd admin
npm run build
popd

rm -rf docker/pz-admin/
cp -r admin/dist/ docker/pz-admin/

cp plugins/*.wasm docker/plugins/
cp plugins/*.json docker/plugins/

docker build -t pagezest docker
