use std::collections::HashMap;
use std::fs;
use indoc::indoc;

/*----------------------------------------------------------------------
Lindenmayer System interpreter and display using SVG.

This is a rewrite of previous version from python/postscript.
*/

/*----------------------------------------------------------------------
Tune-able parameters
*/

static PARMS : [(&str, &str); 3 ] = [
  ("linewidth",   "0.02"      ),   // inches
  ("pagewidth",   "8.5"       ),   // inches
  ("pageheight", "11.0"       ),   // inches
  //titlefont = "/Times-Bold",
  //titlesize = 30,
  //attrfont = "/Arial",
  //attrsize = 12,
];

fn pget(key:&str) -> &str {
    let mut val:&str = "";
    for (k,v) in PARMS {
        if k == key {
            val = v;
            break;
        }
    }
    val
}

/*----------------------------------------------------------------------
Page layout bounding boxes

For convenience of page layout, define bounding boxes for
the regions, "top", "a", "b", "left", "center", "right", "main",
as diagrammed below.

Note that the page origin for SVG is at top left.  This is different
from that used by postscript which is at the bottom left.
The orientation of y-axis for SVG is inverted


    +-----------------0-----------------+
    |                top                |
    +------------1---+------------------+
    |                |                  |
    |       a        2        b         |
    |                |                  |
    +----------+-2---+-----+------------+
    |          |           |            |
    0 left     1  center   3   right    4
    |          |           |            |
    +----------+-----3-----+------------+
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |               main                |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    |                                   |
    +---------------4-------------------+
*/
type LayoutBoxes = HashMap<&'static str,(f64,f64,f64,f64)>;
fn make_layout_boxes() -> LayoutBoxes {

    // all box edges as fraction of page size
    //             0     1     2     3     4
    let xf = vec![0.05, 0.35, 0.50, 0.65, 0.95];
    let yf = vec![0.03, 0.14, 0.20, 0.42, 0.97];

    // scale to page size
    let width:f64  = pget("pagewidth").parse().unwrap();
    let x:Vec<f64> = xf.into_iter().map(|x| x * width).collect();
    let height:f64 = pget("pageheight").parse().unwrap();
    let y:Vec<f64> = yf.into_iter().map(|y| y * height).collect();

    // make named bounding boxes
    HashMap::from([
        (  "main"   , (x[0],y[3],x[4],y[4]) ),
        (  "left"   , (x[0],y[2],x[1],y[3]) ),
        (  "center" , (x[1],y[2],x[3],y[3]) ),
        (  "right"  , (x[3],y[2],x[4],y[3]) ),
        (  "a"      , (x[0],y[1],x[2],y[2]) ),
        (  "b"      , (x[2],y[1],x[4],y[2]) ),
        (  "top"    , (x[0],y[0],x[4],y[1]) ),
    ])
}

fn draw_layout_boxes(boxes: &LayoutBoxes) -> String {
    let mut svg = String::new();

    // prelude
    let s1 = format!( indoc! {r#"
        <!-- draw_layout_boxes -->
        <svg
            xmlns="http://www.w3.org/2000/svg"
            version="1.2"
            width="{pagewidth}in"
            height="{pageheight}in"
        >
        "#},
        pagewidth   = pget("pagewidth"),
        pageheight  = pget("pageheight"),
    );
    svg.push_str(&s1);

    // foreach box
    for (_k,v) in boxes {
        let s2 = format!( indoc! {r#"
            <rect
                x      = "{x0:.4}in"
                y      = "{y0:.4}in"
                rx     = "0.1in"
                ry     = "0.1in"
                width  = "{w:.4}in"
                height = "{h:.4}in"
                style  = "
                    fill           :  none;
                    stroke         :  black;
                    stroke-width   :  {strokewidth:.4}in;
                "
            />
            "#},
            x0=v.0,y0=v.1,w=v.2-v.0,h=v.3-v.1,
            strokewidth = pget("linewidth"),
        );
        svg.push_str(&s2);
    }

    // postlude
    let s3 = format!( indoc! {r#"
        </svg>
        "#}
    );
    svg.push_str(&s3);

    svg
}

fn test_layout_boxes() {
    let lb = make_layout_boxes();
    let svg = draw_layout_boxes(&lb);
    print!("bounding boxes{:#?}",&lb);
    _ = fs::write("layout_boxes.svg", svg);
}

/*----------------------------------------------------------------------
Elaborate Lindenmayer System

Apply rules iteratively until specified order is reached.
Use two strings, old and new, remove character from old string,
if it is a rule, do substitution, append to new string.
Exchange old and new after each iteration.
*/

type Rules = HashMap<char,&'static str>;
fn apply_rules(rules:&Rules, start:&str, order:i32) -> String {
    let mut new = String::from(start);
    for _ in 0..order {
        let mut old = new;
        new = "".to_string();
        while old != "" {
            let c = old.remove(0);
            match rules.get(&c) {
                Some(s) => new.push_str(s),
                None    => new.push(c),
            }
        }
    }
    new
}

fn test_apply_rules() {
    let rules:Rules = HashMap::from([
        ('A',"AB"),
        ('B',"A")
    ]);
    let start:&str = "A";

    assert!(apply_rules(&rules,start,0) == "A");
    assert!(apply_rules(&rules,start,1) == "AB");
    assert!(apply_rules(&rules,start,2) == "ABA");
    assert!(apply_rules(&rules,start,3) == "ABAAB");
    assert!(apply_rules(&rules,start,4) == "ABAABABA");
    println!("tested apply_rules");
}

/*----------------------------------------------------------------------
Top level
*/

fn main() {
    test_layout_boxes();
    test_apply_rules();
}
