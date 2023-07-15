# Contributing

There are a couple of ways you can contribute to Pipeline. You can report bugs, propose changes and additions, directly contribute to the codebase, or translating.

## Reporting bugs / Proposing changes or additions

If you experience a bug, or you want to suggest some changes or additions within an app, you can open an issue in the [issue tracker](https://github.com/Tubefeeder/Pipeline/issues). However, we heavily encourage to check if there is an existing issue first. If there is an existing issue, please leave a thumbs up (👍).

## Submitting code

You can submit code by [forking](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/about-forks) this project, editing the desired code and finally submitting a [pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request).

## Translating

### Prerequisites

The process of translating Pipeline is similar to submitting code. Start with [forking](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/about-forks) the project. Then, [clone](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository) the repository. And finally, install [Poedit](https://flathub.org/apps/details/net.poedit.Poedit).

Proceed to [A. Adding a new language](#a-adding-new-language) to add a new language, or [B. Translating an existing language](#b-translating-an-existing-language) to translate an existing language.

### A. Adding a new language

Open Poedit, then press "Create new...". Navigate to the root of the repository. Navigate to `po` and click on `tubefeeder.pot`. Enter the locale you want to translate the program to save the file.

Once translated, click the "Save" button and save the file where `tubefeeder.pot` is located.

Proceed to [Submitting translations](#submitting-translations).

### B. Translating an existing language

Open Poedit, then press "Browse files". Open the `po` file with the appropriate locale.

Once translated, click the "Save" button.

Proceed to [Submitting translations](#submitting-translations).

### Submitting translations

[Push](https://github.com/git-guides/git-push) the changes to GitHub and finally submit a [pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request).
