# Contributing

Please read through this whole file before submitting a pull request. Thank you for taking your time for this!

## Unbreakable behaviour

**You must keep the following behaviour in tact!**

You should be able to execute a program by feeding the source code into STDIN and reading the final state of the machine in STDOUT. You may **not** print anything else to STDOUT! In the future, we may want to print to STDERR, but currently there is no code which does so.

So, you should be able to run a remuir program with the following code:

```sh
$ ./remuir < path/to/source_code.remuir
> register 3 8 2
```

## Code style

For Rust files, please follow the [official style guide](https://doc.rust-lang.org/nightly/style-guide/). In particular, every line should be no more than 100 characters long (with an exception for string literals, e.g. in tests).

For Markdown files, I'm not too fussed. Don't restrict yourself to 100 characters per line like in Rust, use your editors word wrap feature and keep one paragraph per line (licensing information is an exception to this). Most Markdown linters will help with the rest of my style preferences (e.g. surround code-blocks in 1 blank line).

For code blocks showing off remuir source code files, your Markdown linter may complain that you should specify a language for the code block. You should leave this blank and not specify a language.
