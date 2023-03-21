use std::collections::HashMap;
use indoc::indoc;
use serde::{Deserialize, Serialize};

pub mod test_main;
use crate::test_main::*;

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
type LayoutBoxes<'a> = HashMap<&'a str,(f64,f64,f64,f64)>;
fn make_layout_boxes() -> LayoutBoxes<'static> {

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

/*----------------------------------------------------------------------
Elaborate Lindenmayer System

Apply rules iteratively until specified order is reached.
Use two strings, old and new, remove character from old string,
if it is a rule, do substitution, append to new string.
Exchange old and new after each iteration.
*/

pub type Rules<'a> = HashMap<char,&'a str>;

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

/*----------------------------------------------------------------------
 Work with top level curves
*/

#[derive(Debug, Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Curve<'a> {
    title: String,
    refs:  Vec<String>,
    start: String,
    angle: f64,
    order: Option<Vec<i32>>,
    #[serde(borrow)]
    rules: Rules<'a>,
    post_rules: Option<Rules<'a>>,
}

// split json file into chunks corresponding to top level objects
// assumes that objects begin with line containing only "{"
// and end with line containing only "}"
fn get_json_chunks(json:&str) -> Vec<String> {
    let mut chunks:Vec<String> = vec!();
    let mut chunk = String::new();
    let mut inchunk:bool = false;
    for line in json.lines() {
        let l = line.trim_end();
        match (l, inchunk) {
        ("{",_) =>  {
                // begin chunk, or discard false chunk
                inchunk = true;
                chunk = "".to_string();
                chunk = chunk + line + "\n";
            }
        ("}",true) =>  {
                // end chunk
                inchunk = false;
                chunk = chunk + line + "\n";
                chunks.push(chunk.clone());
            }
        (_,true) =>  {
                // accumulate lines in chunk
                chunk = chunk + line + "\n";
            }
        (_,_) =>  {
                // ignore the rest
            }
        }
    }
    chunks
}

// load curves from json chunks
fn load_curves(chunks:Vec<String>) -> Vec<Curve<'static>> {

    // iterate over chunks of lines with serde
    let mut curves:Vec<Curve> = vec!();
    let mut chunk_no = 0;
    let mut errcnt = 0;
    let mut okcnt = 0;
    for chunk in chunks {
        chunk_no += 1;
        let r = serde_json::from_str::<Curve>(&chunk);
        match r {
            Err(why) => {
                errcnt += 1;
                println!();
                println!("Failed to read chunk {}",chunk_no);
                println!("--------------------------------");
                println!("{}",&chunk);
                println!("--------------------------------");
                println!("{:?}", why);
                println!();
            }
            Ok(curve) => {
                okcnt += 1;
                println!("{:#?}",&curve);
                //curves.push(curve.clone());
            }
        }
    }
    println!("Successfully loaded {} of {} curves",
        okcnt,okcnt+errcnt);

    curves
}

/*----------------------------------------------------------------------
Top level
*/

fn main() {
    if false { test_layout_boxes(); }
    if false { test_apply_rules();  }
    if false { test_serde();        }

    let json = include_str!("curves.json");
    let chunks:Vec<String> = get_json_chunks(json);
    //println!("{:#?}",chunks);
    let curves = load_curves(chunks);
}
