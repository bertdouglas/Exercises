/*----------------------------------------------------------------------
*/

use crate::Curve;
use std::collections::HashMap;

pub fn test_serde() {

    let mut c = Curve::default();
    c.title = String::from("Hilbert Curve");
    c.refs.push(String::from(
        "https://www.cs.unh.edu/~charpov/programming-lsystems.html"
    ));
    c.start = String::from("X");
    c.angle = 90.0;
    c.rules = HashMap::from([
        ('X', "-YF+XFX+FY-"),
        ('Y', "+XF-YFY-FX+"),
    ]);

  let j = serde_json::to_string_pretty(&c).unwrap();
  println!("{}",j);

  let j1 = String::from(r#"
{
  "title": "Hilbert Curve",
  "refs": [
    "https://www.cs.unh.edu/~charpov/programming-lsystems.html"
  ],
  "start": "X",
  "angle": 90.0,
  "rules": {
    "X": "-YF+XFX+FY-",
    "Y": "+XF-YFY-FX+"
  }
}
"#);

    let c1:Curve = serde_json::from_str(&j1).unwrap();
    println!("{:#?}",&c1);
}

/*----------------------------------------------------------------------
*/

use crate::apply_rules_basic;
use crate::Rules;

pub fn test_apply_rules_basic() {
    let rules:Rules = HashMap::from([
        ('A',"AB"),
        ('B',"A")
    ]);
    let start:&str = "A";

    assert!(apply_rules_basic(&rules,start,0) == "A");
    assert!(apply_rules_basic(&rules,start,1) == "AB");
    assert!(apply_rules_basic(&rules,start,2) == "ABA");
    assert!(apply_rules_basic(&rules,start,3) == "ABAAB");
    assert!(apply_rules_basic(&rules,start,4) == "ABAABABA");
    println!("tested apply_rules_basic");
}

/*----------------------------------------------------------------------
*/
use crate::draw_layout_boxes;
use crate::make_layout_boxes;
use std::fs;

pub fn test_layout_boxes() {
    let lb = make_layout_boxes();
    let svg = draw_layout_boxes(&lb);
    print!("bounding boxes{:#?}",&lb);
    _ = fs::write("layout_boxes.svg", svg);
}
