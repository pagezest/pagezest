# this script is used to run tests

set -x  # print commands
set -e  # exit on error

export TOP="$PWD"

test_ui() {
  cd $TOP/admin
  npm install
  npm run build
}

test_backend() {
  cd $TOP
  cargo build --release
  cp $TOP/target/release/pagezest $TOP/build/
  cd $TOP/build/
  rm -rf $TOP/build/pagezest.db
  ./pagezest
}

# NOTE: for now, this compiles ONLY the toc (table of contents) plugin
test_plugins() {
  mkdir -p $TOP/build/plugins/toc-zig
  mkdir -p $TOP/build/plugins/toc
  mkdir -p $TOP/build/plugins/footer

  cd $TOP/plugins/toc
  zig build -p . --prefix-exe-dir .
  rm -rf $TOP/build/plugins/toc-zig/*
  cp -r $TOP/plugins/toc/manifest.json $TOP/build/plugins/toc-zig/
  cp -r $TOP/plugins/toc/toc.wasm $TOP/build/plugins/toc-zig/

  cd $TOP/plugins/toc-rust
  cargo build --release --target wasm32-unknown-unknown
  rm -rf $TOP/build/plugins/toc-rust/*
  cp -r $TOP/plugins/toc-rust/manifest.json $TOP/build/plugins/toc/
  cp -r $TOP/plugins/toc-rust/target/wasm32-unknown-unknown/release/toc.wasm $TOP/build/plugins/toc/toc.wasm

  cd $TOP/plugins/footer
  cargo build --release --target wasm32-unknown-unknown
  rm -rf $TOP/build/plugins/footer/*
  cp -r $TOP/plugins/footer/manifest.json $TOP/build/plugins/footer/
  cp -r $TOP/plugins/footer/target/wasm32-unknown-unknown/release/footer.wasm $TOP/build/plugins/footer/footer.wasm

  # This last cp is purely if you want to test the wasm in the browser.
  # cp -r $TOP/plugins/toc/toc.wasm $TOP/tests/test_wasm/test.wasm
}

main() {
  mkdir -p $TOP/build/

  case "$1" in
    ui) test_ui ;;
    b) test_backend ;;
    p) test_plugins ;;
    *)
      set +x
      echo "Invalid option: \`$1\`" >&2
      echo "Usage: $0"
      echo "  ui: run the ui"
      echo "  b: run the backend"
      echo "  p: run the plugins"
      ;;
  esac
}

main $@
