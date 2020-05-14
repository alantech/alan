#!/bin/bash

# Implement readlink which does not work on mac os https://stackoverflow.com/a/1116890
# This gives us flexibility to change the Java code and recompile a jar that we can test via `./alan`
# without blowing away our stable, globally-installed version
TARGET_FILE=$0

ORIG_DIR=`pwd -P`

cd `dirname $TARGET_FILE`
TARGET_FILE=`basename $TARGET_FILE`

# Iterate down a (possible) chain of symlinks
while [ -L "$TARGET_FILE" ]
do
    TARGET_FILE=`readlink $TARGET_FILE`
    cd `dirname $TARGET_FILE`
    TARGET_FILE=`basename $TARGET_FILE`
done

# Compute the canonicalized name by finding the physical path
# for the directory we're in and appending the target file.
PHYS_DIR=`pwd -P`

# Restore the original directory so the script referenced can be found
# cd $ORIG_DIR

PATH="${PATH}:${PHYS_DIR}:${PHYS_DIR}/../shellspec:${PHYS_DIR}/../node_modules/.bin:${PHYS_DIR}/../runtime/target/release:${PHYS_DIR}/../interpreter"
export PATH
echo $PATH
shellspec

exit $?
