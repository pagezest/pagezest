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

main() {
  mkdir -p $TOP/build/
  mkdir -p $TOP/build/plugins/
  cp $TOP/plugins/page.json $TOP/build/plugins/page.json

  case "$1" in
    ui) test_ui ;;
    b) test_backend ;;
    *)
      set +x
      echo "Invalid option: \`$1\`" >&2
      echo "Usage: $0"
      echo "  ui: run the ui"
      echo "  b: run the backend"
      ;;
  esac
}

main $@
