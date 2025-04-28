# this script is used to run tests

set -x  # print commands
set -e  # exit on error

export TOP="$PWD"

test_ui() {
  cd $TOP/admin
  npm install
  npm run build
  cd $TOP
  rm -rf $TOP/build/pz-admin/
  cp -r $TOP/admin/dist/ $TOP/build/pz-admin/
  cd $TOP/build
  ./pagezest
}

test_backend() {
  cd $TOP
  cargo build
  cp $TOP/target/debug/pagezest $TOP/build/
  cd $TOP/build/
  ./pagezest
}

# NOTE: for now, this compiles ONLY the toc (table of contents) plugin
test_plugins() {
  mkdir -p $TOP/build/plugins/
  cd $TOP/plugins/
  npm install
  npx asc toc/toc.ts --target release --exportRuntime --exportTable --outFile toc/toc.wasm
  cp $TOP/plugins/page.json $TOP/build/plugins/page.json
  rm -rf $TOP/build/plugins/toc/
  cp -r $TOP/plugins/toc/ $TOP/build/plugins/toc/

  # This last cp is purely if you want to test the wasm in the browser.
  cp -r $TOP/plugins/toc/toc.wasm $TOP/tests/test_wasm/test.wasm
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
