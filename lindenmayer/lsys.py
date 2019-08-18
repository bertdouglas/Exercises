#!/usr/bin/python3

"""
Goal of this exercise is to learn about:
  - The object oriented approach to programming
  - Lindenmayer systems.  Amazing variety from simple rules.
  - Simple page layout with bounding boxes
  - Using one program to generate code for another (postscript output)

You also get to see some awesome visuals !

The major references for Lindenmayer Systems are:
  https://en.wikipedia.org/wiki/L-system
  https://en.wikipedia.org/wiki/The_Algorithmic_Beauty_of_Plants

Copyright: 2019 Bert Douglas
SPDX-License-Identifier: MIT
"""

#------------------------------------------------------------------------------
# Known bugs and missing features

"""
FIXME:  Add some color to make look better

FIXME:  Add pdfmark for clickable hyperlinks on refs.  It already just works
on google pdf viewer, but not on evince.

FIXME:  Bug in adobe DSC page numbers.
        grep '%%Page' lsys-examples.ps
          %%Pages: 11
          %%Page: 1 11
          %%Page: 1 12
          %%Page: 1 13
          %%Page: 1 14
          %%Page: 1 15
          %%Page: 1 16
        Study how others do this.  I suspect the i-of-n format is not common.
        I think you can wildcard the number of pages. Maybe like this:
          %%Page: 1 ?
        Memory is unsure.
"""

#------------------------------------------------------------------------------
# tuneable parameters

PARMS = dict(
  # units are postscript points
  linewidth =   1.0,
  pagewidth =   8.5 * 72,
  pageheight = 11.0 * 72,
  titlefont = "/Times-Bold",
  titlesize = 30,
  attrfont = "/Arial",
  attrsize = 12,
)

#------------------------------------------------------------------------------
# imports and abbreviations

import math
import sys
import datetime
import pprint
pp = pprint.PrettyPrinter(indent=4).pprint

def die(str_msg):
  raise Exception(str_msg)

#------------------------------------------------------------------------------
# Adobe Document Structuring Conventions for postscript

# https://en.wikipedia.org/wiki/Document_Structuring_Conventions
# https://www-cdf.fnal.gov/offline/PostScript/5001.PDF

class AdobeDSC :
  """
    Accept one page at a time and write to output stream.
    Add DSC at beginning of document and for each page.
  """
  def __init__(self,title,npages,ostream) :

    self._npages = npages
    self._page = 1
    self._ostream = ostream
    self._out = []

    # doc prefix
    pbb = (0,0,PARMS['pagewidth'],PARMS['pageheight'])
    (x0,y0,x1,y1) = pbb
    date = datetime.datetime.now().isoformat()
    doc_prefix = (
      f"%!PS-Adobe-3.0\n"
      f"%%Title: {title}\n"
      f"%%Creator: 'lsys.py' Copyright 2019 Bert Douglas\n"
      f"%%CreationDate: {date}\n"
      f"%%BoundingBox: {x0} {y0} {x1} {y1}\n"
      f"%%Pages: {npages}\n"
      f"%%EndComments\n"
    )
    self._out += [doc_prefix]

  def AddPage(self,ps) :
    """
      emit page prefix
      emit page data
      show page
      count pages
    """
    page_prefix = (
      f"\n%%Page: {self._page} {self._npages}\n"
    )
    out = self._out
    out += [page_prefix]
    out += ps
    out += ["\nshowpage\n"]
    sout = "".join(out)
    self._ostream.write(sout)
    self._out = []
    self._npages += 1

  def Finish(self) :
    """
      Finish document
    """
    doc_suffix = "\n%%EOF\n"
    self._ostream.write(doc_suffix)

#------------------------------------------------------------------------------
# Lindenmayer System

"""
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

  Postscript is generated to draw the LSys.
"""

class LSys :
  def __init__(self,**props) :
    # Copy keyword args to named variables
    # These are required
    self._Title = props['Title']
    self._Rules = props['Rules']
    # These are optional
    self._Refs = props.get('Refs',[])
    self._PostRules = props.get('PostRules',{})
    self._Rules['ShowOrder'] = props['Rules'].get('ShowOrder',[1,2,3,6])
    # all possible actions
    self._actions = "Ff+-[]|"
    # all actions that perform drawing
    self._drawing_actions = "F"

  def ElabCore(self,rules,start,order) :
    """
      Use two lists.  Remove item from 'old' list,
      if a rule, then do rule substitution
      append to 'new' list
    """
    new = list(start)
    for _ in range(order):
      old = new
      new = []
      while old != []:
        c = old.pop(0)
        new += list(rules.get(c,c))
    return ''.join(new)


  def Elaborate(self, order) :
    """
    Produce LSys string, elaborated to specified order.

    If the start string does drawing, one is subtracted from the order.
    The goal is to make order 1 to produce the simplest non-null drawing.
    An elaboration of order 0 always returns the start string.

    The post rule substitution is used to allow use of rules from
    sources that presume implicit drawing on rules other than F.
    """

    # decrement order if start does drawing action
    drawset = set(self._drawing_actions)
    startset = set(self._Rules['Start'])
    if 0 < len(drawset.intersection(startset)) :
      order -= 1

    # do rule substition
    ecore = self.ElabCore(self._Rules,self._Rules['Start'],order)

    # do post rule substitution
    # FIXME this is slower than it should be, remove this special case
    pr = self._PostRules
    if 0 != len(pr) :
      epost = self.ElabCore(pr,ecore,1)
    else:
      epost = ecore
    return epost

  def Minimize(self,s) :
    """
      Remove all non-action characters from LSys string.
    """
    s1 = []
    for c in s:
      if c in self._actions:
        s1 += [c]
    smin = "".join(s1)
    return smin

  def DrawCore(self,actions) :
    """
      Produce postscript to draw action string in abstract space, with only
      relative moves and with step size of 1. Starting position is assumed to
      be (0,0). This requires separate, earlier, definition of actual starting
      position and scale.

      Returns postscript as list of strings and bounding box in steps.
    """
    stack = []
    ps = []
    # direction and angle step
    d = 0.0
    angle = -self._Rules['Angle'] * math.pi / 180.0
    # current position and bounding box
    x = y = x0 = y0 = x1 = y1 = 0.0
    # do the actions
    for action in actions:
      # forward
      if "F" == action:
        xs = math.cos(d);   ys = math.sin(d)
        xs = round(xs,15);  ys = round(ys,15)
        x  += xs;           y  += ys
        x  = round(x,15);   y  = round(y,15)
        ps += [f"{xs} {ys} rlineto\n"]
      elif "+" == action:
        d += angle
      elif "-" == action:
        d -= angle
      elif "[" == action:
        stack.append((d,x,y))
      elif "]" == action:
        (d,xp,yp) = stack.pop()
        ps += [f"{xp-x} {yp-y} rmoveto\n"]
        x = xp;  y = yp;
      elif "|" == action:
        d += math.pi
      else:
        die(f"Unimplemented action: '{action}'")

      # maintain bounding box
      x0 = min(x0,x);     y0 = min(y0,y)
      x1 = max(x1,x);     y1 = max(y1,y)

    # adjust bounding box so it can't have zero size
    # treat as if it has 1 step, keep center at zero
    if x0==x1 : x0 = -0.5;  x1 = +0.5
    if y0==y1 : y0 = -0.5;  y1 = +0.5

    return (ps,(x0,y0,x1,y1))

  def DrawBasic(self, order, pbb) :
    """
      Produce postscript to draw LSys at specified order to fit in specified
      layout box on page.  Units of pbb are postscript points.

      Returns postscript as a list of strings
    """
    (px0,py0,px1,py1) = pbb
    elab = self.Elaborate(order)
    actions = self.Minimize(elab)
    (pscore,abb) = self.DrawCore(actions)
    (ax0,ay0,ax1,ay1) = abb

    # find scale factor
    sx = (px1-px0)/(ax1-ax0)
    sy = (py1-py0)/(ay1-ay0)
    scale = min(sx,sy) * 0.9

    #pp(abb); pp(pbb)
    #pp(sx); pp(sy); pp(scale)

    # find starting position
    x = (px0+px1)/2.0 - scale*(ax0+ax1)/2.0
    y = (py0+py1)/2.0 - scale*(ay0+ay1)/2.0

    # make postscript
    lw = PARMS['linewidth']
    ps = []
    ps += [f"\n%DrawBasic({order},({px0},{py0},{px1},{py1}))\n"]
    ps += [f"gsave\n"]
    ps += [f"newpath\n"]
    ps += [f"{x} {y} moveto\n"]
    ps += [f"{scale} dup scale\n"]
    ps += [f"{lw/scale} setlinewidth\n"]
    ps += pscore
    ps += [f"stroke\n"]
    ps += [f"grestore\n"]
    return ps

  def LayoutBoxes(self):
    """
    Return bounding boxes for layout regions
    "top", "left", "center", "right", "main"
    as shown below:
    +------------3----------------------+
    |                                   |
    |                top                |
    |                                   |
    |                                   |
    +-------+-----2-----+---------------+
    |       |           |               |
    0 left  1  center   2   right       3
    |       |           |               |
    +-------+-----1-----+---------------+
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
    +---------------0-------------------+
    """
    # all box edges as fraction of page size
    #      0     1     2     3
    xf = [0.05, 0.35, 0.65, 0.95]
    yf = [0.03, 0.58, 0.80, 0.97]
    # scale to page size
    x = [s * PARMS['pagewidth']  for s in xf]
    y = [s * PARMS['pageheight'] for s in yf]
    # make named bounding boxes
    bb = dict(
      m = (x[0],y[0],x[3],y[1]),
      l = (x[0],y[1],x[1],y[2]),
      c = (x[1],y[1],x[2],y[2]),
      r = (x[2],y[1],x[3],y[2]),
      t = (x[0],y[2],x[3],y[3]),
    )
    return bb

  def DrawFancy(self):
    bb = self.LayoutBoxes()
    so = self._Rules['ShowOrder']
    psl = self.DrawBasic(so[0],bb['l'])
    psc = self.DrawBasic(so[1],bb['c'])
    psr = self.DrawBasic(so[2],bb['r'])
    psm = self.DrawBasic(so[3],bb['m'])
    pst = self.DrawTop()
    # draw outlines of layout boxes
    if False:
      psb = self.DrawBoxOutlines()
    else:
      psb = []
    return psl+psc+psr+psm+pst+psb

  def DrawTop(self):
    """
      Draw all annotations in any box
    """
    bbs = self.LayoutBoxes()
    (x0,y0,x1,y1) = bbs['t']
    ps = []
    ps += ["\n%DrawTop\n"]
    ps += ["gsave\n"]

    # Title
    tf = PARMS['titlefont']
    ts = PARMS['titlesize']
    t = self._Title
    ps += [f"{tf} findfont\n"]
    ps += [f"{ts} scalefont setfont\n"]
    ps += [f"{x1-x0} ({t}) stringwidth pop sub 2 div\n"]
    ps += [f"{y1-ts} moveto\n"]
    ps += [f"({t}) show\n"]

    # Rules
    atf = PARMS['attrfont']
    ats = PARMS['attrsize']
    ps += [f"{atf} findfont\n"]
    ps += [f"{ats} scalefont setfont\n"]
    ps += [f"{x0} {y1-ts-ats} moveto\n"]
    for k,v in self._Rules.items():
      ps += [f"0.0 {-ats} rmoveto\n"]
      ps += [f"({k} : {v}) gsave show grestore\n"]

    # References
    ps += [f"0.0 {-ats} rmoveto\n"]
    for ref in self._Refs:
      ps += [f"0.0 {-ats} rmoveto\n"]
      ps += [f"({ref}) gsave show grestore\n"]

    ps += ["grestore\n"]
    return ps

  def DrawBoxOutlines(self):
    ps = []
    ps += ["\n%DrawBoxOutlines\n"]
    ps += ["gsave\n"]
    lw = PARMS['linewidth']
    ps += [f"{lw} setlinewidth\n"]
    ps += [f"1 setlinejoin\n"]
    for bb in self.LayoutBoxes().values():
      (x0,y0,x1,y1) = bb
      ps += [f"newpath\n"]
      ps += [f"{x0} {y0} moveto\n"]
      ps += [f"{x0} {y1} lineto\n"]
      ps += [f"{x1} {y1} lineto\n"]
      ps += [f"{x1} {y0} lineto\n"]
      ps += [f"{x0} {y0} lineto\n"]
      ps += [f"closepath stroke\n"]
    ps += ["grestore\n"]
    return ps

#------------------------------------------------------------------------------
# A few example curves

Curves = dict(

  Hilbert = LSys(
    Title = "Hilbert Curve",
    Refs = [
      "https://www.cs.unh.edu/~charpov/programming-lsystems.html"
    ],
    Rules = dict(
      Angle = 90.0,
      Start = "X",
      X = "-YF+XFX+FY-",
      Y = "+XF-YFY-FX+",
    ),
  ),

  Koch = LSys(
    Title = "Koch's Snowflake",
    Refs = [
      "https://www.cs.unh.edu/~charpov/programming-lsystems.html"
    ],
    Rules = dict(
      Angle = 60.0,
      Start = "+F--F--F",
      F = "F+F--F+F",
    ),
  ),

  Peano = LSys(
    Title = "Peano Curve aka Hilbert II",
    Refs = [
      "http://bl.ocks.org/nitaku/8949471",
      "http://mathworld.wolfram.com/HilbertCurve.html",
    ],
    Rules = dict(
      Angle = 90.0,
      Start = "L",
      L = "LFRFL-F-RFLFR+F+LFRFL",
      R = "RFLFR+F+LFRFL-F-RFLFR",
      ShowOrder = [1,2,3,4],
    ),
  ),

  Gosper = LSys(
    Title = "Peano-Gosper Curve aka 'Flowsnake'",
    Refs = [
      "https://en.wikipedia.org/wiki/Gosper_curve"
      "http://larryriddle.agnesscott.org/ifs/ksnow/flowsnake.htm"
    ],
    Rules = dict(
      Angle = 60.0,
      Start = "A",
      A = "A-B--B+A++AA+B-",
      B = "+A-BB--B-A++A+B",
      ShowOrder = [1,2,3,4],
    ),
    PostRules = dict(
      A = "F",
      B = "F",
    ),
  ),

  QGosper = LSys(
    Title = "Quadratic Gosper",
    Refs = ["http://paulbourke.net/fractals/lsys/"
    ],
    Rules = dict(
      Angle = 90.0,
      Start = "YF",
      X = "XFX-YF-YF+FX+FX-YF-YFFX+YF+FXFXYF-FX+YF+FXFX+YF-FXYF-YF-FX+FX+YFYF-",
      Y = "+FXFX-YF-YF+FX+FXYF+FX-YFYF-FX-YF+FXYFYF-FX-YFFX+FX+YF-YF-FX+FX+YFY",
      ShowOrder = [1,2,3,4],
    ),
  ),

  SierpD = LSys(
    Title = "Sierpinski Diamond",
    Refs = ["http://paulbourke.net/fractals/lsys/",
    ],
    Rules = dict(
      Angle = 90.0,
      Start = "F+XF+F+XF",
      X = "XF-F+F-XF+F+XF-F+F-X",
      ShowOrder = [2,3,4,5],
    ),
  ),

  SierpA = LSys(
    Title = "Sierpinski Arrowhead",
    Refs = ["http://paulbourke.net/fractals/lsys/",
    ],
    Rules = dict(
      Angle = 60,
      Start = "YF",
      X = "YF+XF+Y",
      Y = "XF-YF-X",
      ShowOrder = [2,3,4,8],
    ),
  ),

  Pent1 = LSys(
    Title = "Pentaplexity",
    Refs = ["http://paulbourke.net/fractals/lsys/",
    ],
    Rules = dict(
      Angle = 36.0,
      Start = "F++F++F++F++F",
      F = "F++F++F|F-F++F",
      ShowOrder = [1,2,3,4],
    ),
  ),

  Dragon = LSys(
    Title = "Dragon Curve",
    Refs = ["http://paulbourke.net/fractals/lsys/",
    ],
    Rules = dict(
      Angle = 90.0,
      Start = "FX",
      X = "X+YF+",
      Y = "-FX-Y",
      ShowOrder = [2,4,6,14],
    ),
  ),

  Plant1 = LSys(
    Title = "Plant 1",
    Refs = [
      "https://www.cs.unh.edu/~charpov/programming-lsystems.html",
    ],
    Rules = dict(
      Angle = 22.5,
      Start = "----X",
      X = "F+[[X]-X]-F[-FX]+X",
      F = "FF",
    ),
  ),

  Plant2 = LSys(
    Title = "Plant 2",
    Refs = [
      "https://www.cs.unh.edu/~charpov/programming-lsystems.html",
    ],
    Rules = dict(
      Angle = 22.5,
      Start = "----F",
      F = "FF-[-F+F+F]+[+F-F-F]",
    ),
  ),

)

#------------------------------------------------------------------------------
# Tests

def Test1():
  print("\n\nBegin Test1")
  lsys = Curves["Hilbert"]
  pp(vars(lsys))
  s = lsys.Elaborate(1)
  pp(s)
  s1 = lsys.Minimize(s)
  pp(s1)
  (ps,bb) = lsys.DrawCore(s1)
  pp(ps)
  pp(bb)

def Test2():
  # should die
  print("\n\nBegin Test2")
  lsys = Curves["Hilbert"]
  (ps,bb) = lsys.DrawCore(lsys._actions+'&')

def Test3():
  lsys = Curves["Hilbert"]
  ps = lsys.DrawBasic(1,(200,200,250,250))
  pp(ps)

def TestAll():
  #Test1()
  #Test2()
  Test3()

def TestBox() :
  ostream = open("boxes.ps","w")
  dsc = AdobeDSC("TestBox",1,ostream)
  lsys = Curves["Hilbert"]
  psb = lsys.DrawBoxOutlines()
  pst = lsys.DrawTop()
  bb = lsys.LayoutBoxes()['l']
  psl = lsys.DrawBasic(1,bb)
  dsc.AddPage(psb+pst+psl)
  dsc.Finish()
  ostream.close()

def TestElab() :
  lsys = Curves["Hilbert"]
  for i in range(6):
    e = lsys.Elaborate(i)
    print()
    print(i)
    pp(e)

#TestBox()
#TestElab()

# single curve
#DoCurves({'Hilbert':Curves['Hilbert']})
#DoCurves({'Plant1':Curves['Plant1']})


#------------------------------------------------------------------------------
# Print Curves

# One page for each curve.
# Curves passed as dictionary of LSys.


def DoCurves(curves) :
  # input is a dictionary of lsys objects
  # get path
  npages = len(curves)
  if 1 == npages :
    # one object, use key for name
    opath = list(curves.keys())[0].lower()
  else :
    opath = "lsys-examples"
  ostream = open(opath +".ps","w")
  title = "Lindenmayer System Examples"
  dsc = AdobeDSC(title,npages,ostream)
  # iterate over curves
  for lsys in curves.values():
    ps = lsys.DrawFancy()
    dsc.AddPage(ps)
  dsc.Finish()
  ostream.close()

#------------------------------------------------------------------------------
# Top level code

DoCurves(Curves)



