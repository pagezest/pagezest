set -e
set -x

TOP=$(realpath $(dirname $0))

cd "$TOP"

flatc --rust post.fbs
cp post_generated.rs ../src/post_flatbuffers.rs
rm -rf ../admin/src/buffers/pagezest-markdown*
cp -r pagezest-markdown* ../admin/src/buffers/
