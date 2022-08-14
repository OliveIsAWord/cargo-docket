alias i := install
alias r := run

default:
  @just --list

install:
    cargo install --path .

run:
    cargo run

del:
    rm docket.md
