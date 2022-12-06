#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use std::str::FromStr;
use std::vec::Vec;
use std::{error, fmt};

#[derive(Debug, Eq, PartialEq)]
pub struct HostsFile {
    pub lines: Vec<HostsFileLine>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HostsFileLine {
    is_empty: bool,
    comment: Option<String>,
    ip: Option<String>,
    hosts: Option<Vec<String>>,
}

impl fmt::Display for HostsFileLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let out = match self {
        //     HostsFileLine::Empty => "".to_string(),
        //     HostsFileLine::Comment(s) => format!("#{}", s),
        //     HostsFileLine::Host(h) => format!("{} {}", h.ip, h.hosts.join(" ")),
        //     // write!(f, "Error parsing hosts file")
        // };
        let mut parts: Vec<Option<String>> = vec![self.ip.clone()];
        if let Some(hosts) = self.hosts.clone() {
            let mut clone: Vec<Option<String>> =
                hosts.clone().iter_mut().map(|h| Some(h.clone())).collect();
            parts.append(&mut clone);
        }
        parts.push(self.comment.clone());
        let parts: Vec<String> = parts
            .iter()
            .filter(|s| s.is_some())
            .map(|s| s.clone().unwrap())
            .collect();
        write!(f, "{}", parts.join(" "))
    }
}

impl FromStr for HostsFileLine {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<HostsFileLine, Self::Err> {
        HostsFileLine::from_string(s)
    }
}

impl HostsFileLine {
    pub fn from_empty() -> HostsFileLine {
        HostsFileLine {
            is_empty: true,
            comment: None,
            ip: None,
            hosts: None,
        }
    }
    pub fn from_comment(c: &str) -> HostsFileLine {
        HostsFileLine {
            is_empty: false,
            comment: Some(c.to_string()),
            ip: None,
            hosts: None,
        }
    }
    pub fn from_string(line: &str) -> Result<HostsFileLine, ParseError> {
        let line = line.trim();
        if line == "" {
            return Ok(HostsFileLine::from_empty());
        }
        lazy_static! {
            static ref COMMENT_RE: Regex = Regex::new(r"^#.*").unwrap();
        }
        if COMMENT_RE.is_match(line) {
            return Ok(HostsFileLine::from_comment(line));
        }
        let slices: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        let ip: String = slices.first().ok_or(ParseError)?.clone();
        let hosts: Vec<String> = (&slices[1..])
            .iter()
            .take_while(|s| !COMMENT_RE.is_match(s))
            .map(|h| h.to_string())
            .collect();
        if hosts.is_empty() {
            return Err(ParseError);
        }
        let comment: String = (&slices[1..])
            .iter()
            .skip_while(|s| !COMMENT_RE.is_match(s))
            .map(|h| h.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let comment = match comment.as_str() {
            "" => None,
            _ => Some(comment.to_string()),
        };
        Ok(HostsFileLine {
            is_empty: false,
            ip: Some(ip),
            hosts: Some(hosts),
            comment,
        })
    }
    pub fn ip(&self) -> Option<String> {
        self.ip.clone()
    }
    pub fn hosts(&self) -> Vec<String> {
        self.hosts.clone().unwrap_or_else(|| vec![])
    }
    pub fn comment(&self) -> Option<String> {
        self.comment.clone()
    }
    pub fn has_host(&self) -> bool {
        self.ip.is_some()
    }
    pub fn has_comment(&self) -> bool {
        self.comment.is_some()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct HostsFileHost {
    pub ip: String,
    pub hosts: Vec<String>,
    pub comment: Option<String>,
}

pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error parsing hosts file")
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl error::Error for ParseError {}

impl FromStr for HostsFile {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<HostsFile, Self::Err> {
        HostsFile::from_string(s)
    }
}
impl HostsFile {
    fn from_string(s: &str) -> Result<HostsFile, ParseError> {
        let lines: Vec<HostsFileLine> = s
            .lines()
            .map(|l| l.parse::<HostsFileLine>())
            .collect::<Result<Vec<HostsFileLine>, ParseError>>()?;
        Ok(HostsFile { lines })
    }
    pub fn serialize(&self) -> String {
        format!(
            "{}\n",
            self.lines
                .iter()
                .map(|l| format!("{}", l))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Parse tests
    #[test]
    fn from_empty() {
        let parsed = HostsFileLine::from_empty();
        let expected = HostsFileLine {
            is_empty: true,
            ip: None,
            comment: None,
            hosts: None,
        };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn from_comment() {
        let parsed = HostsFileLine::from_comment("#test");
        let expected = HostsFileLine {
            is_empty: false,
            ip: None,
            comment: Some("#test".to_string()),
            hosts: None,
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn empty_line_from_string() {
        let parsed = HostsFileLine::from_string("").unwrap();
        let expected = HostsFileLine::from_empty();
        assert_eq!(parsed, expected);
    }
    #[test]
    fn comment_from_string() {
        let parsed = HostsFileLine::from_string("# comment").unwrap();
        let expected = HostsFileLine::from_comment("# comment");
        assert_eq!(parsed, expected);
    }

    #[test]
    fn broken_from_string() {
        HostsFileLine::from_string("127.0.0.1").expect_err("should fail");
    }
    #[test]
    fn host_from_string() {
        let parsed = HostsFileLine::from_string("127.0.0.1 localhost").unwrap();
        let expected = HostsFileLine {
            is_empty: false,
            ip: Some("127.0.0.1".to_string()),
            hosts: Some(vec!["localhost".to_string()]),
            comment: None,
        };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn full_from_string() {
        let parsed = HostsFileLine::from_string("127.0.0.1 localhost  # a comment").unwrap();
        let expected = HostsFileLine {
            is_empty: false,
            ip: Some("127.0.0.1".to_string()),
            hosts: Some(vec!["localhost".to_string()]),
            comment: Some("# a comment".to_string()),
        };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn empty_input() {
        let parsed = HostsFile::from_str("").unwrap();
        let expected = HostsFile { lines: vec![] };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn a_comment() {
        let parsed = HostsFile::from_str("# comment").unwrap();
        let expected = HostsFile {
            lines: vec![HostsFileLine::from_comment("# comment")],
        };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn two_comments() {
        let parsed = HostsFile::from_str("# comment1\n## comment2\n").unwrap();
        let expected = HostsFile {
            lines: vec![
                HostsFileLine::from_comment("# comment1"),
                HostsFileLine::from_comment("## comment2"),
            ],
        };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn host_with_comments() {
        let parsed = HostsFile::from_str("127.0.0.1 localhost # comment\n").unwrap();
        let expected = HostsFile {
            lines: vec![HostsFileLine {
                is_empty: false,
                ip: Some("127.0.0.1".to_string()),
                hosts: Some(vec!["localhost".to_string()]),
                comment: Some("# comment".to_string()),
            }],
        };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn whitespace() {
        let parsed = HostsFile::from_str(" # comment1\n \n    127.0.0.1    localhost\n").unwrap();
        let expected = HostsFile {
            lines: vec![
                HostsFileLine::from_comment("# comment1"),
                HostsFileLine::from_empty(),
                HostsFileLine::from_string("127.0.0.1 localhost").unwrap(),
            ],
        };
        assert_eq!(parsed, expected);
    }
    #[test]
    fn a_ipv6_host() {
        let parsed = HostsFile::from_str("fe80::1%lo0 localhost\n").unwrap();
        let expected = HostsFile {
            lines: vec![HostsFileLine {
                is_empty: false,
                ip: Some("fe80::1%lo0".to_string()),
                hosts: Some(vec!["localhost".to_string()]),
                comment: None,
            }],
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn a_ipv4_host() {
        let parsed = HostsFile::from_str("127.0.0.1 localhost").unwrap();
        let expected = HostsFile {
            lines: vec![HostsFileLine {
                is_empty: false,
                ip: Some("127.0.0.1".to_string()),
                hosts: Some(vec!["localhost".to_string()]),
                comment: None,
            }],
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn complex_1() {
        let parsed = HostsFile::from_str("# A sample host file\n# empty line\n\n127.0.0.1 localhost\n# multiple hosts\n127.0.0.2 host1 host2\n").unwrap();
        let expected = HostsFile {
            lines: vec![
                HostsFileLine::from_comment("# A sample host file"),
                HostsFileLine::from_comment("# empty line"),
                HostsFileLine::from_empty(),
                HostsFileLine {
                    is_empty: false,
                    ip: Some("127.0.0.1".to_string()),
                    hosts: Some(vec!["localhost".to_string()]),
                    comment: None,
                },
                HostsFileLine::from_comment("# multiple hosts"),
                HostsFileLine {
                    is_empty: false,
                    ip: Some("127.0.0.2".to_string()),
                    hosts: Some(
                        vec!["host1", "host2"]
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    ),
                    comment: None,
                },
            ],
        };
        assert_eq!(parsed, expected);
    }

    // Serialize

    #[test]
    fn serialize_empty() {
        let input = "\n";
        let serialized = HostsFile::from_str(input).unwrap().serialize();
        assert_eq!(serialized, input);
    }
    #[test]
    fn serialize_a_comment() {
        let input = "# a comment\n";
        let serialized = HostsFile::from_str(input).unwrap().serialize();
        assert_eq!(serialized, input);
    }

    #[test]
    fn serialize_complex_1() {
        let input = "# A sample host file\n# empty line\n\n127.0.0.1 localhost\n# multiple hosts\n127.0.0.2 host1 host2\n";
        let serialized = HostsFile::from_str(input).unwrap().serialize();
        assert_eq!(serialized, input);
    }
}
