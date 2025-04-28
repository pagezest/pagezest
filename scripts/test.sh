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

main() {
  case "$1" in
    ui) test_ui ;;
  esac
}

main $@
