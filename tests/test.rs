use postcss_px2rem::transform::{Px2Rem, Px2RemOption, SimplePrettier};
use recursive_parser::{parse, visitor::VisitMut, WrapString};
const BASIC_CSS: &str = ".rule { font-size: 15px; }";

#[cfg(test)]
mod test_pxtorem {
    use similar_asserts::assert_str_eq;
    use unindent::unindent;

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
    #[ignore]
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



    //      it("should handle < 1 values and values without a leading 0 - legacy", function() {
    //     var rules = ".rule { margin: 0.5rem .5px -0.2px -.2em }";
    //     var expected = ".rule { margin: 0.5rem 0.03125rem -0.0125rem -.2em }";
    //     var options = {
    //       propWhiteList: ["margin"]
    //     };
    //     var processed = postcss(pxtorem(options)).process(rules).css;

    //     expect(processed).toBe(expected);
    //   });
}

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
