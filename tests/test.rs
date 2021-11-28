use postcss_px2rem::transform::{Px2Rem, SimplePrettier};
use recursive_parser::{parse, visitor::VisitMut, WrapString};

#[cfg(test)]
mod test_pxtorem {
    use super::*;

    #[test]
    fn test_readme_example() {
        let input =
            "h1 { margin: 0 0 20px; font-size: 32px; line-height: 1.2; letter-spacing: 1px; }";
        let actual = get_transformed_content(input);
        // assert_eq!(, );
    }
}

fn get_transformed_content(input: &str) -> String {
    let mut root = parse(input, None);

    let mut px_to_rem = Px2Rem::default();
    px_to_rem.visit_root(&mut root);
    let wrap_string = WrapString::default();
    let mut writer = SimplePrettier::new(wrap_string, 2);
    writer.visit_root(&mut root).unwrap();
    writer.writer.0
}
