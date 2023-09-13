writeFile() {
    local message=${1}
    shift

    printf "$message\n" $@ >> temp_merge_msg
}

commit() {
    targetBranch=$1

    if [ -z "$targetBranch" ]; then
        echo "error: the target branch does not been provided"
        exit 0
    fi

    actualBranch=$(git symbolic-ref --short HEAD)

    targetCommits=$(git log --no-merges --format=format:'%s' $actualBranch..$targetBranch)
    actualCommits=$(git log --no-merges --format=format:'%s' $targetBranch..$actualBranch)
    verboseCommits=$(git log --no-merges --left-right $actualBranch...$targetBranch)

    writeFile '🔀 merge: branch %s into %s' $targetBranch $actualBranch
    writeFile ''

    if [ ! -z "$targetCommits" ]; then
        writeFile 'Branch %s commits:' $targetBranch
        writeFile ''
        writeFile "$targetCommits"
        writeFile ''
    fi

    if [ ! -z "$actualCommits" ]; then
        writeFile 'Branch %s commits:' $actualBranch
        writeFile ''
        writeFile "$actualCommits"
        writeFile ''
    fi

    if [ ! -z "$verboseCommits" ]; then
        writeFile 'Details of commits:'
        writeFile ''
        writeFile "$verboseCommits"
    fi

    git merge --no-ff --no-commit $targetBranch
    git commit -F temp_merge_msg
    rm temp_merge_msg
}

commit $1
