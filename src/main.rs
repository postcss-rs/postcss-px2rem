use postcss_px2rem::transform::{Px2Rem, Px2RemOption, SimplePrettier};
use recursive_parser::{parse, visitor::VisitMut, WrapString};
use std::time::Instant;
fn main() {
    let css = ".rule { margin: 12px calc(100% - 14PX); height: calc(100% - 20px); font-size: 12Px; line-height: 16px; }";
    let css2 = include_str!("../assets/bootstrap.css");
    let start = Instant::now();
    let mut root = parse(css2, None);
    println!("{:?}", start.elapsed());

    let start = Instant::now();
    let mut px_to_rem = Px2Rem::new(Px2RemOption {
        prop_list: Some(vec!["*".to_string()]),
        ..Px2RemOption::default()
    });
    px_to_rem.visit_root(&mut root);
    println!("{:?}", start.elapsed());
    let mut writer = SimplePrettier::new(std::io::stdout(), 2);
    // writer.visit_root(&mut root).unwrap();
    let input = ".rule { font-size: 15px } .rule2 { font-size: 15px }";
    let expected = r#"
    .rule {
        font-size: 0.9375rem;
    }
    .rule2 {
        font-size: 15px;
    }
    "#;
    assert_eq!(
        expected,
        get_transformed_content_new(
            input,
            Px2RemOption {
                selector_black_list: Some(vec![postcss_px2rem::transform::StringOrRegexp::String(
                    ".rule2".to_string()
                )]),
                ..Default::default()
            }
        )
    );
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
