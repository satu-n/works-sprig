use combine::{
    Parser, Stream, attempt, choice, eof, from_str, many, many1,
    optional, parser, satisfy, sep_by1, skip_many, skip_many1, token,
};
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
        let req = req_().parse(s)?.0;
        if let Req::Tasks(ts) = &req {
            if ts.tasks.iter().any(|t| t.attribute.title.is_empty()) {
                return Err(Self::Err::BadRequest("there is a task with no title.".into()))
            }
        }
        Ok(req)
    }
}

impl text::ReqBody {
    pub fn wash(&self) -> String {
        self
        .remove_comments()
        .trim()
        .lines()
        .map(|s| s.trim_end().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join("\n")
    }
    fn remove_comments(&self) -> String {
        let prefix = "<!--";
        let suffix = "-->";
        let mut src = &*self.text;
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
    fn req_[Input]()(Input) -> Req
    where [ Input: Stream<Token = char> ] {
        choice((
            token('/').with(optional(req_command_())).map(|opt| {
                Req::Command(opt.unwrap_or(ReqCommand::Help))
            }),
            many(req_task_()).map(|ts| {
                Req::Tasks(ReqTasks {
                    tasks: ts,
                })
            }),
        ))
    }
}
parser! {
    fn req_command_[Input]()(Input) -> ReqCommand
    where [ Input: Stream<Token = char> ] {
        choice((
            token('u').with(optional(spaces1_().with(req_user_()))).map(|opt| {
                ReqCommand::User(opt.unwrap_or(ReqUser::Info))
            }),
            token('s').with(conditions_()).map(|x| {
                ReqCommand::Search(x)
            }),
            string("tutorial").map(|_| ReqCommand::Tutorial),
            string("coffee").map(|_| ReqCommand::Coffee),
        ))
    }
}
parser! {
    fn req_user_[Input]()(Input) -> ReqUser
    where [ Input: Stream<Token = char> ] {
        token('-').with(req_modify_()).map(|x| ReqUser::Modify(x))
    }
}
parser! {
    fn req_modify_[Input]()(Input) -> ReqModify
    where [ Input: Stream<Token = char> ] {
        choice((
            token('e').with(spaces1_().with(ascii_graphics1_())).map(|x| ReqModify::Email(x)),
            token('p').with(spaces1_().with(password_set_())).map(|x| ReqModify::Password(x)),
            token('n').with(spaces1_().with(ascii_graphics1_())).map(|x| ReqModify::Name(x)),
            token('t').with(spaces1_().with(timescale_())).map(|x| ReqModify::Timescale(x)),
            token('a').with(many(spaces1_().with(req_allocation_()))).map(|x| ReqModify::Allocations(x)),
        ))
    }
}
parser! {
    fn password_set_[Input]()(Input) -> PasswordSet
    where [ Input: Stream<Token = char> ] {
        ascii_graphics1_().skip(spaces1_())
        .and(ascii_graphics1_()).skip(spaces1_())
        .and(ascii_graphics1_())
        .map(|((o, n), c)| PasswordSet {
            old: o,
            new: n,
            confirmation: c,
        })
    }
}
parser! {
    fn timescale_[Input]()(Input) -> Timescale
    where [ Input: Stream<Token = char> ] {
        let p = |t: Timescale| string(t.as_str()).map(move |_| t.clone());
        choice((
            p(Timescale::Year),
            p(Timescale::Quarter),
            p(Timescale::Month),
            p(Timescale::Week),
            p(Timescale::Day),
            p(Timescale::Hours),
            p(Timescale::Hour),
            p(Timescale::Minutes),
            p(Timescale::Minute),
            p(Timescale::Second),
        ))
    }
}
parser! {
    fn req_allocation_[Input]()(Input) -> ReqAllocation
    where [ Input: Stream<Token = char> ] {
        non_nega_i_().skip(token(':')).and(non_nega_i_()).skip(token('-')).and(non_nega_i_()).skip(token('h'))
        .map(|((open_h, open_m), hours)| ReqAllocation {
            open_h: open_h,
            open_m: open_m,
            hours: hours,
        })
    }
}
parser! {
    fn conditions_[Input]()(Input) -> Condition
    where [ Input: Stream<Token = char> ] {
        many(spaces1_().with(condition_()))
    }
}
parser! {
    fn condition_[Input]()(Input) -> Condition
    where [ Input: Stream<Token = char> ] {
        choice((
            token('-').with(booleans1_()).map(|x| {
                let mut condition = Condition::default();
                condition.boolean = x;
                condition
            }),
            attempt(
                optional(non_nega_i_().skip(token('<'))).skip(token('#')).and(optional(token('<').with(non_nega_i_())))
            ).map(|(l, r)| {
                let mut condition = Condition::default();
                condition.context = (l, r);
                condition
            }),
            attempt(
                optional(non_nega_f_().skip(token('<'))).skip(token('w')).and(optional(token('<').with(non_nega_f_())))
            ).map(|(l, r)| {
                let mut condition = Condition::default();
                condition.weight = (l, r);
                condition
            }),
            attempt(
                optional(datetime_().skip(token('<'))).and(choice([
                    token('s'),
                    token('d'),
                    token('c'),
                    token('u'),
                ])).and(optional(token('<').with(datetime_())))
            ).map(|((l, c), r)| {
                let mut condition = Condition::default();
                match c {
                    's' => condition.startable = (l, r),
                    'd' => condition.deadline = (l, r),
                    'c' => condition.created_at = (l, r),
                    'u' => condition.updated_at = (l, r),
                    _ => unreachable!()
                }
                condition
            }),
            attempt(optional(token('@').or(token('&'))).and(expression_())).map(|(opt, expr)| {
                let mut condition = Condition::default();
                match opt {
                    None => condition.title = Some(expr),
                    Some('@') => condition.assign = Some(expr),
                    Some('&') => condition.link = Some(expr),
                    _ => unreachable!()
                }
                condition
            }),
        ))
    }
}
// TODO enum ConditionItem
impl std::iter::Extend<Self> for Condition {
    fn extend<T: IntoIterator<Item=Self>>(&mut self, iter: T) {
        for item in iter {
            self.boolean.extend(std::iter::once(item.boolean));
            if self.context.lt(&item.context) { self.context = item.context };
            if self.weight.lt(&item.weight) { self.weight = item.weight };
            if self.startable.lt(&item.startable) { self.startable = item.startable };
            if self.deadline.lt(&item.deadline) { self.deadline = item.deadline };
            if self.created_at.lt(&item.created_at) { self.created_at = item.created_at };
            if self.updated_at.lt(&item.updated_at) { self.updated_at = item.updated_at };
            if self.title.lt(&item.title) { self.title = item.title };
            if self.assign.lt(&item.assign) { self.assign = item.assign };
            if self.link.lt(&item.link) { self.link = item.link };
        }
    }
}
parser! {
    fn expression_[Input]()(Input) -> text::Expression
    where [ Input: Stream<Token = char> ] {
        attempt(optional(token('r')).and(quoted_())).map(|(opt, q)| {
            match opt {
                None => text::Expression::Words(q.split_whitespace().map(|s| s.to_string()).collect()),
                _ => text::Expression::Regex(q),
            }
        })
    }
}
parser! { // ####" any "####
    fn quoted_[Input]()(Input) -> String
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
    fn booleans1_[Input]()(Input) -> Boolean
    where [ Input: Stream<Token = char> ] {
        many1(boolean_())
    }
}
parser! {
    fn boolean_[Input]()(Input) -> Boolean
    where [ Input: Stream<Token = char> ] {
        optional(token('!')).and(choice((
            token('a'),
            token('s'),
            token('l'),
            token('r'),
        ))).map(|(not, c)| {
            let mut boolean = Boolean::default();
            let some = Some(not.is_none());
            match c {
                'a' => boolean.is_archived = some,
                's' => boolean.is_starred = some,
                'l' => boolean.is_leaf = some,
                'r' => boolean.is_root = some,
                _ => unreachable!()
            }
            boolean
        })
    }
}
// TODO enum BooleanItem
impl std::iter::Extend<Self> for Boolean {
    fn extend<T: IntoIterator<Item=Self>>(&mut self, iter: T) {
        for item in iter {
            if self.is_archived.lt(&item.is_archived) { self.is_archived = item.is_archived };
            if self.is_starred.lt(&item.is_starred) { self.is_starred = item.is_starred };
            if self.is_leaf.lt(&item.is_leaf) { self.is_leaf = item.is_leaf };
            if self.is_root.lt(&item.is_root) { self.is_root = item.is_root };
        }
    }
}
parser! {
    fn datetime_[Input]()(Input) -> models::EasyDateTime
    where [ Input: Stream<Token = char> ] {
        choice((
            attempt(date_().skip(token('T')).and(time_())).map(|(d, t)| {
                models::EasyDateTime {
                    date: Some(d),
                    time: Some(t),
                }
            }),
            date_().map(|d| {
                models::EasyDateTime {
                    date: Some(d),
                    time: None,
                }
            }),
            time_().map(|t| {
                models::EasyDateTime {
                    date: None,
                    time: Some(t),
                }
            }),
        ))
    }
}
parser! {
    fn date_[Input]()(Input) -> models::EasyDate
    where [ Input: Stream<Token = char> ] {
        attempt(optional(non_nega_i_()).skip(token('/')).and(optional(non_nega_i_())).skip(token('/')).and(optional(non_nega_i_())))
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
    fn time_[Input]()(Input) -> models::EasyTime
    where [ Input: Stream<Token = char> ] {
        attempt(optional(non_nega_i_()).skip(token(':')).and(optional(non_nega_i_())))
        .map(|(h, m)| {
            models::EasyTime {
                h: h,
                m: m,
            }
        })
    }
}
parser! {
    fn req_task_[Input]()(Input) -> ReqTask
    where [ Input: Stream<Token = char> ] {
        indents_()
        .and(attributes1_())
        .and(optional(attempt(newline().with(inline_spaces_().with(link_())))))
        .skip(choice((
            newline().map(|_| ()),
            eof(),
        )))
        .map(|((indent, attribute), link)| ReqTask {
            indent: indent,
            attribute: attribute,
            link: link,
        })
    }
}
parser! {
    fn indents_[Input]()(Input) -> i32
    where [ Input: Stream<Token = char> ] {
        many(indent_())
    }
}
#[derive(Debug, PartialEq)]
struct Indent;
parser! {
    fn indent_[Input]()(Input) -> Indent
    where [ Input: Stream<Token = char> ] {
        choice((
            token('\t').map(|_| Indent),
            skip_count_min_max(4, 4, token(' ')).map(|_| Indent),
        ))
    }
}
impl std::iter::Extend<Indent> for i32 {
    fn extend<T: IntoIterator<Item=Indent>>(&mut self, iter: T) {
        for _ in iter {
            *self += 1;
        }
    }
}
// TODO enum AttributeItem
impl std::iter::Extend<Self> for Attribute {
    fn extend<T: IntoIterator<Item=Self>>(&mut self, iter: T) {
        for item in iter {
            if item.is_starred { self.is_starred = true };
            if let Some(x) = item.id { self.id = Some(x) };
            if let Some(x) = item.weight { self.weight = Some(x) };
            if let Some(x) = item.joint_head { self.joint_head = Some(x) };
            if let Some(x) = item.joint_tail { self.joint_tail = Some(x) };
            if let Some(x) = item.assign { self.assign = Some(x) };
            if let Some(x) = item.startable { self.startable = Some(x) };
            if let Some(x) = item.deadline { self.deadline = Some(x) };
            if !item.title.is_empty() {
                if !self.title.is_empty() {
                    self.title.push(' ');
                }
                self.title.push_str(&item.title)
            };
        }
    }
}
parser! {
    fn attributes1_[Input]()(Input) -> Attribute
    where [ Input: Stream<Token = char> ] {
        sep_by1(attribute_(), inline_spaces1_())
    }
}
parser! {
    fn attribute_[Input]()(Input) -> Attribute
    where [ Input: Stream<Token = char> ] {
        choice((
            token('*').map(|_| {
                let mut attribute = Attribute::default();
                attribute.is_starred = true;
                attribute
            }),
            token('#').with(non_nega_i_()).map(|i| {
                let mut attribute = Attribute::default();
                attribute.id = Some(i);
                attribute
            }),
            token('$').with(non_nega_f_()).map(|f| {
                let mut attribute = Attribute::default();
                attribute.weight = Some(f);
                attribute
            }),
            token('@').with(ascii_graphics1_()).map(|ag| {
                let mut attribute = Attribute::default();
                attribute.assign = Some(ag);
                attribute
            }),
            token('-').with(datetime_()).map(|dt| {
                let mut attribute = Attribute::default();
                attribute.deadline = Some(dt);
                attribute
            }),
            token('[').with(graphics1_not_joint_()).map(|g| {
                let mut attribute = Attribute::default();
                attribute.joint_tail = Some(g);
                attribute
            }),
            attempt(datetime_().skip(token('-'))).map(|dt| {
                let mut attribute = Attribute::default();
                attribute.startable = Some(dt);
                attribute
            }),
            attempt(graphics1_not_joint_().skip(token(']'))).map(|g| {
                let mut attribute = Attribute::default();
                attribute.joint_head = Some(g);
                attribute
            }),
            graphics1_().map(|g| {
                let mut attribute = Attribute::default();
                attribute.title = g;
                attribute
            }),
        ))
    }
}
parser! {
    fn link_[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        attempt(recognize((
            string("http"),
            optional(token('s')),
            string("://"),
            ascii_graphics_(),
        )))
    }
}
parser! {
    fn spaces1_[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        skip_many1(space())
    }
}
parser! {
    fn inline_space_[Input]()(Input) -> char
    where [ Input: Stream<Token = char> ] {
        satisfy(|c: char| c.is_whitespace() && c != '\n')
    }
}
parser! {
    fn inline_spaces_[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        skip_many(inline_space_())
    }
}
parser! {
    fn inline_spaces1_[Input]()(Input) -> ()
    where [ Input: Stream<Token = char> ] {
        skip_many1(inline_space_())
    }
}
parser! {
    fn ascii_graphic_[Input]()(Input) -> char
    where [ Input: Stream<Token = char> ] {
        satisfy(|c: char| c.is_ascii_graphic())
    }
}
parser! {
    fn ascii_graphics_[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many(ascii_graphic_())
    }
}
parser! {
    fn ascii_graphics1_[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many1(ascii_graphic_())
    }
}
parser! {
    fn graphic_[Input]()(Input) -> char
    where [ Input: Stream<Token = char> ] {
        satisfy(|c: char| !c.is_whitespace() && !c.is_control())
    }
}
parser! {
    fn graphics_[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many(graphic_())
    }
}
parser! {
    fn graphics1_[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many1(graphic_())
    }
}
parser! {
    fn graphic_not_joint_[Input]()(Input) -> char
    where [ Input: Stream<Token = char> ] {
        satisfy(|c: char| !c.is_whitespace() && !c.is_control() && !"[]".contains(c))
    }
}
parser! {
    fn graphics1_not_joint_[Input]()(Input) -> String
    where [ Input: Stream<Token = char> ] {
        many1(graphic_not_joint_())
    }
}
parser! {
    fn non_nega_i_[Input]()(Input) -> i32
    where [ Input: Stream<Token = char> ] {
        from_str(many1::<String, _, _>(digit()))
    }
}
parser! {
    fn non_nega_f_[Input]()(Input) -> f32
    where [ Input: Stream<Token = char> ] {
        from_str(recognize::<String, _, _>((
            many::<String, _, _>(digit()),
            optional(token('.')),
            many::<String, _, _>(digit()),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::EasyParser;

    fn req_body() -> text::ReqBody {
        text::ReqBody { text: String::from(
            " \n\r\n pon  \n\r\n <!-- \n\r\n cho  \n\r\n <!-- \n\r\n cho  \n\r\n -->  \n\r\n pon  \n\r\n -->  \n\r\n pon  \n\r\n <!-- \n\r\n cho  \n\r\n "
        )}
    }
    #[test]
    fn t_washer_remove_comments() {
        let t_00 = req_body().remove_comments();
        assert_eq!(t_00, String::from(
            " \n\r\n pon  \n\r\n   \n\r\n pon  \n\r\n -->  \n\r\n pon  \n\r\n "
        ));
    }
    #[test]
    fn t_washer_wash() {
        let t_00 = req_body().wash();
        assert_eq!(t_00, String::from(
            "pon\n pon\n -->\n pon"
        ));
    }
    #[test]
    fn t_req_() {
        let t_00 = req_().easy_parse("");
        let t_01 = req_().easy_parse("/");
        let t_10 = req_().easy_parse("/12/- * task");
        assert_eq!(t_00, Ok((Req::Tasks(ReqTasks { tasks: Vec::new() }), "")));
        assert_eq!(t_01, Ok((Req::Command(ReqCommand::Help), "")));
        assert_eq!(t_10, Ok((Req::Command(ReqCommand::Help), "12/- * task")));
    }
    #[test]
    fn t_req_command_() {
        let t_01 = req_command_().easy_parse("u");
        let t_02 = req_command_().easy_parse("s");
        let t_03 = req_command_().easy_parse("tutorial");
        let t_04 = req_command_().easy_parse("coffee");
        let t_10 = req_command_().easy_parse(" ");
        let t_11 = req_command_().easy_parse("x");
        assert_eq!(t_01, Ok((ReqCommand::User(ReqUser::Info), "")));
        assert_eq!(t_02, Ok((ReqCommand::Search(Condition::default()), "")));
        assert_eq!(t_03, Ok((ReqCommand::Tutorial, "")));
        assert_eq!(t_04, Ok((ReqCommand::Coffee, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_req_user_() {
        let t_10 = req_user_().easy_parse("x");
        assert!(t_10.is_err());
    }
    #[test]
    fn t_req_modify_() {
        let t_00 = req_modify_().easy_parse("n   satun__   etc...   ");
        let t_10 = req_modify_().easy_parse("");
        let t_11 = req_modify_().easy_parse(" ");
        let t_12 = req_modify_().easy_parse("x");
        assert_eq!(t_00, Ok((ReqModify::Name(String::from("satun__")), "   etc...   ")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
    }
    #[test]
    fn t_password_set_() {
        let t_00 = password_set_().easy_parse(
            r##"old!"#$%&'()*+,-./   new0123456789   confirmation:;<=>?@   etc..."##
        );
        let t_10 = password_set_().easy_parse("");
        let t_11 = password_set_().easy_parse("   old new confirmation");
        let t_12 = password_set_().easy_parse("old new_without_confirmation");
        let t_13 = password_set_().easy_parse("ぱすわーどに　和文字　とな？");
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
    fn t_timescale_() {
        let t_00 = timescale_().easy_parse("15mm   etc...");
        let t_10 = timescale_().easy_parse("");
        let t_11 = timescale_().easy_parse("   15m");
        let t_12 = timescale_().easy_parse("y");
        let t_13 = timescale_().easy_parse("恒河沙");
        assert_eq!(t_00, Ok((Timescale::Minutes, "m   etc...")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());
    }
    #[test]
    fn t_conditions_() {
        let t_00 = conditions_().easy_parse("");
        let t_02 = conditions_().easy_parse(" # w");
        let t_03 = conditions_().easy_parse(
            r##" 333<#<777 -a!s -l .5<w<24 s<15: /12/<d c 2021//<u<//30T6:"##
        );
        let t_04 = conditions_().easy_parse(
            r##" 333<#<777 -a!s -l .5<w<24 s<15: /12/<d c 2021//<u<//30T6: "tit le" @r#"double"quoted"man"# &r".*domain\.com.*\?page=[1-5]#(frag|ment)""##
        );
        let t_10 = conditions_().easy_parse(" title");
        let t_11 = conditions_().easy_parse(" ");
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
        assert!(t_11.is_err());
    }
    #[test]
    fn t_expression_() {
        let t_000 = expression_().easy_parse(r#""""#);
        let t_001 = expression_().easy_parse(r#""title""#);
        let t_002 = expression_().easy_parse(r#""tit le""#);
        let t_003 = expression_().easy_parse(r#""   tit le   ""#);
        let t_004 = expression_().easy_parse(r#""tit""le""#);
        let t_010 = expression_().easy_parse(r#"r"""#);
        let t_011 = expression_().easy_parse(r###"r##""##"###);
        let t_012 = expression_().easy_parse(r#"r"re gex""#);
        let t_013 = expression_().easy_parse(r#"r"   re gex   ""#);
        let t_014 = expression_().easy_parse(r###"r##"   r#""#   "##"###);
        let t_015 = expression_().easy_parse(
            r#####"r####".*"### I'm header 3".*|^(WRY{3,}\.*)?(無駄)+$"####   etc...   "#####
        );
        let t_100 = expression_().easy_parse("");
        let t_101 = expression_().easy_parse("double quotes lack");
        let t_110 = expression_().easy_parse("r");
        let t_111 = expression_().easy_parse("r#double quotes lack#");
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
    }
    #[test]
    fn t_quoted_() {
        let t_00 = quoted_().easy_parse(r####""""####);
        let t_01 = quoted_().easy_parse(r####""sha""####);
        let t_02 = quoted_().easy_parse(r####"#""#"####);
        let t_03 = quoted_().easy_parse(r####"#"sha"#"####);
        let t_04 = quoted_().easy_parse(r####"#"""#"####);
        let t_05 = quoted_().easy_parse(r####"##"#""#"##"####);
        let t_06 = quoted_().easy_parse(r####"#""##"####);
        let t_10 = quoted_().easy_parse(r####"##""#"####);
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
    fn t_booleans1_() {
        let t_00 = booleans1_().easy_parse("a!sl   etc...   ");
        let t_10 = booleans1_().easy_parse("a!");
        assert_eq!(t_00, Ok((Boolean {
            is_archived: Some(true),
            is_starred: Some(false),
            is_leaf: Some(true),
            is_root: None,
        }, "   etc...   ")));
        assert!(t_10.is_err());
    }
    #[test]
    fn t_datetime_() {
        let t_00 = datetime_().easy_parse("//T:");
        let t_01 = datetime_().easy_parse("//");
        let t_02 = datetime_().easy_parse(":");
        let t_10 = datetime_().easy_parse("");
        let t_11 = datetime_().easy_parse("T");
        let t_12 = datetime_().easy_parse("//:");
        let t_13 = datetime_().easy_parse("//T");
        let t_14 = datetime_().easy_parse("T:");
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
    fn t_date_() {
        let t_00 = date_().easy_parse("//");
        let t_01 = date_().easy_parse( // TODO limit to "9999/12/31" in following process
            "294277/01/01"
        );
        let t_02 = date_().easy_parse( // TODO limit to "1000/01/01" in following process
            "0001/01/01"
        );
        let t_10 = date_().easy_parse("");
        let t_11 = date_().easy_parse("12/31");
        let t_12 = date_().easy_parse("2021-01-07");
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
    fn t_time_() {
        let t_00 = time_().easy_parse(":");
        let t_01 = time_().easy_parse("00:000");
        let t_02 = time_().easy_parse("023:00059");
        let t_03 = time_().easy_parse("24:");
        let t_04 = time_().easy_parse(":60");
        let t_05 = time_().easy_parse("23:59:59");
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
    fn t_req_task_() {
        let t_01 = req_task_().easy_parse("title");
        let t_02 = req_task_().easy_parse("\t\ttitle");
        let t_03 = req_task_().easy_parse("    title http://localhost");
        let t_04 = req_task_().easy_parse("    title\n    http://localhost");
        let t_10 = req_task_().easy_parse("");
        let t_11 = req_task_().easy_parse("      ambiguous indent");
        let t_13 = req_task_().easy_parse("    title\n    some    http://localhost");
        let t_14 = req_task_().easy_parse("    title\n    http://localhost    some");
        assert_eq!(t_01, Ok((ReqTask {
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
                title: String::from("title"),
            },
            link: None,
        }, "")));
        assert_eq!(t_02, Ok((ReqTask {
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
                title: String::from("title"),
            },
            link: None,
        }, "")));
        assert_eq!(t_03, Ok((ReqTask {
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
                title: String::from("title http://localhost"), // inline links fall into title
            },
            link: None,
        }, "")));
        assert_eq!(t_04, Ok((ReqTask {
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
                title: String::from("title"),
            },
            link: Some(String::from("http://localhost")), // ok
        }, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
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
                title: String::from("title"),
            },
            link: None,
        }, "    some    http://localhost")));
        assert!(t_14.is_err());
    }
    #[test]
    fn t_indents_() {
        let t_00 = indents_().easy_parse("    g");
        let t_01 = indents_().easy_parse("\t\tg    ");
        let t_03 = indents_().easy_parse("");
        let t_04 = indents_().easy_parse("\n");
        let t_10 = indents_().easy_parse("     g");
        assert_eq!(t_00, Ok((1, "g")));
        assert_eq!(t_01, Ok((2, "g    ")));
        assert_eq!(t_03, Ok((0, "")));
        assert_eq!(t_04, Ok((0, "\n")));
        assert!(t_10.is_err());
    }
    #[test]
    fn t_attributes1_() {
        let t_00 = attributes1_().easy_parse("https://");
        let t_02 = attributes1_().easy_parse("#333 h] something * 15:- 魁 -/12/ [t $5 great $530000. @satun ⚡");
        let t_03 = attributes1_().easy_parse("//T: //T //: // T: T :");
        let t_04 = attributes1_().easy_parse("//T- //:- T:- T-");
        let t_10 = attributes1_().easy_parse("");
        let t_11 = attributes1_().easy_parse(" ");
        let t_12 = attributes1_().easy_parse("\n");
        let t_13 = attributes1_().easy_parse("[joint]");
        let t_14 = attributes1_().easy_parse("-2021/01/07-");
        let t_15 = attributes1_().easy_parse("-//T title");
        let t_16 = attributes1_().easy_parse("-//: title");
        let t_17 = attributes1_().easy_parse("#");
        let t_18 = attributes1_().easy_parse("]");
        let t_19 = attributes1_().easy_parse("[");
        let t_20 = attributes1_().easy_parse("$");
        let t_21 = attributes1_().easy_parse("@");
        let t_22 = attributes1_().easy_parse("-T: -T");
        let mut attr = Attribute::default();
        assert_eq!(t_00, Ok(({ attr.title = String::from("https://"); attr }, "")));
        assert_eq!(t_02, Ok((Attribute {
            is_starred: true,
            id: Some(333),
            weight: Some(530000.0),
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
            title: String::from("something 魁 great ⚡"),
        }, "")));
        let mut attr = Attribute::default();
        assert_eq!(t_03, Ok(({ attr.title = String::from("//T: //T //: // T: T :"); attr }, "")));
        let mut attr = Attribute::default();
        assert_eq!(t_04, Ok(({ attr.title = String::from("//T- //:- T:- T-"); attr }, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_ok());
        assert_eq!(t_13.unwrap().1, "]");
        assert!(t_14.is_ok());
        assert_eq!(t_14.unwrap().1, "-");
        assert!(t_15.is_ok());
        assert_eq!(t_15.unwrap().1, "T title");
        assert!(t_16.is_ok());
        assert_eq!(t_16.unwrap().1, ": title");
        assert!(t_17.is_err());
        let mut attr = Attribute::default();
        assert_eq!(t_18, Ok(({ attr.title = String::from("]"); attr }, "")));
        assert!(t_19.is_err());
        assert!(t_20.is_err());
        assert!(t_21.is_err());
        assert!(t_22.is_err());
    }
    #[test]
    fn t_link_() {
        let t_00 = link_().easy_parse("https://");
        let t_01 = link_().easy_parse("http://subdomain.domaiニホンゴ");
        let t_10 = link_().easy_parse("");
        let t_11 = link_().easy_parse("   https://");
        assert_eq!(t_00, Ok((String::from("https://"), "")));
        assert_eq!(t_01, Ok((String::from("http://subdomain.domai"), "ニホンゴ")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_inline_spaces1_() {
        let t_00 = inline_spaces1_().easy_parse(" ");
        let t_01 = inline_spaces1_().easy_parse("   \n   ");
        let t_10 = inline_spaces1_().easy_parse("");
        let t_11 = inline_spaces1_().easy_parse("\n");
        assert_eq!(t_00, Ok(((), "")));
        assert_eq!(t_01, Ok(((), "\n   ")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
    }
    #[test]
    fn t_ascii_graphics1_() {
        let t_00 = ascii_graphics_().easy_parse(
            r##"!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~   etc..."##
        );
        let t_10 = ascii_graphics1_().easy_parse("");
        let t_11 = ascii_graphics1_().easy_parse(" ");
        let t_12 = ascii_graphics1_().easy_parse("\n");
        let t_13 = ascii_graphics1_().easy_parse("のんあすきー");
        assert_eq!(t_00, Ok((String::from(
            r##"!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~"##
        ), "   etc...")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());
    }
    #[test]
    fn t_graphics1_() {
        let t_00 = graphics1_().easy_parse("ばぶ👶aZ09!~");
        let t_10 = graphics1_().easy_parse("");
        let t_11 = graphics1_().easy_parse(" ");
        let t_12 = graphics1_().easy_parse("\n");
        let t_13 = graphics1_().easy_parse("　");
        assert_eq!(t_00, Ok((String::from("ばぶ👶aZ09!~"), "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
        assert!(t_13.is_err());
    }
    #[test]
    fn t_graphics1_not_joint_() {
        let t_00 = graphics1_not_joint_().easy_parse("ばぶ👶aZ09!~");
        let t_10 = graphics1_not_joint_().easy_parse("");
        let t_11 = graphics1_not_joint_().easy_parse("[tail");
        let t_12 = graphics1_not_joint_().easy_parse("head]");
        assert_eq!(t_00, Ok((String::from("ばぶ👶aZ09!~"), "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert_eq!(t_12, Ok((String::from("head"), "]")));
    }
    #[test]
    fn t_non_nega_i_() {
        let t_00 = non_nega_i_().easy_parse("000");
        let t_10 = non_nega_i_().easy_parse("");
        let t_11 = non_nega_i_().easy_parse("-1");
        let t_12 = non_nega_i_().easy_parse("   0");
        assert_eq!(t_00, Ok((0i32, "")));
        assert!(t_10.is_err());
        assert!(t_11.is_err());
        assert!(t_12.is_err());
    }
    #[test]
    fn t_non_nega_f_() {
        let t_00 = non_nega_f_().easy_parse("6");
        let t_01 = non_nega_f_().easy_parse("6.0");
        let t_02 = non_nega_f_().easy_parse("6.");
        let t_03 = non_nega_f_().easy_parse(".6");
        let t_04 = non_nega_f_().easy_parse("6..0");
        let t_05 = non_nega_f_().easy_parse("6.0.6");
        let t_06 = non_nega_f_().easy_parse("6.0e-01");
        let t_10 = non_nega_f_().easy_parse(".");
        let t_11 = non_nega_f_().easy_parse("   6");
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
}
