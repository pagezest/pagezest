set -e
set -x

TOP=$(realpath $(dirname $0))

cd "$TOP"

rm -rf pagezest-markdown.ts pagezest-markdown
rm -rf $TOP/../admin/src/buffers/pagezest-markdown*
flatc --rust post.fbs
flatc --ts post.fbs
cp post_generated.rs $TOP/../src/post_flatbuffers.rs
cp -r pagezest-markdown* $TOP/../admin/src/buffers/
