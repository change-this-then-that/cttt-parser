use std::vec;

fn main() {
    let s = "
// @cttt.name(foo)
let x = 1;
// @cttt.change(bar)

// @cttt.name(bar)
let y = 2;
// @cttt.change(foo)
";

    let allowed_commands = vec![];

    println!("{:#?}", cttt_parser::parse_strict(&s, allowed_commands));
}
