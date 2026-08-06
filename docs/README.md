# Fobos documentation

The wiki source is in `docs/src`. The book is built with [mdBook](https://rust-lang.github.io/mdBook/):

```console
mdbook serve docs
mdbook build docs
```

The generated `docs/book` directory is a build artifact. Keep authored
content in `docs/src` and update `docs/src/SUMMARY.md` when adding a page.
