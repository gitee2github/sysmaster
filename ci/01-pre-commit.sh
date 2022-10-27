#!/bin/bash
# try install 3 times

function finish() {
    echo "--- PLEASE RUN sh -x ci/01-pre-commit.sh FIRST IN YOUR LOCALHOST!!! ---"
    # remove tmp
    set +x
    for rustlist in `git diff origin/master --stat | awk '{print $1}' | grep \.rs$ | tr '\n' ' '`
    do
        sed -i '/#!\[deny(missing_docs)]/d' $rustlist 2>/dev/null || true
        sed -i '/#!\[deny(clippy::all)]/d' $rustlist 2>/dev/null || true
        sed -i '/#!\[deny(warnings)]/d' $rustlist 2>/dev/null || true
    done
}

trap finish EXIT

for rustlist in `git diff origin/master --stat | awk '{print $1}' | grep \.rs$ | tr '\n' ' '`
do
    grep -P '[\p{Han}]' $rustlist  && exit 1
done

pip3 install pre-commit -i http://mirrors.aliyun.com/pypi/simple/ || pip3 install  -i https://pypi.tuna.tsinghua.edu.cn/simple/ pre-commit || pip3 install pre-commit

## one PR ? Commit
# oldnum=`git rev-list origin/master --no-merges --count`
# newnum=`git rev-list HEAD --no-merges --count`
# changenum=$[newnum - oldnum]

# add doc for src code
for rustlist in `git diff origin/master --stat | awk '{print $1}' | grep \.rs$ | tr '\n' ' '`
do
    egrep '#!\[deny\(missing_docs\)\]' $rustlist || sed -i '1i\#![deny(missing_docs)]' $rustlist 2>/dev/null || true
    egrep '#!\[deny\(clippy::all\)\]' $rustlist || sed -i '1i\#![deny(clippy::all)]' $rustlist 2>/dev/null || true
    egrep '#!\[deny\(warnings\)\]' $rustlist || sed -i '1i\#![deny(warnings)]' $rustlist 2>/dev/null || true
done

#fix cargo clippy fail in pre-commit when build.rs is changed
RUSTC_WRAPPER="" cargo clippy -v --all-targets --all-features --tests --benches --examples || exit 1

# run base check
filelist=`git diff origin/master --stat | grep -v "files changed" | awk '{print $1}' | tr '\n' ' '`
export PATH="$PATH:/home/jenkins/.local/bin"
pre-commit run -vvv --files ${filelist}
