
#[cfg(test)]
use super::*;

/*----------------------------------------------------------------------
Test Serialization/Deserialization of LSys examples

Start with json1
convert to native rust struct1
convert to json2
compare to native rust struct2
compare struct1 with struct2

This scheme avoids compare failure because of unstable ordering of
items in a hashmap from one run to another. It also avoids compare
failure from extra or missing spaces in json.
*/

#[test]
fn test_serde() {

    // two example lsys in json form
    let json1 = indoc! {r#"
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
    let chunks = json_to_chunks(json1);
    //println!("{:#?}",chunks);
    let lsysv1 = lsys_from_json_chunks(&chunks);

    // convert back to json
    let mut json2 = String::new();
    for lsys in &lsysv1 {
        json2.push_str(&serde_json::to_string_pretty(&lsys).unwrap());
        json2.push_str(&"\n\n");
    }

    // convert to structs again
    let chunks = json_to_chunks(&json2);
    let lsysv2 = lsys_from_json_chunks(&chunks);

    //println!("{}",json);
    //println!("{}",json2);

    // force failure
    //lsysv2[0].angle = -1.0;

    // compare
    assert_eq!(lsysv1.len(), lsysv2.len());
    for i in 0..lsysv1.len() {
        assert_eq!(lsysv1[i],lsysv2[i]);
    }
}

/*----------------------------------------------------------------------
*/

#[test]
fn test_rules_apply_basic() {
    let rules:Rules = HashMap::from([
        ('A',"AB"),
        ('B',"A")
    ]);
    let start:&str = "A";

    assert_eq!(rules_apply_basic(&rules,start,0), "A");
    assert_eq!(rules_apply_basic(&rules,start,1), "AB");
    assert_eq!(rules_apply_basic(&rules,start,2), "ABA");
    assert_eq!(rules_apply_basic(&rules,start,3), "ABAAB");
    assert_eq!(rules_apply_basic(&rules,start,4), "ABAABABA");
}

/*----------------------------------------------------------------------
*/

#[test]
fn test_rules_minimize() {
    assert_eq!(rules_minimize("ABCD"),            ""      );
    assert_eq!(rules_minimize(ACTIONS),           ACTIONS );
    assert_eq!(rules_minimize("AFBfC+D-E[G]H|I"), ACTIONS );
}

/*----------------------------------------------------------------------
*/

#[test]
fn test_layout_boxes() {
    let lb = layout_boxes_make();
    let svg = layout_boxes_draw(&lb);
    //print!("bounding boxes{:#?}",&lb);
    let mut ds = doc_new();
    doc(&mut ds, DocAct::DocOpenPathTitle(
        &"layout_boxes.html",
        &"test_layout_boxes",
    ));
    doc(&mut ds, DocAct::PageStartComment(&""));
    doc(&mut ds, DocAct::PageAddFragment(&svg));
    doc(&mut ds, DocAct::PageEnd);
    doc(&mut ds, DocAct::DocClose);
}
