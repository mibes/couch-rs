# Contributing

Feel free to contribute bug-fixes and new features. If you do so, please apply the
[Gitflow](https://www.atlassian.com/git/tutorials/comparing-workflows/gitflow-workflow) workflow. This sounds more
complicated than it is. Check out
the [Getting Started](https://www.atlassian.com/git/tutorials/comparing-workflows/gitflow-workflow)
to initialize your forked project for Gitflow usage. Then use:

* `git flow feature start feature_branch` when contributing a new feature, this is always based on the `develop` branch
* `git flow hotfix start hotfix_branch` when contributing a patch, this is always based on the `master` branch.

Replace `feature_branch` or `hotfix_branch` in the above example with a descriptive name. Do not finish the feature or
hotfix, but do a pull request on the new branch.
