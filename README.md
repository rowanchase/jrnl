# jnrl

An embarrassingly simple note taking CLI tool.

All this does is create and/or open a markdown file in your $EDITOR with a title header already populated.
When you save and close the file it will commit and push.
That's it.

### NOTE

This is just for me, so there's a lot of hard coded values at the moment, eg. the root folder
of your notes will always be "$HOME/journal/" and your editor will be neovim.

## Usage

### Make a note for today:

`jrnl`

This will create a note at $HOME/journal/{year}/{month}/{day}.md with the header eg. # Monday 30 June 2025

### Make a note for another date:

`jrnl date 2025 6 30`

This will create a note at $HOME/journal/2025/6/30.md with the header # Monday 30 June 2025

### Make a namespace'd note

`jrnl ns pets dogs shepherds swiss`

This will create a note at $HOME/journal/pets/dogs/shepherds/swiss.md with the header # pets.dogs.shepherds.swiss

## Complimentary tools

### Marksman

[A Markdown LSP](https://github.com/artempyanykh/marksman) will help you link between and navigate your notes.

### git-remote-gcrypt

[Simple tool to encrypt your repo](https://spwhitton.name/tech/code/git-remote-gcrypt/), there's full instructions
[here](https://flolu.de/blog/encrypted-git-repository).
