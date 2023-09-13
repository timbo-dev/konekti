#!/bin/sh

feature() {
    local command=$1

    finish() {
        local target=$1
        local mergeTarget=$2

        if [ -z "$mergeTarget" ]; then
            mergeTarget="develop"
        fi

        if ! git show-ref --verify --quiet "refs/heads/$mergeTarget"; then
            echo "error: the provided branch '$mergeTarget' does not exist."
            exit 1
        fi

        if ! git show-ref --verify --quiet "refs/heads/feature/$target"; then
            echo "error: the provided branch 'feature/$target' does not exist."
            exit 1
        else
            actualCommits=$(git log --no-merges --format=format:'%s' $mergeTarget..feature/$target)

            if [ -z "$actualCommits" ]; then
                echo "error: you cannot finish an unmodified branch"
                exit 1
            fi

            git checkout $mergeTarget
            git unikamerge feature/$target
            git branch -D feature/$target
        fi
    }

    start() {
        local target=$1
        local refs=$2

        if [ -z "$refs" ]; then
            refs="develop"
        fi

        if ! git show-ref --verify --quiet "refs/heads/feature/$target"; then

            if ! git show-ref --verify --quiet "refs/heads/$refs"; then
                echo "error: the provided reference branch '$refs' does not exist"
                exit 1
            fi

            git checkout -b feature/$target $refs
        else
            echo "error: the provided feature branch already exist"
            exit 1
        fi
    }

    case "$command" in
        start|s)
            shift

            if [ -z "$1" ]; then
                echo "error: you need to provide a branch name"
                exit 1
            fi

            start $@
            break
        ;;

        finish|f)
            shift

            if [ -z "$1" ]; then
                echo "error: you need to provide a branch name"
                exit 1
            fi

            finish $@
            break
        ;;

        *)
            if [ -z "$command" ]; then
                echo "error: the flow feature command does not been provided."
                exit 1
            fi

            echo "Command '$command' not found"
            exit 1
        ;;
    esac
}
release() {
    local command=$1

    start() {
        local release=$1

        if ! git show-ref --verify --quiet "refs/heads/release/$release"; then
            git checkout -b release/$release develop
        else
            echo "error: the provided release branch already exist"
            exit 1
        fi
    }

    finish() {
        local target=$1
        local FORCE_MODE=$2

        if ! git show-ref --verify --quiet "refs/heads/release/$target"; then
            echo "error: the provided branch 'release/$target' does not exist."
            exit 1
        else
            if [ "$FORCE_MODE" == '--force' ]; then
                echo
                git commit -m "🔖 release: add tag $target" --allow-empty
            else
                git commit -m "🔖 release: add tag $target"

                if [ $? == 1 ]; then
                    echo "error: please modify your version files"
                    echo "or use --force to skip this verification"
                    exit 1
                fi
            fi

            local last_commit_sha1=$(git rev-parse HEAD)

            tag() {
                targetBranch=$1
                target=$2

                if [ -z "$targetBranch" ]; then
                    echo "error: the target branch does not been provided"
                    exit 1
                fi

                actualBranch=$(git symbolic-ref --short HEAD)
                actualCommits=$(git log --no-merges --format=format:'%s' $targetBranch..$actualBranch)

                printf '🔖 release: add tag %s\n' $target
                echo ''

                if [ ! -z "$actualCommits" ]; then
                    printf 'Release %s changelog:\n' $target
                    echo ''
                    echo "$actualCommits"
                    echo ''
                fi
            }

            tag_message=$(tag main $target)

            git tag -a $target -m "$tag_message" $last_commit_sha1

            git checkout develop
            git unikamerge release/$target
            git checkout main
            git unikamerge release/$target

            git branch -D release/$target
            git checkout develop
            git push --tags
            git push origin develop
            git push origin main
        fi
    }

    case "$command" in
        start|s)
            shift

            if [ -z "$1" ]; then
                echo "error: you need to provide a branch name"
                exit 1
            fi

            start $@
            break
        ;;

        finish|f)
            shift

            if [ -z "$1" ]; then
                echo "error: you need to provide a branch name"
                exit 1
            fi

            finish $@
            break
        ;;

        *)
            if [ -z "$command" ]; then
                echo "error: the flow feature command does not been provided."
                exit 1
            fi

            echo "Command '$command' not found"
            exit 1
        ;;
    esac

}

flow() {
    local command=$1

    case "$command" in
        feature|f)
            shift
            feature $@
            break
        ;;

        release|r)
            shift
            release $@
            break
        ;;

        hotfix|h)
            break
        ;;

        *)
            if [ -z "$command" ]; then
                echo "error: the flow command does not been provided."
                exit 1
            fi

            echo "Command '$command' not found"
            exit 1
        ;;
    esac
}

flow $@
