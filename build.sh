#!/bin/bash

tag=
while [[ $# -gt 0 ]]; do
  case $1 in
    -t)
      tag="$2"
      shift 2
      ;;
    *)
      echo "Invalid option: $1" >&2
      usage
      ;;
  esac
done

echo "tag => $tag"

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

docker push pagezest/pagezest
docker image tag pagezest/pagezest pagezest/pagezest:v0.1
docker image push --all-tags pagezest/pagezest
