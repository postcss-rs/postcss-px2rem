use postcss_px2rem::transform::{Px2Rem, Px2RemOption, SimplePrettier};
use recursive_parser::{parse, visitor::VisitMut, WrapString};
use std::time::Instant;
fn main() {
    let css = r#"
   .rule { background: url(16px.jpg); font-size: 16px; }
   .rule { margin: 0.5rem .5px -0.2px -.2em }
    "#;
    let css2 = include_str!("../assets/bootstrap.css");
    let start = Instant::now();
    let mut root = parse(css, None);
    println!("{:?}", start.elapsed());

    let start = Instant::now();
    let mut px_to_rem = Px2Rem::new(Px2RemOption {
        prop_list: Some(vec!["margin".to_string()]),
        ..Px2RemOption::default()
    });
    px_to_rem.visit_root(&mut root);
    println!("{:?}", start.elapsed());
    let mut writer = SimplePrettier::new(std::io::stdout(), 2);
    writer.visit_root(&mut root).unwrap();
}
