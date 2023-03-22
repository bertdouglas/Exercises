use std::collections::HashMap;
use indoc::indoc;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::f64;

pub mod test_main;
use crate::test_main::*;

/*----------------------------------------------------------------------
Lindenmayer System interpreter and display using SVG.

An LSys is a set of rules for string substitution. There is a starting
string and a set of rule strings.  Each character in a string is either the
name of another rule, or a special action character.

The special action characters are:
F Move forward by line length drawing a line
f Move forward by line length without drawing a line
+ Turn left by turning angle
- Turn right by turning angle
| Reverse direction (ie: turn by 180 degrees)
[ Push current drawing state onto stack
] Pop current drawing state from the stack

The drawing state consists of:
- drawing direction
- drawing position

Structured Vector Graphics (SVG) is generated to draw the LSys.
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
type BBox = (f64,f64,f64,f64);
type LayoutBoxes<'a> = HashMap<&'a str,BBox>;
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
DrawCore

Produce svg to draw lsys string in abstract space, with only
relative moves and with step size of 1. Starting position is assumed to
be (0,0). This requires separate definition of actual starting
position and scale.

Returns svg as string and bounding box in steps.
*/

// collect drawing actions and then later produce svg
enum DAct {
    RmoveTo(f64,f64),
    RlineTo(f64,f64)
}

fn draw_core(curve:&Curve, actions:&str) -> (Vec<DAct>,BBox) {
    type DXY = (f64,f64,f64);
    let mut stack:Vec<DXY> = vec!();
    let mut dacts:Vec<DAct> = vec!();

    // direction and angle step
    let mut d:f64 = 0.0;
    let angle:f64 = curve.angle * PI / 180.0;

    // current position and bounding box
    let (mut x, mut y, mut x0, mut y0, mut x1, mut y1 )
      : (f64,   f64,   f64,    f64,    f64,    f64,   )
      = (0.0,   0.0,   0.0,    0.0,    0.0,    0.0,   );
    let (mut xt, mut yt) : (f64,f64);

    // do the actions
    for action in actions.chars() {
        // forward
        if 'F' == action {
            xt = d.cos();       yt = d.sin();
            x += xt;            y += yt;
            dacts.push(DAct::RlineTo(xt,yt));
        }
        else if '+' == action {
            d += angle;
        }
        else if '-' == action {
            d -= angle;
        }
        else if '[' == action {
            stack.push((d,x,y));
        }
        else if ']' == action {
            (d,xt,yt) = stack.pop().unwrap();
            dacts.push(DAct::RmoveTo(xt-x,yt-y));
            x = xt;  y = yt;
        }
        else if '|' == action {
            d += PI;
        }
        else {
            panic!("Unimplemented action: '{action}'");
        }
        // maintain bounding box
        x0 = f64::min(x0,x);     y0 = f64::min(y0,y);
        x1 = f64::max(x1,x);     y1 = f64::max(y1,y);
    }

    // adjust bounding box so it can't have zero size
    // treat as if it has 1 step, keep center at zero
    if x0==x1 { x0 = -0.5;  x1 = 0.5; }
    if y0==y1 { y0 = -0.5;  y1 = 0.5; }

    (dacts,(x0,y0,x1,y1))
}

/*----------------------------------------------------------------------
Elaborate Lindenmayer System

Apply rules iteratively until specified order is reached.
Use two strings, old and new, remove character from old string,
if it is a rule, do substitution, append to new string.
Exchange old and new after each iteration.
*/

pub type Rules<'a> = HashMap<char,&'a str>;

fn apply_rules_basic(rules:&Rules, start:&str, order:i32) -> String {
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
Higher level rule application

Apply both main rules and post rules.
The post rule substitution is used to allow use of rules from
sources that presume implicit drawing on rules other than F.
After all rule application, do minimization.
*/
fn apply_rules(curve:&Curve,order:i32) -> String {
    // do rule substition
    let basic = apply_rules_basic(&curve.rules,&curve.start,order);
    // do post rule substitution
    let post = apply_rules_basic(&curve.post_rules,&basic,1);
    minimize_rules(post)
}

/*----------------------------------------------------------------------
remove non-action characters from LSys rules
*/

fn minimize_rules(rules:String) -> String {

    let mut out = String::new();
    let actions:&str = "Ff+-[]|";
    for rule in rules.chars() {
        if actions.contains(rule) {
            out.push(rule);
        }
    }
    out
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
    order: Vec<i32>,
    #[serde(borrow)]
    rules: Rules<'a>,
    post_rules: Rules<'a>,
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
fn load_curves<'a>(chunks:&'a Vec<String>) -> Vec<Curve<'a>> {

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
                //println!("{:#?}",&curve);
                //println!("{}",&curve.title);
                curves.push(curve);
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
    if false { test_apply_rules_basic();  }
    if false { test_serde();        }

    let json = include_str!("curves.json");
    let chunks:Vec<String> = get_json_chunks(json);
    //println!("{:#?}",chunks);
    let curves = load_curves(&chunks);
}
