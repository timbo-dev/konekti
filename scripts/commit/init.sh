#!/bin/sh

init() {
    unikaConfig="[alias]\n\tunikamerge = \"!./.git/merge.sh\"\n\tunikainit = \"!./.git/init.sh\"\n\tunikaflow = \"!./.git/flow.sh\""

    scriptsDir="./scripts/commit"

    mergeSh="$scriptsDir/merge.sh"
    flowSh="$scriptsDir/flow.sh"
    initSh="$scriptsDir/init.sh"

    if ! [ -f $mergeSh ] || ! [ -f $flowSh ] || ! [ -f $initSh ]; then
        echo "the unikaflow scripts not found in $scriptsDir"
        exit 1
    fi

    cp -r $mergeSh "./.git/merge.sh"
    cp -r $flowSh "./.git/flow.sh"
    cp -r $initSh "./.git/init.sh"

    if ! git show-ref --verify --quiet "refs/heads/develop"; then
        git checkout -b develop
    fi

    if [ $(git symbolic-ref --short HEAD) != "develop" ]; then
        git checkout develop
    fi

    if ! [[ "$(cat .git/config)" =~ "$(echo -e "$unikaConfig")" ]]; then
        echo -e "$unikaConfig" >> .git/config
    fi
}

init
