use postcss_px2rem::transform::{Px2Rem, Px2RemOption, SimplePrettier};
use recursive_parser::{parse, visitor::VisitMut, WrapString};
const BASIC_CSS: &str = ".rule { font-size: 15px; }";
use similar_asserts::assert_str_eq;
use unindent::unindent;

#[cfg(test)]
mod test_pxtorem {
    use super::*;

    #[test]
    fn test_readme_example() {
        let input =
            "h1 { margin: 0 0 20px; font-size: 32px; line-height: 1.2; letter-spacing: 1px; }";
        let actual = get_transformed_content_default(input);
        let expected = unindent(
            r#"
        h1 {
            margin: 0 0 20px;
            font-size: 2rem;
            line-height: 1.2;
            letter-spacing: 0.0625rem;
        }
        "#,
        );
        assert_str_eq!(expected, actual);
    }
    #[test]
    fn test_should_replace_px_with_rem() {
        let expected = unindent(
            r#"
        .rule {
            font-size: 0.9375rem;
        }
        "#,
        );
        assert_str_eq!(expected, get_transformed_content_default(BASIC_CSS));
    }

    #[test]
    fn test_should_ignore_non_px() {
        let expected = unindent(
            r#"
        .rule {
            font-size: 2em;
        }
        "#,
        );
        assert_str_eq!(expected, get_transformed_content_default(&expected));
    }

    #[test]
    fn test_leading_zero() {
        let input = ".rule { margin: 0.5rem .5px -0.2px -.2em }";
        let expected = unindent(
            r#"
        .rule {
            margin: 0.5rem 0.03125rem -0.0125rem -.2em;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec!["margin".to_string()]),
                    ..Px2RemOption::default()
                }
            )
        );
    }

    #[test]
    fn ignore_px_in_custom_properties() {
        let input = ":root { --rem-14px: 14px; } .rule { font-size: var(--rem-14px); }";
        let expected = unindent(
            r#"
        :root {
            --rem-14px: 0.875rem;
        }
        .rule {
            font-size: var(--rem-14px);
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec!["font-size".to_string(), "--*".to_string()]),
                    ..Px2RemOption::default()
                }
            )
        );
    }

    #[test]
    #[ignore]
    fn test_ignore_if_prop_exists() {
        let input = ".rule { font-size: 16px; font-size: 1rem; }";
        let expected = unindent(
            r"
        .rule {
            font-size: 16px;
            font-size: 1rem;
        }
        ",
        );
        assert_str_eq!(expected, get_transformed_content_default(input));
    }

    #[test]
    fn test_remain_unitless_if_0() {
        let input = ".rule { font-size: 0px; font-size: 0; }";
        let expected = unindent(
            r"
        .rule {
            font-size: 0px;
            font-size: 0;
        }
        ",
        );
        assert_str_eq!(expected, get_transformed_content_default(input));
    }
}

#[cfg(test)]
mod test_value_parsing {
    use super::*;

    #[test]
    fn test_value_in_quotes() {
        let input = ".rule { content: '16px'; font-family: \"16px\"; font-size: 16px; }";
        let expected = unindent(
            r#"
        .rule {
            content: '16px';
            font-family: "16px";
            font-size: 1rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec!["*".to_string()]),
                    ..Default::default()
                }
            )
        );
    }

    #[test]
    fn test_not_replace_values_in_url_function() {
        let input = ".rule { background: url(16px.jpg); font-size: 16px; }";
        let expected = unindent(
            r#"
        .rule {
            background: url(16px.jpg);
            font-size: 1rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec!["*".to_string()]),
                    ..Default::default()
                }
            )
        );
    }

    #[test]
    fn test_not_replace_values_with_uppercase_PX() {
        let input = ".rule { margin: 12px calc(100% - 14PX); height: calc(100% - 20px); font-size: 12Px; line-height: 16px; }";
        let expected = unindent(
            r#"
        .rule {
            margin: 0.75rem calc(100% - 14PX);
            height: calc(100% - 1.25rem);
            font-size: 12Px;
            line-height: 1rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec!["*".to_string()]),
                    ..Default::default()
                }
            )
        );
    }
}

#[cfg(test)]
mod test_root_value {
    use super::*;

    #[test]
    fn test_root_value_of_10() {
        let expected = unindent(
            r#"
        .rule {
            font-size: 1.5rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                BASIC_CSS,
                Px2RemOption {
                    root_value: Some(10),
                    ..Default::default()
                }
            )
        );
    }

    // TODO: maybe this should handled by loader or cli?
    // it("should replace using different root values with different files", function() {
    //     var css2 = ".rule { font-size: 20px }";
    //     var expected = ".rule { font-size: 1rem }";
    //     var options = {
    //       rootValue: function(input) {
    //         if (input.from.indexOf("basic.css") !== -1) {
    //           return 15;
    //         }
    //         return 20;
    //       }
    //     };
    //     var processed1 = postcss(pxtorem(options)).process(basicCSS, {
    //       from: "/tmp/basic.css"
    //     }).css;
    //     var processed2 = postcss(pxtorem(options)).process(css2, {
    //       from: "/tmp/whatever.css"
    //     }).css;

    //     expect(processed1).toBe(expected);
    //     expect(processed2).toBe(expected);
    //   });
}
#[cfg(test)]
mod test_unit_precision {
    use super::*;

    #[test]
    fn test_precision_two() {
        let expected = unindent(
            r#"
        .rule {
            font-size: 0.94rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                BASIC_CSS,
                Px2RemOption {
                    unit_precision: Some(2),
                    ..Default::default()
                }
            )
        );
    }
}

#[cfg(test)]
mod test_prop_list {
    use super::*;

    #[test]
    fn test_only_replace_prop_in_whitelist() {
        let expected = unindent(
            r#"
        .rule {
            font-size: 15px;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                BASIC_CSS,
                Px2RemOption {
                    prop_list: Some(vec!["font".to_string()]),
                    ..Default::default()
                }
            )
        );
    }
    #[test]
    fn test_only_replace_prop_in_whitelist_two() {
        let input = ".rule { margin: 16px; margin-left: 10px }";
        let expected = unindent(
            r#"
        .rule {
            margin: 1rem;
            margin-left: 10px;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec!["margin".to_string()]),
                    ..Default::default()
                }
            )
        );
    }
    #[test]
    fn test_only_replace_prop_in_whitelist_three() {
        let input = ".rule { font-size: 16px; margin: 16px; margin-left: 5px; padding: 5px; padding-right: 16px }";
        let expected = unindent(
            r#"
        .rule {
            font-size: 1rem;
            margin: 1rem;
            margin-left: 5px;
            padding: 5px;
            padding-right: 1rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec![
                        "*font*".to_string(),
                        "margin*".to_string(),
                        "!margin-left".to_string(),
                        "*-right".to_string(),
                        "pad".to_string()
                    ]),
                    ..Default::default()
                }
            )
        );
    }

    #[test]
    fn test_only_replace_prop_in_whitelist_with_wildcard() {
        let input = ".rule { font-size: 16px; margin: 16px; margin-left: 5px; padding: 5px; padding-right: 16px }";
        let expected = unindent(
            r#"
        .rule {
            font-size: 16px;
            margin: 1rem;
            margin-left: 5px;
            padding: 5px;
            padding-right: 16px;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    prop_list: Some(vec![
                        "*".to_string(),
                        "!*padding*".to_string(),
                        "!margin-left".to_string(),
                        "!font*".to_string()
                    ]),
                    ..Default::default()
                }
            )
        );
    }
    // ignore this case, since we don't have legacy option
    //     it("should replace all properties when white list is empty", function() {
    //     var rules = ".rule { margin: 16px; font-size: 15px }";
    //     var expected = ".rule { margin: 1rem; font-size: 0.9375rem }";
    //     var options = {
    //       propWhiteList: []
    //     };
    //     var processed = postcss(pxtorem(options)).process(rules).css;

    //     expect(processed).toBe(expected);
    //   });
}

#[cfg(test)]
mod test_selector_black_list {
    use super::*;

    #[test]
    fn test_ignore_rule_when_selector_in_black_list() {
        let input = ".rule { font-size: 15px } .rule2 { font-size: 15px }";
        let expected = unindent(
            r#"
        .rule {
            font-size: 0.9375rem;
        }
        .rule2 {
            font-size: 15px;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    selector_black_list: Some(vec![
                        postcss_px2rem::transform::StringOrRegexp::String(".rule2".to_string())
                    ]),
                    ..Default::default()
                }
            )
        );
    }

    #[test]
    fn test_ignore_rule_every_selector_with_body_dollar() {
        let input = "body { font-size: 16px; } .class-body$ { font-size: 16px; } .simple-class { font-size: 16px; }";
        let expected = unindent(
            r#"
        body {
            font-size: 1rem;
        }
        .class-body$ {
            font-size: 16px;
        }
        .simple-class {
            font-size: 1rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    selector_black_list: Some(vec![
                        postcss_px2rem::transform::StringOrRegexp::String("body$".to_string())
                    ]),
                    ..Default::default()
                }
            )
        );
    }

    #[test]
    fn test_ignore_rule_regex_body() {
        let input = "body { font-size: 16px; } .class-body { font-size: 16px; } .simple-class { font-size: 16px; }";
        let expected = unindent(
            r#"
        body {
            font-size: 16px;
        }
        .class-body {
            font-size: 1rem;
        }
        .simple-class {
            font-size: 1rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    selector_black_list: Some(vec![
                        postcss_px2rem::transform::StringOrRegexp::Regexp("^body$".to_string())
                    ]),
                    ..Default::default()
                }
            )
        );
    }
}
// We don't consider replace now, just think such `.test {width: 16px; width: 1rem;}` is meaningless
// will execute as follow `.test {width: 1rem}`

// describe("replace", function() {
//     it("should leave fallback pixel unit with root em value", function() {
//       var options = {
//         replace: false
//       };
//       var processed = postcss(pxtorem(options)).process(basicCSS).css;
//       var expected = ".rule { font-size: 15px; font-size: 0.9375rem }";

//       expect(processed).toBe(expected);
//     });
//   });

#[cfg(test)]
mod test_media_query {
    use super::*;

    #[test]
    fn test_replace_px_in_media_query() {
        let input = "@media (min-width: 500px) { .rule { font-size: 16px } }";
        let expected = unindent(
            r#"
        @media (min-width: 31.25rem) {
            .rule {
                font-size: 1rem;
            }
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    media_query: Some(true),
                    ..Default::default()
                }
            )
        );
    }
}

#[cfg(test)]
mod test_min_pixel_value {
    use super::*;

    #[test]
    fn test_not_replace_value_below_min_pixel_value() {
        let input = ".rule { border: 1px solid #000; font-size: 16px; margin: 1px 10px; }";
        let expected = unindent(
            r#"
        .rule {
            border: 1px solid #000;
            font-size: 1rem;
            margin: 1px 0.625rem;
        }
        "#,
        );
        assert_str_eq!(
            expected,
            get_transformed_content_new(
                input,
                Px2RemOption {
                    min_pixel_value: Some(2f64),
                    prop_list: Some(vec!["*".to_string()]),
                    ..Default::default()
                }
            )
        );
    }
}

#[cfg(test)]
mod test_filter_prop_list {
    use super::*;

    #[test]
    fn test_exact() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "margin".to_string(),
                "!padding".to_string(),
                "*border*".to_string(),
                "*".to_string(),
                "*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(
            px_to_rem.match_list.exact_list.join(","),
            "font-size,margin"
        );
    }
    #[test]
    fn test_contain() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "*margin*".to_string(),
                "!padding".to_string(),
                "*border*".to_string(),
                "*".to_string(),
                "*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(px_to_rem.match_list.contain_list.join(","), "margin,border");
    }

    #[test]
    fn test_start() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "*margin*".to_string(),
                "!padding".to_string(),
                "border*".to_string(),
                "*".to_string(),
                "*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(px_to_rem.match_list.starts_with_list.join(","), "border");
    }

    #[test]
    fn test_end() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "*margin*".to_string(),
                "!padding".to_string(),
                "border*".to_string(),
                "*".to_string(),
                "*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(px_to_rem.match_list.ends_with_list.join(","), "y");
    }

    #[test]
    fn test_not_exact() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "*margin*".to_string(),
                "!padding".to_string(),
                "border*".to_string(),
                "*".to_string(),
                "*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(px_to_rem.match_list.not_exact_list.join(","), "padding");
    }

    #[test]
    fn test_not_start() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "*margin*".to_string(),
                "!padding".to_string(),
                "border*".to_string(),
                "*".to_string(),
                "*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(px_to_rem.match_list.not_contain_list.join(","), "font");
    }
    #[test]
    fn test_not_contain() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "*margin*".to_string(),
                "!padding".to_string(),
                "border*".to_string(),
                "*".to_string(),
                "*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(px_to_rem.match_list.not_contain_list.join(","), "font");
    }

    #[test]
    fn test_not_end() {
        let mut px_to_rem = Px2Rem::new(Px2RemOption {
            prop_list: Some(vec![
                "font-size".to_string(),
                "*margin*".to_string(),
                "!padding".to_string(),
                "!border*".to_string(),
                "*".to_string(),
                "!*y".to_string(),
                "!*font*".to_string(),
            ]),
            ..Default::default()
        });
        px_to_rem.generate_match_list();
        assert_str_eq!(px_to_rem.match_list.not_ends_list.join(","), "y");
    }
}
// these test case should handled by cli or node binding
// describe("exclude", function() {
//   it("should ignore file path with exclude RegEx", function() {
//     var options = {
//       exclude: /exclude/i
//     };
//     var processed = postcss(pxtorem(options)).process(basicCSS, {
//       from: "exclude/path"
//     }).css;
//     expect(processed).toBe(basicCSS);
//   });

//   it("should not ignore file path with exclude String", function() {
//     var options = {
//       exclude: "exclude"
//     };
//     var processed = postcss(pxtorem(options)).process(basicCSS, {
//       from: "exclude/path"
//     }).css;
//     expect(processed).toBe(basicCSS);
//   });

//   it("should not ignore file path with exclude function", function() {
//     var options = {
//       exclude: function(file) {
//         return file.indexOf("exclude") !== -1;
//       }
//     };
//     var processed = postcss(pxtorem(options)).process(basicCSS, {
//       from: "exclude/path"
//     }).css;
//     expect(processed).toBe(basicCSS);
//   });
// });
fn get_transformed_content_default(input: &str) -> String {
    let mut root = parse(input, None);

    let mut px_to_rem = Px2Rem::default();
    px_to_rem.generate_match_list();
    px_to_rem.visit_root(&mut root);
    let wrap_string = WrapString::default();
    let mut writer = SimplePrettier::new(wrap_string, 4);
    writer.visit_root(&mut root).unwrap();
    writer.writer.0
}

fn get_transformed_content_new(input: &str, option: Px2RemOption) -> String {
    let mut root = parse(input, None);

    let mut px_to_rem = Px2Rem::new(option);
    px_to_rem.visit_root(&mut root);
    let wrap_string = WrapString::default();
    let mut writer = SimplePrettier::new(wrap_string, 4);
    writer.visit_root(&mut root).unwrap();
    writer.writer.0
}
