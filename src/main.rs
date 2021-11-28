use std::time::Instant;
use postcss_px2rem::transform::{Px2Rem, SimplePrettier};
use recursive_parser::{WrapString, parse, visitor::VisitMut};
fn main() {
    let css = r#"
    h1 { margin: 0 0 20px; font-size: 32px; line-height: 1.2; letter-spacing: 1px; }
    "#;
    let css2 = include_str!("../assets/bootstrap.css");
    let start = Instant::now();
    let mut root = parse(css, None);
    println!("{:?}", start.elapsed());

    let start = Instant::now();
    let mut px_to_rem = Px2Rem::default();
    px_to_rem.visit_root(&mut root);
    println!("{:?}", start.elapsed());
    let mut writer = SimplePrettier::new(std::io::stdout(), 2);
    writer.visit_root(&mut root).unwrap();
}
