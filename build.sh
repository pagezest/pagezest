#!/bin/bash

set -e
set -x


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
cargo build --release --no-default-features
rm -rf ./docker/pagezest
cp target/release/pagezest ./docker/

pushd admin
npm run build
popd

rm -rf docker/pz-admin/
cp -r admin/dist/ docker/pz-admin/
rm -rf docker/assets/
cp -r assets/ docker/assets


mkdir -p docker/plugins/toc
cp plugins/toc/*.{wasm,json} docker/plugins/toc


docker build -t pagezest/pagezest docker

docker push pagezest/pagezest
docker image tag pagezest/pagezest pagezest/pagezest:v0.4
docker image push --all-tags pagezest/pagezest
