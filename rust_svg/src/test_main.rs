/*----------------------------------------------------------------------
*/

use crate::LSys;
use std::collections::HashMap;

pub fn test_serde() {

    let mut c = LSys::default();
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

    let c1:LSys = serde_json::from_str(&j1).unwrap();
    println!("{:#?}",&c1);
}

/*----------------------------------------------------------------------
*/

use crate::rules_apply_basic;
use crate::Rules;

pub fn test_rules_apply_basic() {
    let rules:Rules = HashMap::from([
        ('A',"AB"),
        ('B',"A")
    ]);
    let start:&str = "A";

    assert!(rules_apply_basic(&rules,start,0) == "A");
    assert!(rules_apply_basic(&rules,start,1) == "AB");
    assert!(rules_apply_basic(&rules,start,2) == "ABA");
    assert!(rules_apply_basic(&rules,start,3) == "ABAAB");
    assert!(rules_apply_basic(&rules,start,4) == "ABAABABA");
    println!("tested rules_apply_basic");
}

/*----------------------------------------------------------------------
*/
use crate::layout_boxes_draw;
use crate::layout_boxes_make;
use std::fs;

pub fn test_layout_boxes() {
    let lb = layout_boxes_make();
    let svg = layout_boxes_draw(&lb);
    print!("bounding boxes{:#?}",&lb);
    _ = fs::write("layout_boxes.svg", svg);
}
