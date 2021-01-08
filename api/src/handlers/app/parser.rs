use combine::{
    ParseError, Parser, Stream, attempt, choice, eof, from_str, look_ahead,
    many, many1, optional, parser, satisfy, skip_many, skip_many1, token, value,
};
use combine::error::StreamError;
use combine::parser::{
    char::{digit, newline, space, string},
    combinator::recognize,
    repeat::{skip_count_min_max, take_until},
};
use std::str::FromStr;

use crate::errors;
use crate::models;
use super::text::{self, *};

impl FromStr for Req {
    type Err = errors::ServiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let src = Washer { src: s }.wash();
        Ok(a_req().parse(&*src)?.0)
    }
}

struct Washer<'a> {
    src: &'a str,
}
impl<'a> Washer<'a> {
    fn wash(&self) -> String {
        self
        .remove_comments()
        .trim()
        .into()
    }
    fn remove_comments(&self) -> String {
        let prefix = "<!--";
        let suffix = "-->";
        let mut src = self.src;
        let mut res = String::new();
        loop {
            let cursor = src.find(prefix).unwrap_or_else(|| src.len());
            let pair = src.split_at(cursor);
            res.push_str(pair.0);
            src = pair.1;
            let cursor = src.find(suffix).map(|cur| cur + suffix.len()).unwrap_or_else(|| src.len());
            let pair = src.split_at(cursor);
            src = pair.1;
            if src.is_empty() { break }
        }
        res
    }
}

parser! {
    fn a_req[Input]()(Input) -> Req
    where [ Input: Stream<Token = char> ] {
        choice((
            token('/').with(a_req_command()).map(|x| Req::Command(x)),
            a_req_tasks().map(|x| Req::Tasks(ReqTasks {
                tasks: x.tasks.into_iter().rev().collect()
            })),
        ))
    }
}
parser! {
    fn a_req_command[Input]()(Input) -> ReqCommand
    where [ Input: Stream<Token = char> ] {
        choice((
            eof().map(|_| ReqCommand::Help),
            token('u').with(a_req_user()).map(|x| ReqCommand::User(x)),
            token('s').with(a_condition()).map(|x| ReqCommand::Search(x)),
            string("tutorial").map(|_| ReqCommand::Tutorial),
            string("coffee").map(|_| ReqCommand::Coffee),
        ))
    }
}
parser! {
    fn a_spaces1[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        skip_many1(space())
    }
}
parser! {
    fn a_req_user[Input]()(Input) -> ReqUser
    where [ Input: Stream<Token = char> ] {
        choice((
            eof().map(|_| ReqUser::Info),
            a_spaces1().with(a_req_user()),
            token('-').with(a_req_modify()).map(|x| ReqUser::Modify(x)),
        ))
    }
}
parser! {
    fn a_req_modify[Input]()(Input) -> ReqModify
    where [ Input: Stream<Token = char> ] {
        choice((
            token('e').with(a_spaces1().with(a_ascii_graphics())).map(|x| ReqModify::Email(x)),
            token('p').with(a_spaces1().with(a_password_set())).map(|x| ReqModify::Password(x)),
            token('n').with(a_spaces1().with(a_ascii_graphics())).map(|x| ReqModify::Name(x)),
            token('t').with(a_spaces1().with(a_timescale())).map(|x| ReqModify::Timescale(x)),
        ))
    }
}
parser! {
    fn a_ascii_graphics[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many1(satisfy(|c: char| c.is_ascii_graphic()))
    }
}
parser! {
    fn a_password_set[Input]()(Input) -> PasswordSet
    where [ Input: Stream<Token = char> ] {
        a_ascii_graphics().skip(a_spaces1())
        .and(a_ascii_graphics()).skip(a_spaces1())
        .and(a_ascii_graphics())
        .map(|((o, n), c)| PasswordSet {
            old: o,
            new: n,
            confirmation: c,
        })
    }
}
parser! {
    fn a_timescale[Input]()(Input) -> Timescale
    where [ Input: Stream<Token = char> ] {
        choice((
            string("Y")  .map(|_| Timescale::Year),
            string("Q")  .map(|_| Timescale::Quarter),
            string("M")  .map(|_| Timescale::Month),
            string("W")  .map(|_| Timescale::Week),
            string("D")  .map(|_| Timescale::Day),
            string("6h") .map(|_| Timescale::Hours6),
            string("h")  .map(|_| Timescale::Hour),
            string("15m").map(|_| Timescale::Minutes15),
            string("m")  .map(|_| Timescale::Minute),
        ))
    }
}
parser! {
    fn a_condition[Input]()(Input) -> Condition
    where [ Input: Stream<Token = char> ] {
        let opt = || choice((
            eof().map(|_| Condition::default()),
            a_spaces1().with(a_condition()),
        ));
        choice((
            opt(),
            token('-').with(a_boolean()).and(opt()).map(|(boolean, mut opt)| {
                if let Some(b) = boolean.is_archived {
                    opt.boolean.is_archived = Some(b)
                }
                if let Some(b) = boolean.is_starred {
                    opt.boolean.is_starred = Some(b)
                }
                if let Some(b) = boolean.is_leaf {
                    opt.boolean.is_leaf = Some(b)
                }
                if let Some(b) = boolean.is_root {
                    opt.boolean.is_root = Some(b)
                }
                opt
            }),
            attempt(
                optional(a_non_nega_i().skip(token('<'))).skip(token('#')).and(optional(token('<').with(a_non_nega_i())))
            ).and(opt()).map(|((l, r), mut opt)| {
                if let Some(non_nega_i) = l {
                    opt.context.0 = Some(non_nega_i);
                }
                if let Some(non_nega_i) = r {
                    opt.context.1 = Some(non_nega_i);
                }
                opt
            }),
            attempt(
                optional(a_non_nega_f().skip(token('<'))).skip(token('w')).and(optional(token('<').with(a_non_nega_f())))
            ).and(opt()).map(|((l, r), mut opt)| {
                if let Some(non_nega_f) = l {
                    opt.weight.0 = Some(non_nega_f);
                }
                if let Some(non_nega_f) = r {
                    opt.weight.1 = Some(non_nega_f);
                }
                opt
            }),
            attempt(
                optional(a_datetime().skip(token('<'))).skip(token('s')).and(optional(token('<').with(a_datetime())))
            ).and(opt()).map(|((l, r), mut opt)| {
                if let Some(datetime) = l {
                    opt.startable.0 = Some(datetime);
                }
                if let Some(datetime) = r {
                    opt.startable.1 = Some(datetime);
                }
                opt
            }),
            attempt(
                optional(a_datetime().skip(token('<'))).skip(token('d')).and(optional(token('<').with(a_datetime())))
            ).and(opt()).map(|((l, r), mut opt)| {
                if let Some(datetime) = l {
                    opt.deadline.0 = Some(datetime);
                }
                if let Some(datetime) = r {
                    opt.deadline.1 = Some(datetime);
                }
                opt
            }),
            attempt(
                optional(a_datetime().skip(token('<'))).skip(token('c')).and(optional(token('<').with(a_datetime())))
            ).and(opt()).map(|((l, r), mut opt)| {
                if let Some(datetime) = l {
                    opt.created_at.0 = Some(datetime);
                }
                if let Some(datetime) = r {
                    opt.created_at.1 = Some(datetime);
                }
                opt
            }),
            attempt(
                optional(a_datetime().skip(token('<'))).skip(token('u')).and(optional(token('<').with(a_datetime())))
            ).and(opt()).map(|((l, r), mut opt)| {
                if let Some(datetime) = l {
                    opt.updated_at.0 = Some(datetime);
                }
                if let Some(datetime) = r {
                    opt.updated_at.1 = Some(datetime);
                }
                opt
            }),
            a_expression().and(opt()).map(|(expression, mut opt)| {
                opt.title = Some(expression);
                opt
            }),
            token('@').with(a_expression()).and(opt()).map(|(expression, mut opt)| {
                opt.assign = Some(expression);
                opt
            }),
            token('&').with(a_expression()).and(opt()).map(|(expression, mut opt)| {
                opt.link = Some(expression);
                opt
            }),
        ))
    }
}
parser! {
    fn a_expression[Input]()(Input) -> text::Expression
    where [ Input: Stream<Token = char> ] {
        choice((
            a_quoted().map(|quoted|
                text::Expression::Words(quoted.split_whitespace().map(|s| s.to_string()).collect())
            ),
            attempt(token('r').with(a_quoted())).map(|quoted| text::Expression::Regex(quoted)),
        ))
    }
}
parser! { // ####" any "####
    fn a_quoted[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        let prefix = |depth: usize| attempt(skip_count_min_max(depth, depth, token('#')).skip(token('"')));
        let suffix = |depth: usize| attempt(token('"').with(skip_count_min_max(depth, depth, token('#'))));
        let quoted = |depth: usize| prefix(depth).with(take_until(suffix(depth))).skip(suffix(depth));
        choice((
            quoted(0),
            quoted(1),
            quoted(2),
            quoted(3),
            quoted(4),
        ))
    }
}
parser! {
    fn a_graphics[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many1(satisfy(|c: char| !c.is_whitespace() && !c.is_control()))
    }
}
parser! {
    fn a_graphics_not_joint[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many1(satisfy(|c: char| !c.is_whitespace() && !c.is_control() && c != '[' && c != ']'))
    }
}
parser! {
    fn a_boolean[Input]()(Input) -> Boolean
    where [ Input: Stream<Token = char> ] {
        choice((
            attempt(optional(token('!')).skip(token('a'))).and(optional(a_boolean()))
            .map(|(not, opt)| {
                let mut boolean = opt.unwrap_or_default();
                boolean.is_archived = Some(not.is_none());
                boolean
            }),
            attempt(optional(token('!')).skip(token('s'))).and(optional(a_boolean()))
            .map(|(not, opt)| {
                let mut boolean = opt.unwrap_or_default();
                boolean.is_starred = Some(not.is_none());
                boolean
            }),
            attempt(optional(token('!')).skip(token('l'))).and(optional(a_boolean()))
            .map(|(not, opt)| {
                let mut boolean = opt.unwrap_or_default();
                boolean.is_leaf = Some(not.is_none());
                boolean
            }),
            attempt(optional(token('!')).skip(token('r'))).and(optional(a_boolean()))
            .map(|(not, opt)| {
                let mut boolean = opt.unwrap_or_default();
                boolean.is_root = Some(not.is_none());
                boolean
            }),
        ))
    }
}
parser! {
    fn a_non_nega_i[Input]()(Input) -> i32
    where [ Input: Stream<Token = char> ] {
        from_str(many1::<String, _, _>(digit()))
    }
}
parser! {
    fn a_non_nega_f[Input]()(Input) -> f32
    where [ Input: Stream<Token = char> ] {
        from_str(recognize::<String, _, _>((
            many::<String, _, _>(digit()),
            optional(token('.')),
            many::<String, _, _>(digit()),
        )))
    }
}
parser! {
    fn a_datetime[Input]()(Input) -> models::EasyDateTime
    where [ Input: Stream<Token = char> ] {
        choice((
            attempt(a_date().skip(token('T')).and(a_time())).map(|(d, t)| {
                models::EasyDateTime {
                    date: Some(d),
                    time: Some(t),
                }
            }),
            a_date().map(|d| {
                models::EasyDateTime {
                    date: Some(d),
                    time: None,
                }
            }),
            a_time().map(|t| {
                models::EasyDateTime {
                    date: None,
                    time: Some(t),
                }
            }),
        ))
    }
}
parser! {
    fn a_date[Input]()(Input) -> models::EasyDate
    where [ Input: Stream<Token = char> ] {
        attempt(optional(a_non_nega_i()).skip(token('/')).and(optional(a_non_nega_i())).skip(token('/')).and(optional(a_non_nega_i())))
        .map(|((y, m), d)| {
            models::EasyDate {
                y: y,
                m: m,
                d: d,
            }
        })
    }
}
parser! {
    fn a_time[Input]()(Input) -> models::EasyTime
    where [ Input: Stream<Token = char> ] {
        attempt(optional(a_non_nega_i()).skip(token(':')).and(optional(a_non_nega_i())))
        .map(|(h, m)| {
            models::EasyTime {
                h: h,
                m: m,
            }
        })
    }
}
parser! {
    fn a_req_tasks[Input]()(Input) -> ReqTasks
    where [ Input: Stream<Token = char> ] {
        choice((
            eof().map(|_| ReqTasks::default()),
            attempt(a_req_task()).and(a_req_tasks()).map(|(t, mut ts)| {
                ts.tasks.push(t);
                ts
            }),
            a_empty_lines().with(a_req_tasks()),
        ))
    }
}
parser! {
    fn a_req_task[Input]()(Input) -> ReqTask
    where [ Input: Stream<Token = char> ] {
        a_indent()
        .and(a_attribute())
        .and(optional(choice((
            a_link(),
            attempt(newline().with(a_inline_spaces().with(a_link()))),
        ))))
        .skip(a_inline_spaces()).skip(choice((
            newline().map(|_| ()),
            eof(),
        )))
        .and_then(|((indent, mut attribute), link)| {
            if attribute.title.is_empty() {
                return Err(
                    <Input::Error as ParseError<_, _, _>>::StreamError::expected_static_message(
                        "empty title"
                    )
                )
            }
            attribute.title = attribute.title.into_iter().rev().collect();
            Ok(ReqTask {
                indent: indent,
                attribute: attribute,
                link: link,
            })
        })
    }
}
parser! {
    fn a_indent[Input]()(Input) -> i32
    where [ Input: Stream<Token = char> ] {
        choice((
            look_ahead(a_graphics()).map(|_| 0),
            skip_count_min_max(4, 4, token(' ')).with(a_indent()).map(|mut indent| {
                indent += 1;
                indent
            }),
        ))
    }
}
parser! {
    fn a_attribute[Input]()(Input) -> Attribute
    where [ Input: Stream<Token = char> ] {
        let opt = || choice((
            eof().map(|_| Attribute::default()),
            look_ahead(newline()).map(|_| Attribute::default()),
            look_ahead(a_link()).map(|_| Attribute::default()),
            a_inline_spaces1().with(a_attribute())
        ));
        choice((
            opt(),
            token('*').with(opt()).map(|mut opt| {
                opt.is_starred = true;
                opt
            }),
            attempt(token('#').with(a_non_nega_i())).and(opt()).map(|(i, mut opt)| {
                opt.id = Some(i);
                opt
            }),
            attempt(token('$').with(a_non_nega_f())).and(opt()).map(|(f, mut opt)| {
                opt.weight = Some(f);
                opt
            }),
            attempt(a_graphics_not_joint().skip(token(']'))).and(opt()).map(|(g, mut opt)| {
                opt.joint_head = Some(g);
                opt
            }),
            attempt(token('[').with(a_graphics_not_joint())).and(opt()).map(|(g, mut opt)| {
                opt.joint_tail = Some(g);
                opt
            }),
            attempt(token('@').with(a_ascii_graphics())).and(opt()).map(|(ag, mut opt)| {
                opt.assign = Some(ag);
                opt
            }),
            attempt(a_datetime().skip(token('-'))).and(opt()).map(|(dt, mut opt)| {
                opt.startable = Some(dt);
                opt
            }),
            attempt(token('-').with(a_datetime())).and(opt()).map(|(dt, mut opt)| {
                opt.deadline = Some(dt);
                opt
            }),
            a_graphics().and(opt()).map(|(g, mut opt)| {
                opt.title.push(g);
                opt
            }),
        ))
    }
}
parser! {
    fn a_link[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        recognize((
            choice([
                attempt(string("http://")),
                attempt(string("https://")),
            ]),
            optional(a_ascii_graphics()),
        ))
    }
}
parser! {
    fn a_inline_spaces[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        skip_many(satisfy(|c: char| c.is_whitespace() && c != '\n'))
    }
}
parser! {
    fn a_inline_spaces1[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        skip_many1(satisfy(|c: char| c.is_whitespace() && c != '\n'))
    }
}
parser! {
    fn a_empty_lines[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        choice((
            eof().map(|_| ()),
            a_empty_line().with(a_empty_lines()),
            value(()),
        ))
    }
}
parser! {
    fn a_empty_line[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        choice((
            eof().map(|_| ()),
            newline().map(|_| ()),
            attempt(a_inline_spaces1().with(a_empty_line()))
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::EasyParser;

    fn washer<'a>() -> Washer<'a> {
        Washer { src:
            " \n\r\n pon  \n\r\n <!-- \n\r\n cho  \n\r\n <!-- \n\r\n cho  \n\r\n -->  \n\r\n pon  \n\r\n -->  \n\r\n pon  \n\r\n <!-- \n\r\n cho  \n\r\n "
        }
    }
    #[test]
    fn t_washer_remove_comments() {
        let t_00 = washer().remove_comments();
        assert_eq!(t_00, String::from(
            " \n\r\n pon  \n\r\n   \n\r\n pon  \n\r\n -->  \n\r\n pon  \n\r\n "
        ));
    }
    #[test]
    fn t_washer_wash() {
        let t_00 = washer().wash();
        assert_eq!(t_00, String::from(
            "pon  \n\r\n   \n\r\n pon  \n\r\n -->  \n\r\n pon"
        ));
    }
    #[test]
    fn t_a_req() {
        let t_00 = a_req().easy_parse("");
        let t_01 = a_req().easy_parse(" ");
        let t_02 = a_req().easy_parse("\n\r\n");
        let t_03 = a_req().easy_parse("/");
        let t_10 = a_req().easy_parse("/ ");
        let t_11 = a_req().easy_parse("/12/- * task");
        assert_eq!(t_00, Ok((Req::Tasks(ReqTasks { tasks: Vec::new() }), "")));
        assert_eq!(t_01, t_00);
        assert_eq!(t_02, t_00);
        assert_eq!(t_03, Ok((Req::Command(ReqCommand::Help), "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_a_empty_line() {
        let t_00 = a_empty_line().easy_parse("");
        let t_01 = a_empty_line().easy_parse("\n   ");
        let t_02 = a_empty_line().easy_parse("   \n   ");
        let t_10 = a_empty_line().easy_parse("x");
        let t_11 = a_empty_line().easy_parse("   x\n   ");
        assert_eq!(t_00, Ok(((), "")));
        assert_eq!(t_01, Ok(((), "   ")));
        assert_eq!(t_02, t_01);
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_a_empty_lines() {
        let t_00 = a_empty_lines().easy_parse("");
        let t_01 = a_empty_lines().easy_parse("\n   ");
        let t_02 = a_empty_lines().easy_parse("   \n   x");
        assert_eq!(t_00, Ok(((), "")));
        assert_eq!(t_01, t_00);
        assert_eq!(t_02, Ok(((), "   x")));
    }
    #[test]
    fn t_a_inline_spaces1() {
        let t_00 = a_inline_spaces1().easy_parse(" ");
        let t_01 = a_inline_spaces1().easy_parse("   \n   ");
        let t_10 = a_inline_spaces1().easy_parse("");
        let t_11 = a_inline_spaces1().easy_parse("\n");
        assert_eq!(t_00, Ok(((), "")));
        assert_eq!(t_01, Ok(((), "\n   ")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_a_req_tasks() {
        let t_00 = a_req_tasks().easy_parse("");
        let t_01 = a_req_tasks().easy_parse(
            "jump\n    step\n        hop"
        );
        let t_02 = a_req_tasks().easy_parse(
            "jump https://jump\n    step http://step"
        );
        let t_03 = a_req_tasks().easy_parse(
            "jump\nhttps://jump\n    step\n    http://step"
        );
        let t_04 = a_req_tasks().easy_parse(" ");
        let t_05 = a_req_tasks().easy_parse("\n");
        let t_06 = a_req_tasks().easy_parse("\r\n");
        assert_eq!(t_00, Ok((ReqTasks { tasks: Vec::new() }, "")));
        assert_eq!(t_01, Ok((ReqTasks { tasks: vec![
            ReqTask {
                indent: 2,
                attribute: Attribute {
                    is_starred: false,
                    id: None,
                    weight: None,
                    joint_head: None,
                    joint_tail: None,
                    assign: None,
                    startable: None,
                    deadline: None,
                    title: vec![String::from("hop"),]
                },
                link: None,
            },
            ReqTask {
                indent: 1,
                attribute: Attribute {
                    is_starred: false,
                    id: None,
                    weight: None,
                    joint_head: None,
                    joint_tail: None,
                    assign: None,
                    startable: None,
                    deadline: None,
                    title: vec![String::from("step"),]
                },
                link: None,
            },
            ReqTask {
                indent: 0,
                attribute: Attribute {
                    is_starred: false,
                    id: None,
                    weight: None,
                    joint_head: None,
                    joint_tail: None,
                    assign: None,
                    startable: None,
                    deadline: None,
                    title: vec![String::from("jump"),]
                },
                link: None,
            },
        ]}, "")));
        assert_eq!(t_02, Ok((ReqTasks { tasks: vec![
            ReqTask {
                indent: 1,
                attribute: Attribute {
                    is_starred: false,
                    id: None,
                    weight: None,
                    joint_head: None,
                    joint_tail: None,
                    assign: None,
                    startable: None,
                    deadline: None,
                    title: vec![String::from("step"),]
                },
                link: Some(String::from("http://step")),
            },
            ReqTask {
                indent: 0,
                attribute: Attribute {
                    is_starred: false,
                    id: None,
                    weight: None,
                    joint_head: None,
                    joint_tail: None,
                    assign: None,
                    startable: None,
                    deadline: None,
                    title: vec![String::from("jump"),]
                },
                link: Some(String::from("https://jump")),
            },
        ]}, "")));
        assert_eq!(t_03, t_02);
        assert_eq!(t_04, t_00);
        assert_eq!(t_05, t_00);
        assert_eq!(t_06, t_00);
    }
    #[test]
    fn t_a_req_command() {
        let t_00 = a_req_command().easy_parse("");
        let t_01 = a_req_command().easy_parse("u");
        let t_02 = a_req_command().easy_parse("s");
        let t_03 = a_req_command().easy_parse("tutorial");
        let t_04 = a_req_command().easy_parse("coffee");
        let t_10 = a_req_command().easy_parse(" ");
        let t_11 = a_req_command().easy_parse("x");
        assert_eq!(t_00, Ok((ReqCommand::Help, "")));
        assert_eq!(t_01, Ok((ReqCommand::User(ReqUser::Info), "")));
        assert_eq!(t_02, Ok((ReqCommand::Search(Condition::default()), "")));
        assert_eq!(t_03, Ok((ReqCommand::Tutorial, "")));
        assert_eq!(t_04, Ok((ReqCommand::Coffee, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_a_req_user() {
        let t_00 = a_req_user().easy_parse("");
        let t_01 = a_req_user().easy_parse(" ");
        let t_11 = a_req_user().easy_parse("x");
        assert_eq!(t_00, Ok((ReqUser::Info, "")));
        assert_eq!(t_01, t_00);
        assert!(t_11.is_err());
    }
    #[test]
    fn t_a_req_modify() {
        let t_00 = a_req_modify().easy_parse("n   satun__   etc...   ");
        let t_10 = a_req_modify().easy_parse("");
        let t_11 = a_req_modify().easy_parse(" ");
        let t_12 = a_req_modify().easy_parse("x");
        assert_eq!(t_00, Ok((ReqModify::Name(String::from("satun__")), "   etc...   ")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
    }
    #[test]
    fn t_a_ascii_graphics() {
        let t_00 = a_ascii_graphics().easy_parse(
            r##"!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~   etc..."##
        );
        let t_10 = a_ascii_graphics().easy_parse("");
        let t_11 = a_ascii_graphics().easy_parse(" ");
        let t_12 = a_ascii_graphics().easy_parse("\n");
        let t_13 = a_ascii_graphics().easy_parse("のんあすきー");
        assert_eq!(t_00, Ok((String::from(
            r##"!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~"##
        ), "   etc...")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());
    }
    #[test]
    fn t_a_password_set() {
        let t_00 = a_password_set().easy_parse(
            r##"old!"#$%&'()*+,-./   new0123456789   confirmation:;<=>?@   etc..."##
        );
        let t_10 = a_password_set().easy_parse("");
        let t_11 = a_password_set().easy_parse("   old new confirmation");
        let t_12 = a_password_set().easy_parse("old new_without_confirmation");
        let t_13 = a_password_set().easy_parse("ぱすわーどに　和文字　とな？");
        assert_eq!(t_00, Ok((PasswordSet {
            old: String::from(r##"old!"#$%&'()*+,-./"##),
            new: String::from(r##"new0123456789"##),
            confirmation: String::from(r##"confirmation:;<=>?@"##),
        }, "   etc...")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());

    }
    #[test]
    fn t_a_timescale() {
        let t_00 = a_timescale().easy_parse("15mm   etc...");
        let t_10 = a_timescale().easy_parse("");
        let t_11 = a_timescale().easy_parse("   15m");
        let t_12 = a_timescale().easy_parse("y");
        let t_13 = a_timescale().easy_parse("恒河沙");
        assert_eq!(t_00, Ok((Timescale::Minutes15, "m   etc...")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());
    }
    #[test]
    fn t_a_condition() {
        let t_00 = a_condition().easy_parse("");
        let t_01 = a_condition().easy_parse(" ");
        let t_02 = a_condition().easy_parse("# w");
        let t_03 = a_condition().easy_parse(
            r##"333<#<777 -a!s -l .5<w<24 s<15: /12/<d c 2021//<u<//30T6:"##
        );
        let t_04 = a_condition().easy_parse(
            r##"333<#<777 -a!s -l .5<w<24 s<15: /12/<d c 2021//<u<//30T6: "tit le" @r#"double"quoted"man"# &r".*domain\.com.*\?page=[1-5]#(frag|ment)"   "##
        );
        let t_10 = a_condition().easy_parse("title");
        assert_eq!(t_00, Ok((
            Condition {
                boolean: Boolean {
                    is_archived: None,
                    is_starred: None,
                    is_leaf: None,
                    is_root: None,
                },
                context: (None, None),
                weight: (None, None),
                startable: (None, None),
                deadline: (None, None),
                created_at: (None, None),
                updated_at: (None, None),
                title:  None,
                assign: None,
                link: None,
            },
            ""
        )));
        assert_eq!(t_01, t_00);
        assert_eq!(t_02, t_00);
        assert_eq!(t_03, Ok((
            Condition {
                boolean: Boolean {
                    is_archived: Some(true),
                    is_starred: Some(false),
                    is_leaf: Some(true),
                    is_root: None,
                },
                context: (Some(333), Some(777)),
                weight: (Some(0.5), Some(24.0)),
                startable: (
                    None,
                    Some(models::EasyDateTime {
                        date: None,
                        time: Some(models::EasyTime {
                            h: Some(15),
                            m: None,
                        }),
                    })
                ),
                deadline: (
                    Some(models::EasyDateTime {
                        date: Some(models::EasyDate {
                            y: None,
                            m: Some(12),
                            d: None,
                        }),
                        time: None,
                    }),
                    None
                ),
                created_at: (None, None),
                updated_at: (
                    Some(models::EasyDateTime {
                        date: Some(models::EasyDate {
                            y: Some(2021),
                            m: None,
                            d: None,
                        }),
                        time: None,
                    }),
                    Some(models::EasyDateTime {
                        date: Some(models::EasyDate {
                            y: None,
                            m: None,
                            d: Some(30),
                        }),
                        time: Some(models::EasyTime {
                            h: Some(6),
                            m: None,
                        }),
                    })
                ),
                title: None,
                assign: None,
                link: None,
            },
            ""
        )));
        assert_eq!(t_04, Ok((
            Condition {
                boolean: Boolean {
                    is_archived: Some(true),
                    is_starred: Some(false),
                    is_leaf: Some(true),
                    is_root: None,
                },
                context: (Some(333), Some(777)),
                weight: (Some(0.5), Some(24.0)),
                startable: (
                    None,
                    Some(models::EasyDateTime {
                        date: None,
                        time: Some(models::EasyTime {
                            h: Some(15),
                            m: None,
                        }),
                    })
                ),
                deadline: (
                    Some(models::EasyDateTime {
                        date: Some(models::EasyDate {
                            y: None,
                            m: Some(12),
                            d: None,
                        }),
                        time: None,
                    }),
                    None
                ),
                created_at: (None, None),
                updated_at: (
                    Some(models::EasyDateTime {
                        date: Some(models::EasyDate {
                            y: Some(2021),
                            m: None,
                            d: None,
                        }),
                        time: None,
                    }),
                    Some(models::EasyDateTime {
                        date: Some(models::EasyDate {
                            y: None,
                            m: None,
                            d: Some(30),
                        }),
                        time: Some(models::EasyTime {
                            h: Some(6),
                            m: None,
                        }),
                    })
                ),
                title: Some(text::Expression::Words(vec![
                    String::from("tit"),
                    String::from("le"),
                ])),
                assign: Some(text::Expression::Regex(
                    String::from(r#"double"quoted"man"#)
                )),
                link: Some(text::Expression::Regex(
                    String::from(r".*domain\.com.*\?page=[1-5]#(frag|ment)")
                )),
            },
            ""
        )));
        assert!(t_10.is_err());
    }
    #[test]
    fn t_a_quoted() {
        let t_00 = a_quoted().easy_parse(r####""""####);
        let t_01 = a_quoted().easy_parse(r####""sha""####);
        let t_02 = a_quoted().easy_parse(r####"#""#"####);
        let t_03 = a_quoted().easy_parse(r####"#"sha"#"####);
        let t_04 = a_quoted().easy_parse(r####"#"""#"####);
        let t_05 = a_quoted().easy_parse(r####"##"#""#"##"####);
        let t_06 = a_quoted().easy_parse(r####"#""##"####);
        let t_10 = a_quoted().easy_parse(r####"##""#"####);
        assert_eq!(t_00, Ok((String::new(), "")));
        assert_eq!(t_01, Ok((String::from("sha"), "")));
        assert_eq!(t_02, t_00);
        assert_eq!(t_03, t_01);
        assert_eq!(t_04, Ok((String::from(r#"""#), "")));
        assert_eq!(t_05, Ok((String::from(r##"#""#"##), "")));
        assert_eq!(t_06, Ok((String::from(""), "#")));
        assert!(t_10.is_err());
    }
    #[test]
    fn t_a_expression() {
        let t_000 = a_expression().easy_parse(r#""""#);
        let t_001 = a_expression().easy_parse(r#""title""#);
        let t_002 = a_expression().easy_parse(r#""tit le""#);
        let t_003 = a_expression().easy_parse(r#""   tit le   ""#);
        let t_004 = a_expression().easy_parse(r#""tit""le""#);
        let t_010 = a_expression().easy_parse(r#"r"""#);
        let t_011 = a_expression().easy_parse(r###"r##""##"###);
        let t_012 = a_expression().easy_parse(r#"r"re gex""#);
        let t_013 = a_expression().easy_parse(r#"r"   re gex   ""#);
        let t_014 = a_expression().easy_parse(r###"r##"   r#""#   "##"###);
        let t_015 = a_expression().easy_parse(
            r#####"r####".*"### I'm header 3".*|^(WRY{3,}\.*)?(無駄)+$"####   etc...   "#####
        );
        let t_100 = a_expression().easy_parse("");
        let t_101 = a_expression().easy_parse("double quotes lack");
        let t_110 = a_expression().easy_parse("r");
        let t_111 = a_expression().easy_parse("r#double quotes lack#");
        let t_112 = a_expression().easy_parse(r###"r##" sharp num mismatch "#   "###);
        assert_eq!(t_000, Ok((text::Expression::Words(Vec::new()), "")));
        assert_eq!(t_001, Ok((text::Expression::Words(vec![String::from("title"),]), "")));
        assert_eq!(t_002, Ok((text::Expression::Words(vec![
            String::from("tit"),
            String::from("le"),
            ]), "")));
        assert_eq!(t_003, t_002);
        assert_eq!(t_004, Ok((text::Expression::Words(vec![String::from("tit"),]), r#""le""#)));
        assert_eq!(t_010, Ok((text::Expression::Regex(String::new()), "")));
        assert_eq!(t_011, t_010);
        assert_eq!(t_012, Ok((text::Expression::Regex(String::from("re gex")), "")));
        assert_eq!(t_013, Ok((text::Expression::Regex(String::from("   re gex   ")), "")));
        assert_eq!(t_014, Ok((text::Expression::Regex(String::from(r##"   r#""#   "##)), "")));
        assert_eq!(t_015, Ok((text::Expression::Regex(String::from(
            r####".*"### I'm header 3".*|^(WRY{3,}\.*)?(無駄)+$"####
        )), "   etc...   ")));
        assert!(t_100.is_err());
        assert!(t_101.is_err());
        assert!(t_110.is_err());
        assert!(t_111.is_err());
        assert!(t_112.is_err());
    }
    #[test]
    fn t_a_graphics() {
        let t_00 = a_graphics().easy_parse("ばぶ👶aZ09!~");
        let t_10 = a_graphics().easy_parse("");
        let t_11 = a_graphics().easy_parse(" ");
        let t_12 = a_graphics().easy_parse("\n");
        let t_13 = a_graphics().easy_parse("　");
        assert_eq!(t_00, Ok((String::from("ばぶ👶aZ09!~"), "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());
    }
    #[test]
    fn t_a_boolean() {
        let t_00 = a_boolean().easy_parse("a!sl   etc...   ");
        let t_01 = a_boolean().easy_parse("a!");
        assert_eq!(t_00, Ok((Boolean {
            is_archived: Some(true),
            is_starred: Some(false),
            is_leaf: Some(true),
            is_root: None,
        }, "   etc...   ")));
        assert_eq!(t_01, Ok((Boolean {
            is_archived: Some(true),
            is_starred: None,
            is_leaf: None,
            is_root: None,
        }, "!")));
    }
    #[test]
    fn t_a_non_nega_i() {
        let t_00 = a_non_nega_i().easy_parse("000");
        let t_10 = a_non_nega_i().easy_parse("");
        let t_11 = a_non_nega_i().easy_parse("-1");
        let t_12 = a_non_nega_i().easy_parse("   0");
        assert_eq!(t_00, Ok((0i32, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
    }
    #[test]
    fn t_a_non_nega_f() {
        let t_00 = a_non_nega_f().easy_parse("6");
        let t_01 = a_non_nega_f().easy_parse("6.0");
        let t_02 = a_non_nega_f().easy_parse("6.");
        let t_03 = a_non_nega_f().easy_parse(".6");
        let t_04 = a_non_nega_f().easy_parse("6..0");
        let t_05 = a_non_nega_f().easy_parse("6.0.6");
        let t_06 = a_non_nega_f().easy_parse("6.0e-01");
        let t_10 = a_non_nega_f().easy_parse(".");
        let t_11 = a_non_nega_f().easy_parse("   6");
        assert_eq!(t_00, Ok((6.0f32, "")));
        assert_eq!(t_01, Ok((6.0f32, "")));
        assert_eq!(t_02, Ok((6.0f32, "")));
        assert_eq!(t_03, Ok((0.6f32, "")));
        assert_eq!(t_04, Ok((6.0f32, ".0")));
        assert_eq!(t_05, Ok((6.0f32, ".6")));
        assert_eq!(t_06, Ok((6.0f32, "e-01")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_a_datetime() {
        let t_00 = a_datetime().easy_parse("//T:");
        let t_01 = a_datetime().easy_parse("//");
        let t_02 = a_datetime().easy_parse(":");
        let t_10 = a_datetime().easy_parse("");
        let t_11 = a_datetime().easy_parse("T");
        let t_12 = a_datetime().easy_parse("//:");
        let t_13 = a_datetime().easy_parse("//T");
        let t_14 = a_datetime().easy_parse("T:");
        assert_eq!(t_00, Ok((models::EasyDateTime {
            date: Some(models::EasyDate::default()),
            time: Some(models::EasyTime::default()),
        }, "")));
        assert_eq!(t_01, Ok((models::EasyDateTime {
            date: Some(models::EasyDate::default()),
            time: None,
        }, "")));
        assert_eq!(t_02, Ok((models::EasyDateTime {
            date: None,
            time: Some(models::EasyTime::default()),
        }, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert_eq!(t_12, Ok((models::EasyDateTime {
            date: Some(models::EasyDate::default()),
            time: None,
        }, ":"))); // can cause error next
        assert_eq!(t_13, Ok((models::EasyDateTime {
            date: Some(models::EasyDate::default()),
            time: None,
        }, "T"))); // can cause error next
        assert!(t_14.is_err());
    }
    #[test]
    fn t_a_date() {
        let t_00 = a_date().easy_parse("//");
        let t_01 = a_date().easy_parse( // TODO limit to "9999/12/31" in following process
            "294277/01/01"
        );
        let t_02 = a_date().easy_parse( // TODO limit to "1000/01/01" in following process
            "0001/01/01"
        );
        let t_10 = a_date().easy_parse("");
        let t_11 = a_date().easy_parse("12/31");
        let t_12 = a_date().easy_parse("2021-01-07");
        assert_eq!(t_00, Ok((models::EasyDate {
            y: None,
            m: None,
            d: None,
        }, "")));
        assert!(t_01.is_ok());
        assert!(t_02.is_ok());
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
    }
    #[test]
    fn t_a_time() {
        let t_00 = a_time().easy_parse(":");
        let t_01 = a_time().easy_parse("00:000");
        let t_02 = a_time().easy_parse("023:00059");
        let t_03 = a_time().easy_parse("24:");
        let t_04 = a_time().easy_parse(":60");
        let t_05 = a_time().easy_parse("23:59:59");
        assert_eq!(t_00, Ok((models::EasyTime {
            h: None,
            m: None,
        }, "")));
        assert_eq!(t_01, Ok((models::EasyTime {
            h: Some(0),
            m: Some(0),
        }, "")));
        assert_eq!(t_02, Ok((models::EasyTime {
            h: Some(23),
            m: Some(59),
        }, "")));
        assert_eq!(t_03, Ok((models::EasyTime {
            h: Some(24),
            m: None,
        }, "")));
        assert_eq!(t_04, Ok((models::EasyTime {
            h: None,
            m: Some(60),
        }, "")));
        assert_eq!(t_05, Ok((models::EasyTime {
            h: Some(23),
            m: Some(59),
        }, ":59")));
    }
    #[test]
    fn t_a_req_task() {
        let t_00 = a_req_task().easy_parse("title");
        let t_01 = a_req_task().easy_parse("    title");
        let t_02 = a_req_task().easy_parse("    title http://localhost");
        let t_03 = a_req_task().easy_parse("    title\n    http://localhost");
        let t_10 = a_req_task().easy_parse("");
        let t_11 = a_req_task().easy_parse("      ambiguous indent");
        let t_12 = a_req_task().easy_parse("    http://localhost    as no title");
        let t_13 = a_req_task().easy_parse("    title\n    another    http://localhost");
        let t_14 = a_req_task().easy_parse("    title\n    http://localhost    extra");
        assert_eq!(t_00, Ok((ReqTask {
            indent: 0,
            attribute: Attribute {
                is_starred: false,
                id: None,
                weight: None,
                joint_head: None,
                joint_tail: None,
                assign: None,
                startable: None,
                deadline: None,
                title: vec![String::from("title"),]
            },
            link: None,
        }, "")));
        assert_eq!(t_01, Ok((ReqTask {
            indent: 1,
            attribute: Attribute {
                is_starred: false,
                id: None,
                weight: None,
                joint_head: None,
                joint_tail: None,
                assign: None,
                startable: None,
                deadline: None,
                title: vec![String::from("title"),]
            },
            link: None,
        }, "")));
        assert_eq!(t_02, Ok((ReqTask {
            indent: 1,
            attribute: Attribute {
                is_starred: false,
                id: None,
                weight: None,
                joint_head: None,
                joint_tail: None,
                assign: None,
                startable: None,
                deadline: None,
                title: vec![String::from("title"),]
            },
            link: Some(String::from("http://localhost")),
        }, "")));
        assert_eq!(t_03, t_02);
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert_eq!(t_13, Ok((ReqTask {
            indent: 1,
            attribute: Attribute {
                is_starred: false,
                id: None,
                weight: None,
                joint_head: None,
                joint_tail: None,
                assign: None,
                startable: None,
                deadline: None,
                title: vec![String::from("title"),]
            },
            link: None,
        }, "    another    http://localhost")));
        assert!(t_14.is_err());
    }
    #[test]
    fn t_a_indent() {
        let t_00 = a_indent().easy_parse("g");
        let t_01 = a_indent().easy_parse("    g");
        let t_02 = a_indent().easy_parse("        g    ");
        let t_10 = a_indent().easy_parse("");
        let t_11 = a_indent().easy_parse("\n");
        let t_12 = a_indent().easy_parse("     g");
        assert_eq!(t_00, Ok((0, "g")));
        assert_eq!(t_01, Ok((1, "g")));
        assert_eq!(t_02, Ok((2, "g    ")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
    }
    #[test]
    fn t_a_attribute() {
        let t_00 = a_attribute().easy_parse("");
        let t_01 = a_attribute().easy_parse("\n");
        let t_02 = a_attribute().easy_parse("https://");
        let t_03 = a_attribute().easy_parse(" ");
        let t_04 = a_attribute().easy_parse("# ] * - / // T : $ @ [");
        let t_05 = a_attribute().easy_parse("#333 h] something * 15:- 魁 -/12/ [t $5 great $530000. @satun ⚡ \n");
        let t_06 = a_attribute().easy_parse("//T: //T //: // T: T :");
        let t_07 = a_attribute().easy_parse("//T- //:- T:- T- -");
        let t_08 = a_attribute().easy_parse("-T: -T -");
        let t_10 = a_attribute().easy_parse("head or tail ? [joint]");
        let t_11 = a_attribute().easy_parse("startable or deadline ? -2021/01/07-");
        let t_12 = a_attribute().easy_parse("-//T title");
        let t_13 = a_attribute().easy_parse("-//: title");
        assert_eq!(t_00, Ok((Attribute::default(), "")));
        assert_eq!(t_01, Ok((Attribute::default(), "\n")));
        assert_eq!(t_02, Ok((Attribute::default(), "https://")));
        assert_eq!(t_03, t_00);
        assert_eq!(t_04, Ok((Attribute {
            is_starred: true,
            id: None,
            weight: None,
            joint_head: None,
            joint_tail: None,
            assign: None,
            startable: None,
            deadline: None,
            title: vec![
                String::from("["),
                String::from("@"),
                String::from("$"),
                String::from(":"),
                String::from("T"),
                String::from("//"),
                String::from("/"),
                String::from("-"),
                String::from("]"),
                String::from("#"),
                ]
        }, "")));
        assert_eq!(t_05, Ok((Attribute {
            is_starred: true,
            id: Some(333),
            weight: Some(5.0),
            joint_head: Some(String::from("h")),
            joint_tail: Some(String::from("t")),
            assign: Some(String::from("satun")),
            startable: Some(models::EasyDateTime {
                date: None,
                time: Some(models::EasyTime {
                    h: Some(15),
                    m: None,
                }),
            }),
            deadline: Some(models::EasyDateTime {
                date: Some(models::EasyDate {
                    y: None,
                    m: Some(12),
                    d: None,
                }),
                time: None,
            }),
            title: vec![
                String::from("⚡"),
                String::from("great"),
                String::from("魁"),
                String::from("something"),
                ]
        }, "\n")));
        assert_eq!(t_06, Ok((Attribute {
            is_starred: false,
            id: None,
            weight: None,
            joint_head: None,
            joint_tail: None,
            assign: None,
            startable: None,
            deadline: None,
            title: vec![
                // String::from(""),
                String::from(":"),
                String::from("T"),
                String::from("T:"),
                String::from("//"),
                String::from("//:"),
                String::from("//T"),
                String::from("//T:"),
                ]
        }, "")));
        assert_eq!(t_07, Ok((Attribute {
            is_starred: false,
            id: None,
            weight: None,
            joint_head: None,
            joint_tail: None,
            assign: None,
            startable: None,
            deadline: None,
            title: vec![
                String::from("-"),
                // String::from(":-"),
                String::from("T-"),
                String::from("T:-"),
                // String::from("//-"),
                String::from("//:-"),
                String::from("//T-"),
                // String::from("//T:-"),
                ]
        }, "")));
        assert_eq!(t_08, Ok((Attribute {
            is_starred: false,
            id: None,
            weight: None,
            joint_head: None,
            joint_tail: None,
            assign: None,
            startable: None,
            deadline: None,
            title: vec![
                String::from("-"),
                // String::from("-:"),
                String::from("-T"),
                String::from("-T:"),
                // String::from("-//"),
                // String::from("-//:"),
                // String::from("-//T"),
                // String::from("-//T:"),
                ]
        }, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());
    }
    #[test]
    fn t_a_link() {
        let t_00 = a_link().easy_parse("https://");
        let t_01 = a_link().easy_parse("http://ニホンゴ");
        let t_10 = a_link().easy_parse("");
        let t_11 = a_link().easy_parse("   https://");
        assert_eq!(t_00, Ok((String::from("https://"), "")));
        assert_eq!(t_01, Ok((String::from("http://"), "ニホンゴ")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
}
