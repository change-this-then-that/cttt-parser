# cttt-parser

[![Crates.io](https://img.shields.io/crates/v/cttt-parser.svg)](https://crates.io/crates/cttt-parser)
[![Docs.rs](https://docs.rs/cttt-parser/badge.svg)](https://docs.rs/cttt-parser)
[![Test](https://github.com/change-this-then-that/cttt-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/change-this-then-that/cttt-parser/actions/workflows/ci.yml)
[![Audit](https://github.com/change-this-then-that/cttt-parser/actions/workflows/audit.yml/badge.svg)](https://github.com/change-this-then-that/cttt-parser/actions/workflows/audit.yml)

A parser for the [Change This Then That](https://github.com/change-this-then-that).

# Examples

Basic usage:

```
let s = "
// @cttt.name(foo)
let x = 1;
// @cttt.change(bar)

// @cttt.name(bar)
let y = 2;
// @cttt.change(foo)
";

println!("{:#?}", cttt_parser::parse(&s));
```

Strict usage:

```
let s = "
// @cttt.name(foo)
let x = 1;
// @cttt.change(bar)

// @cttt.name(bar)
let y = 2;
// @cttt.change(foo)
";

println!(
  "{:#?}",
  cttt_parser::parse_strict(&s, vec!["name".to_string(), "change".to_string()])
);
```
