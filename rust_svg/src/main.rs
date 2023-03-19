use std::collections::HashMap;
use std::fs;

/*----------------------------------------------------------------------
This is rewrite of the python version into rust.
Output is generated in SVG (structured vector graphics), but
remains page oriented.
*/


/*----------------------------------------------------------------------
Tune-able parameters

You can't make a static hashmap in rust.
So go back to plist style.
*/

static PARMS : [(&str, &str) ; 3 ] = [
  ("linewidth",   "0.02"),        // inches
  ("pagewidth",   "8.5"),         // inches
  ("pageheight", "11.0"),         // inches
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
    let s1 = format!(r#"
<!-- draw_layout_boxes -->
<svg
    xmlns="http://www.w3.org/2000/svg"
    version="1.2"
    width="{pagewidth}in"
    height="{pageheight}in"
>
"#,
        pagewidth   = pget("pagewidth"),
        pageheight  = pget("pageheight"),
    );
    svg.push_str(&s1);

    // foreach box
    for (_k,v) in boxes {
        let s2 = format!(r#"
<rect
    x      = "{x0:.4}in"
    y      = "{y0:.4}in"
    rx     = "0.1in"
    ry     = "0.1in"
    width  = "{w:.4}in"
    height = "{h:.4}in"
    style = "
        fill           :  none;
        stroke         :  black;
        stroke-width   :  {strokewidth:.4}in;
    "
/>
"#,
            x0=v.0,y0=v.1,w=v.2-v.0,h=v.3-v.1,
            strokewidth = pget("linewidth"),
        );
        svg.push_str(&s2);
    }

    // postlude
    let s3 = format!(r#"
</svg>
"#
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
Top level
*/

fn main() {
    test_layout_boxes();
}
