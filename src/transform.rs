use std::{
    borrow::{Borrow, Cow},
    fmt::Debug,
    io::Write,
    rc::Rc,
};

use crate::{
    filter_prop_list::{
        contain, ends_with, exact, not_contain, not_ends_with, not_exact, not_starts_with,
        starts_with,
    },
    regex,
};
use recursive_parser::{
    parser::{AtRule, Declaration, Root, Rule, RuleOrAtRuleOrDecl},
    visitor::VisitMut,
};
use regex::{Captures, Regex};
use smol_str::SmolStr;
#[derive(Debug)]
pub enum StringOrRegexp {
    Regexp(String),
    String(String),
}
#[derive(Default)]
pub struct Px2RemOption {
    pub root_value: Option<i32>,
    pub unit_precision: Option<i32>,
    pub selector_black_list: Option<Vec<StringOrRegexp>>,
    pub prop_list: Option<Vec<String>>,
    pub replace: Option<bool>,
    pub media_query: Option<bool>,
    pub min_pixel_value: Option<f64>,
}
#[derive(Debug)]
pub struct Px2Rem {
    px_regex: &'static Regex,
    root_value: i32,
    unit_precision: i32,
    selector_black_list: Vec<StringOrRegexp>,
    prop_list: Rc<Vec<String>>,
    replace: bool,
    media_query: bool,
    min_pixel_value: f64,
    has_wild: bool, //   exclude: null we don't need the prop, since this is always used for cli
    pub match_list: MatchList,
    // exact_list: Vec<&'a String>,
    all_match: bool,
}

impl Default for Px2Rem {
    /// default constructor will not automatically generate match list for you,         
    /// because default function used in new constructor, if we call generate match list   
    /// in default function, we also need to call generate match list in new constructor,  
    /// which could introduce a bit overhead, so you if you use default constructor, you need to   
    /// call generate match list function yourself.
    fn default() -> Px2Rem {
        // let prop_list = ;
        let ret = Self {
            px_regex: regex!(r#""[^"]+"|'[^']+'|url\([^)]+\)|var\([^)]+\)|(\d*\.?\d+)px"#),
            root_value: 16,
            unit_precision: 5,
            selector_black_list: vec![],
            prop_list: Rc::new(vec![
                "font".to_string(),
                "font-size".to_string(),
                "line-height".to_string(),
                "letter-spacing".to_string(),
            ]),
            replace: true,
            media_query: false,
            min_pixel_value: 0f64,
            has_wild: false,
            match_list: MatchList::default(),
            all_match: false,
        };
        // ret.generate_match_list();
        ret
    }
}

impl Px2Rem {
    /// new constructor will automatically generate match list for you
    pub fn new(option: Px2RemOption) -> Self {
        let mut ret = Self::default();
        if let Some(root_value) = option.root_value {
            ret.root_value = root_value;
        }
        if let Some(unit_precision) = option.unit_precision {
            ret.unit_precision = unit_precision;
        }
        if let Some(selector_black_list) = option.selector_black_list {
            ret.selector_black_list = selector_black_list;
        }
        if let Some(prop_list) = option.prop_list {
            ret.prop_list = Rc::new(prop_list);
        }
        if let Some(replace) = option.replace {
            ret.replace = replace;
        }
        if let Some(media_query) = option.media_query {
            ret.media_query = media_query;
        }
        if let Some(min_pixel_value) = option.min_pixel_value {
            ret.min_pixel_value = min_pixel_value;
        }
        ret.generate_match_list();
        ret
    }
    pub fn generate_match_list(&mut self) {
        // let prop_list = self.prop_list;
        // self.exact_list = exact(prop_list);
        self.match_list = MatchList {
            exact_list: exact(self.prop_list.clone()),
            contain_list: contain(self.prop_list.clone()),
            starts_with_list: starts_with(self.prop_list.clone()),
            ends_with_list: ends_with(self.prop_list.clone()),
            not_exact_list: not_exact(self.prop_list.clone()),
            not_contain_list: not_contain(self.prop_list.clone()),
            not_starts_list: not_starts_with(self.prop_list.clone()),
            not_ends_list: not_ends_with(self.prop_list.clone()),
        };
        let has_wild = self.prop_list.iter().any(|prop| prop == "*");
        let match_all = has_wild && self.prop_list.len() == 1;
        self.has_wild = has_wild;
        self.all_match = match_all;
    }
    pub fn px_replace<'a>(&self, value: &'a str) -> Cow<'a, str> {
        self.px_regex.replace_all(value, |caps: &Captures| {
            let pixels_value = &caps.get(1);
            if let Some(pixels_value) = pixels_value {
                match pixels_value.as_str().parse::<f64>() {
                    Ok(pixels) => {
                        if pixels < self.min_pixel_value {
                            return caps[0].to_string();
                        }
                        let fixed_value = pixels / self.root_value as f64;
                        if fixed_value == 0f64 {
                            caps[0].to_string()
                        } else {
                            let mut res =
                                format!("{:.*}", self.unit_precision as usize, fixed_value);
                            let cont = res.ends_with("0");
                            if cont {
                                let mut temp = res.trim_end_matches("0");
                                if temp.ends_with(".") {
                                    temp = &temp[0..temp.len() - 1];
                                }
                                res = temp.to_string();
                            }
                            res.to_string() + "rem"
                        }
                    }
                    Err(_) => caps[0].to_string(),
                }
            } else {
                caps[0].to_string()
            }
        })
    }

    pub fn blacklisted_selector(&self, selector: &str) -> bool {
        if self.selector_black_list.len() == 0 {
            return false;
        }
        let BLACK_LIST_RE: once_cell::sync::OnceCell<regex::Regex> =
            once_cell::sync::OnceCell::new();
        // TODO: this implementation is inefficient
        let re = BLACK_LIST_RE.get_or_init(|| {
            regex::Regex::new(
                &self
                    .selector_black_list
                    .iter()
                    .filter(|re| match re {
                        StringOrRegexp::Regexp(_) => true,
                        StringOrRegexp::String(_) => false,
                    })
                    .map(|reg| match reg {
                        StringOrRegexp::Regexp(str) => str.to_string(),
                        StringOrRegexp::String(_) => unreachable!(),
                    })
                    .collect::<Vec<_>>()
                    .join("|"),
            )
            .unwrap()
        });
        (if re.as_str().len() == 0 {
            false
        } else {
            re.is_match(selector)
        }) || {
            self.selector_black_list
                .iter()
                .any(|pattern| match pattern {
                    StringOrRegexp::Regexp(_) => false,
                    StringOrRegexp::String(string) => selector.contains(string),
                })
        }
    }

    fn is_match(&self, prop: &str) -> bool {
        // TODO: this implementation maybe not efficient, need to explore a better way
        if self.all_match {
            return true;
        };
        return (self.has_wild
            || self
                .match_list
                .exact_list
                .iter()
                .any(|p| p.as_str() == prop)
            || self
                .match_list
                .contain_list
                .iter()
                .any(|p| prop.contains(p.as_str()))
            || self
                .match_list
                .starts_with_list
                .iter()
                .any(|p| prop.starts_with(p.as_str()))
            || self
                .match_list
                .ends_with_list
                .iter()
                .any(|p| prop.ends_with(p.as_str())))
            && !(self.match_list.not_exact_list.iter().any(|p| p == prop)
                || self
                    .match_list
                    .not_contain_list
                    .iter()
                    .any(|p| prop.contains(p.as_str()))
                || self
                    .match_list
                    .not_starts_list
                    .iter()
                    .any(|p| prop.starts_with(p.as_str()))
                || self
                    .match_list
                    .not_ends_list
                    .iter()
                    .any(|p| prop.ends_with(p.as_str())));
    }
}
#[derive(Default, Debug)]
pub struct MatchList {
    pub exact_list: Vec<SmolStr>,
    pub contain_list: Vec<SmolStr>,
    pub starts_with_list: Vec<SmolStr>,
    pub ends_with_list: Vec<SmolStr>,
    pub not_exact_list: Vec<SmolStr>,
    pub not_contain_list: Vec<SmolStr>,
    pub not_starts_list: Vec<SmolStr>,
    pub not_ends_list: Vec<SmolStr>,
}

impl<'a> VisitMut<'a> for Px2Rem {
    fn visit_root(&mut self, root: &mut recursive_parser::parser::Root<'a>) -> () {
        for child in root.children.iter_mut() {
            match child {
                recursive_parser::parser::RuleOrAtRuleOrDecl::Rule(rule) => {
                    if !self.blacklisted_selector(&rule.selector.content) {
                        self.visit_rule(rule);
                    }
                }
                recursive_parser::parser::RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule);
                }
                recursive_parser::parser::RuleOrAtRuleOrDecl::Declaration(_) => unreachable!(),
            }
        }
    }

    fn visit_rule(&mut self, rule: &mut recursive_parser::parser::Rule<'a>) -> () {
        for child in rule.children.iter_mut() {
            match child {
                recursive_parser::parser::RuleOrAtRuleOrDecl::Rule(rule) => {
                    unimplemented!()
                }
                recursive_parser::parser::RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule);
                }
                recursive_parser::parser::RuleOrAtRuleOrDecl::Declaration(decl) => {
                    self.visit_declaration(decl);
                }
            }
        }
    }

    fn visit_at_rule(&mut self, at_rule: &mut recursive_parser::parser::AtRule<'a>) -> () {
        if self.media_query && at_rule.name == "media" && at_rule.params.contains("px") {
            let value = self.px_replace(&at_rule.params).to_string();
            at_rule.params = Cow::Owned(value);
        }
        for child in at_rule.children.iter_mut() {
            match child {
                recursive_parser::parser::RuleOrAtRuleOrDecl::Rule(rule) => {
                    self.visit_rule(rule);
                }
                recursive_parser::parser::RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule);
                }
                recursive_parser::parser::RuleOrAtRuleOrDecl::Declaration(decl) => {
                    self.visit_declaration(decl);
                }
            }
        }
    }

    fn visit_declaration(&mut self, decl: &mut recursive_parser::parser::Declaration<'a>) -> () {
        if !decl.value.content.contains("px") {
            return;
        }
        if !self.is_match(&decl.prop.content) {
            return;
        }
        let value = self.px_replace(&decl.value.content).to_string();
        // TODO: decide replace or insert after
        decl.value.content = Cow::Owned(value);
    }
}

#[derive(Default)]
pub struct SimplePrettier<W: Write> {
    level: usize,
    pub writer: W,
    indent: usize,
}

impl<W: Write> SimplePrettier<W> {
    pub fn new(writer: W, indent: usize) -> Self {
        Self {
            level: 0,
            writer,
            indent,
        }
    }
}
impl<'a, W: std::io::Write> VisitMut<'a, std::io::Result<()>> for SimplePrettier<W> {
    fn visit_root(&mut self, root: &mut Root<'a>) -> std::io::Result<()> {
        for child in root.children.iter_mut() {
            match child {
                RuleOrAtRuleOrDecl::Rule(rule) => {
                    self.visit_rule(rule)?;
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule)?;
                }
                RuleOrAtRuleOrDecl::Declaration(_) => {
                    unreachable!()
                }
            }
        }
        Ok(())
    }

    fn visit_rule(&mut self, rule: &mut Rule<'a>) -> std::io::Result<()> {
        self.writer.write(
            format!(
                "{}{} {}\n",
                " ".repeat(self.level * self.indent),
                rule.selector.content,
                "{"
            )
            .as_bytes(),
        )?;
        self.level += 1;
        for child in rule.children.iter_mut() {
            match child {
                RuleOrAtRuleOrDecl::Rule(_) => {
                    unreachable!()
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule)?;
                }
                RuleOrAtRuleOrDecl::Declaration(decl) => {
                    self.visit_declaration(decl)?;
                }
            }
        }
        self.level -= 1;
        write!(
            self.writer,
            "{}{}\n",
            " ".repeat(self.level * self.indent),
            "}"
        )?;
        Ok(())
    }

    fn visit_at_rule(&mut self, at_rule: &mut AtRule<'a>) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}@{} {} {}\n",
            " ".repeat(self.level * self.indent),
            at_rule.name,
            at_rule.params,
            "{"
        )?;
        self.level += 1;
        for child in at_rule.children.iter_mut() {
            match child {
                RuleOrAtRuleOrDecl::Rule(rule) => {
                    self.visit_rule(rule)?;
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule)?;
                }
                RuleOrAtRuleOrDecl::Declaration(_decl) => {
                    //   self.visit_declaration(decl);
                }
            }
        }
        self.level -= 1;
        write!(
            self.writer,
            "{}{}\n",
            " ".repeat(self.level * self.indent),
            "}"
        )
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}{}: {};\n",
            " ".repeat(self.level * self.indent),
            decl.prop,
            decl.value
        )
    }
}
