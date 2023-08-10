fn main() {
    let s = "
// @cttt.name(foo)
let x = 1;
// @cttt.change(bar)

// @cttt.name(bar)
let y = 2;
// @cttt.change(foo)
";

    println!("{:#?}", cttt_parser::parse(s));
}
