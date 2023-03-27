
#[cfg(test)]
use super::*;

/*----------------------------------------------------------------------
Start with json
convert to native rust object
convert back to json
compare to original json

FIXME:
    This is rather fragile regarding spacing. It would be better to do
    one more conversion back to objects and compare objects. Instead of
    json -> object -> json2 and check json==json2 there is json ->
    object -> json -> object2 and check object==object2.
*/

#[test]
fn test_serde() {

    // two example lsys in json form
    let json = indoc! {r#"
        {
          "title": "Hilbert Curve",
          "refs": [
            "https://www.cs.unh.edu/~charpov/programming-lsystems.html"
          ],
          "start": "X",
          "angle": 90.0,
          "order": [],
          "rules": {
            "X": "-YF+XFX+FY-",
            "Y": "+XF-YFY-FX+"
          },
          "post_rules": {}
        }

        {
          "title": "Koch's Snowflake",
          "refs": [
            "https://www.cs.unh.edu/~charpov/programming-lsystems.html"
          ],
          "start": "+F--F--F",
          "angle": 60.0,
          "order": [],
          "rules": {
            "F": "F+F--F+F"
          },
          "post_rules": {}
        }

    "#};

    // covert json to vector of LSys structs
    let chunks = json_to_chunks(json);
    //println!("{:#?}",chunks);
    let lsysv = lsys_from_json_chunks(&chunks);

    // convert back to json
    let mut json2 = String::new();
    for lsys in lsysv {
        json2.push_str(&serde_json::to_string_pretty(&lsys).unwrap());
        json2.push_str(&"\n\n");
    }

    //println!("{}",json);
    //println!("{}",json2);

    // compare
    assert_eq!(json,json2);
}

/*----------------------------------------------------------------------
*/

use crate::rules_apply_basic;
use crate::Rules;
use std::collections::HashMap;

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
