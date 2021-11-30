use std::{borrow::Cow, rc::Rc};
use smol_str::SmolStr;
use crate::regex;

pub fn exact<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!("^[^*!]+$");
            reg.is_match(prop)
        })
        .map(|a| a.into())
        .collect::<Vec<_>>()
}

pub fn contain<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!(r#"^\*.+\*$"#);
            reg.is_match(prop)
        })
        .map(|prop| (&prop[1..prop.len() - 1]).into())
        .collect::<Vec<_>>()
}

pub fn ends_with<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!(r"^\*[^*]+$");
            reg.is_match(prop)
        })
        .map(|prop| (&prop[1..]).into())
        .collect::<Vec<_>>()
}

pub fn starts_with<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!(r"^[^*!]+\*$");
            reg.is_match(prop)
        })
        .map(|prop| (&prop[0..prop.len() - 1]).into())
        .collect::<Vec<_>>()
}

pub fn not_exact<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!("^![^*].*$");
            reg.is_match(prop)
        })
        .map(|prop| (&prop[1..]).into())
        .collect::<Vec<_>>()
}

pub fn not_contain<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!(r#"^!\*.+\*$"#);
            reg.is_match(prop)
        })
        .map(|prop| (&prop[2..prop.len() - 1]).into())
        .collect::<Vec<_>>()
}

pub fn not_ends_with<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!(r"^!\*[^*]+$");
            reg.is_match(prop)
        })
        .map(|prop| (&prop[2..]).into())
        .collect::<Vec<_>>()
}

pub fn not_starts_with<'a>(list: Rc<Vec<String>>) -> Vec<SmolStr> {
    list.iter()
        .filter(|prop| {
            let reg = regex!(r"^![^*]+\*");
            reg.is_match(prop)
        })
        .map(|prop| (&prop[1..prop.len() - 1]).into())
        .collect::<Vec<_>>()
}

// pub fn exact_two<'a>(list: &'a Vec<String>) -> Vec<String> {
//     list.iter()
//         .filter(|prop| {
//             let reg = regex!("^[^*!]+$");
//             reg.is_match(prop)
//         })
//         .collect::<Vec<_>>()
// }
