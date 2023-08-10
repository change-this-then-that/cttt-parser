// MIT License
//
// Copyright (c) 2023 Justin Poehnelt <justin.poehnelt@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"] // relative to src
struct ChangeParser;

#[derive(Debug, PartialEq, serde::Serialize, Clone)]
pub struct Comment {
    command: Option<String>,
    debug: CommentDebug,
    args: Vec<String>,
}

#[derive(Debug, PartialEq, serde::Serialize, Clone)]
pub struct CommentDebug {
    pub comment: String,
    pub line: usize,
    pub col: usize,
}

pub static NAMESPACE: &str = "@cttt";

// Parse a string into a vector of Comments.
pub fn parse(s: &str) -> Result<Vec<Comment>, pest::error::Error<Rule>> {
    let parse = ChangeParser::parse(Rule::document, s).unwrap();

    let mut comments: Vec<Comment> = vec![];

    // make an iterator over the pairs in the rule
    for pair in parse {
        // match the rule, as the rule is an enum
        match pair.as_rule() {
            Rule::EOI => (),
            Rule::document => {
                // for each sub-rule, print the inner contents
                for document in pair.into_inner() {
                    match document.as_rule() {
                        Rule::EOI => (),
                        Rule::comment => {
                            let mut command = None;
                            let mut args: Vec<String> = vec![];
                            let (line, col) = document.as_span().start_pos().line_col();

                            let comment = document
                                .as_span()
                                .as_str()
                                .to_string()
                                .trim_end()
                                .to_string();

                            let col = comment.find(NAMESPACE).unwrap_or(0) + col - 1;

                            // match the sub-rule
                            for part in document.into_inner() {
                                match part.as_rule() {
                                    Rule::command => {
                                        command = Some(part.as_span().as_str().to_string())
                                    }
                                    Rule::args => {
                                        args = match part.as_span().as_str().trim() {
                                            "" => vec![],
                                            s => s
                                                .trim()
                                                .split(',')
                                                .map(|s| s.to_string().trim().to_string())
                                                .filter(|s| !s.is_empty())
                                                .collect(),
                                        }
                                    }
                                    _ => (),
                                }
                            }

                            comments.push(Comment {
                                args,
                                command,
                                debug: CommentDebug { comment, line, col },
                            });
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(comments)
}

// custom error
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct UnknownCommandError {
    comment: String,
    command: String,
    col: usize,
    line: usize,
}

pub enum StrictParseError {
    UnknownCommand(Vec<UnknownCommandError>),
    Pest(pest::error::Error<Rule>),
}

pub fn parse_strict(s: &str, commands: Vec<String>) -> Result<Vec<Comment>, StrictParseError> {
    let comments = parse(s).map_err(StrictParseError::Pest)?;

    let mut unknown_command_errors: Vec<UnknownCommandError> = vec![];

    // check for unknown commands
    comments.iter().for_each(|c| match &c.command {
        Some(command) => {
            if !commands.contains(command) {
                let col = command.find(NAMESPACE).unwrap_or(0)
                    + c.debug.col
                    + NAMESPACE.len()
                    + ".".len();

                unknown_command_errors.push(UnknownCommandError {
                    comment: c.debug.comment.clone(),
                    command: c.command.clone().unwrap(),
                    line: c.debug.line,
                    col,
                });
            }
        }
        None => (),
    });

    if !unknown_command_errors.is_empty() {
        return Err(StrictParseError::UnknownCommand(unknown_command_errors));
    }

    Ok(comments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let input = "/* @cttt.named(123) */\n/* @cttt.change(123,abc) */";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: named
          debug:
            comment: /* @cttt.named(123) */
            line: 1
            col: 3
          args:
            - "123"
        - command: change
          debug:
            comment: "/* @cttt.change(123,abc) */"
            line: 2
            col: 3
          args:
            - "123"
            - abc
        "###);
    }

    #[test]
    fn test_parse_no_command() {
        let input = "// @cttt";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: ~
          debug:
            comment: // @cttt
            line: 1
            col: 3
          args: []
        "###);
    }

    #[test]
    fn test_parse_nested() {
        let input =
            "// @cttt.named(123)\n// @cttt.named(2)\nx +=1;\n// @cttt.change(3,4,5)\n// @cttt.change(1)";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: named
          debug:
            comment: // @cttt.named(123)
            line: 1
            col: 3
          args:
            - "123"
        - command: named
          debug:
            comment: // @cttt.named(2)
            line: 2
            col: 3
          args:
            - "2"
        - command: change
          debug:
            comment: "// @cttt.change(3,4,5)"
            line: 4
            col: 3
          args:
            - "3"
            - "4"
            - "5"
        - command: change
          debug:
            comment: // @cttt.change(1)
            line: 5
            col: 3
          args:
            - "1"
        "###);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let input = "// @CTTT.named(SPECIAL_BLOCK)\n// @cttt.CHANGE(./foo.txt,abc)";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: named
          debug:
            comment: // @CTTT.named(SPECIAL_BLOCK)
            line: 1
            col: 0
          args:
            - SPECIAL_BLOCK
        - command: CHANGE
          debug:
            comment: "// @cttt.CHANGE(./foo.txt,abc)"
            line: 2
            col: 3
          args:
            - "./foo.txt"
            - abc
        "###);
    }

    #[test]
    fn test_parse_kebab_command() {
        let input = "// @cttt.named-bar-baz()";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: named-bar-baz
          debug:
            comment: // @cttt.named-bar-baz()
            line: 1
            col: 3
          args: []
        "###);
    }

    #[test]
    fn test_parse_args_whitespace() {
        let input = "// @cttt.change( )";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: change
          debug:
            comment: // @cttt.change( )
            line: 1
            col: 3
          args: []
        "###);
    }

    #[test]
    fn test_parse_args_whitespace_separated() {
        let input = "// @cttt.change(foo, bar)";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: change
          debug:
            comment: "// @cttt.change(foo, bar)"
            line: 1
            col: 3
          args:
            - foo
            - bar
        "###);
    }

    #[test]
    fn test_parse_args_whitespace_trailing_comma() {
        let input = "// @cttt.change(foo, bar,)";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: change
          debug:
            comment: "// @cttt.change(foo, bar,)"
            line: 1
            col: 3
          args:
            - foo
            - bar
        "###);
    }

    #[test]
    fn test_parse_args_characters() {
        let input = "// @cttt.change(./aFoo_Bar-123)";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: change
          debug:
            comment: // @cttt.change(./aFoo_Bar-123)
            line: 1
            col: 3
          args:
            - "./aFoo_Bar-123"
        "###);
    }

    #[test]
    fn test_parse_args_file_path() {
        let input = "// @cttt.change(./foo/README.md, /bar/foo.rs)";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: change
          debug:
            comment: "// @cttt.change(./foo/README.md, /bar/foo.rs)"
            line: 1
            col: 3
          args:
            - "./foo/README.md"
            - /bar/foo.rs
        "###);
    }

    #[test]
    fn test_parse_comment_syntax() {
        for (leading, trailing) in vec![
            ("--", ""),
            ("!", ""),
            ("(*", "*)"),
            ("{-", "-}"),
            ("{", "}"),
            ("/*", "*/"),
            ("/**", "*/"),
            ("//", ""),
            ("\"\"\"", "\"\"\""),
            ("#", ""),
            ("<!--", "-->"),
        ] {
            let input = format!("{} {}.{} {}", leading, NAMESPACE, "foo()", trailing);
            let output = parse(&input).unwrap();

            assert_eq!(output[0].command.clone().unwrap(), String::from("foo"));
            assert_eq!(output[0].debug.comment, input.trim_end());
        }
    }

    #[test]
    fn test_parse_comment_multiline() {
        let input = "
            /**
             * @cttt.named(123)
             */
            x = 123;
            /**
             * @cttt.noop()
             */";
        insta::assert_yaml_snapshot!(parse(input).unwrap(), @r###"
        ---
        - command: named
          debug:
            comment: "             * @cttt.named(123)"
            line: 3
            col: 15
          args:
            - "123"
        - command: noop
          debug:
            comment: "             * @cttt.noop()"
            line: 7
            col: 15
          args: []
        "###);
    }

    #[test]
    fn test_parse_strict_commands() {
        let input = "// @cttt.unknown()\n// @cttt";
        let commands = vec!["foo".to_string(), "bar".to_string()];

        let output = parse_strict(input, commands).unwrap_err();

        match output {
            StrictParseError::UnknownCommand(e) => {
                insta::assert_yaml_snapshot!(e, @r###"
                ---
                - comment: // @cttt.unknown()
                  command: unknown
                  col: 9
                  line: 1
                "###);
            }
            _ => panic!("unexpected error"),
        }
    }
}
