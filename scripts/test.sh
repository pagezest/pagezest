# this script is used to run tests

set -x  # print commands
set -e  # exit on error

export TOP="$PWD"

test_ui() {
  cd $TOP/admin
  npm install
  npm run build
  cd $TOP
  rm -rf $TOP/target/debug/pz-admin
  cp -r $TOP/admin/dist $TOP/target/debug/pz-admin
  cd $TOP/target/debug
  ./pagezest
}

test_backend() {
  cd $TOP
  cargo build
  cd $TOP/target/debug
  ./pagezest
}

main() {
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
