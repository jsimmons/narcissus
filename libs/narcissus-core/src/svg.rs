use std::fmt;

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(Rgb { r, g, b })
}

pub const fn hsl(h: u32, s: u32, l: u32) -> Color {
    Color::Hsl(Hsl { h, s, l })
}

pub const fn black() -> Color {
    rgb(0, 0, 0)
}

pub const fn white() -> Color {
    rgb(255, 255, 255)
}

pub const fn red() -> Color {
    rgb(255, 0, 0)
}

pub const fn green() -> Color {
    rgb(0, 255, 0)
}

pub const fn blue() -> Color {
    rgb(0, 0, 255)
}

pub const fn fill(color: Color, alpha: f32) -> Fill {
    Fill::Color(color, alpha)
}

pub const fn stroke(color: Color, width: f32, alpha: f32) -> Stroke {
    Stroke::Color(color, width, alpha)
}

pub const fn style(fill: Fill, stroke: Stroke) -> Style {
    Style { fill, stroke }
}

pub fn svg_begin(w: f32, h: f32) -> SvgBegin {
    SvgBegin { w, h }
}

pub fn svg_end() -> SvgEnd {
    SvgEnd
}

pub fn text<'a, T>(x: f32, y: f32, size: f32, style: Style, text: &'a T) -> Text<'a, T>
where
    T: fmt::Display,
{
    Text {
        x,
        y,
        size,
        style,
        text,
    }
}

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect<'static> {
    Rect {
        x,
        y,
        w,
        h,
        style: Default::default(),
        border_radius: 0.0,
        title: None,
    }
}

pub fn circle(x: f32, y: f32, r: f32) -> Circle {
    Circle {
        x,
        y,
        r,
        style: Default::default(),
    }
}

pub fn path_begin() -> PathBegin {
    PathBegin {
        style: Default::default(),
    }
}

pub fn path_end() -> PathEnd {
    PathEnd
}

pub fn path_move_to(x: f32, y: f32) -> PathOp {
    PathOp::MoveTo { x, y }
}

pub fn path_line_to(x: f32, y: f32) -> PathOp {
    PathOp::MoveTo { x, y }
}

pub fn path_quadratic(x1: f32, y1: f32, x: f32, y: f32) -> PathOp {
    PathOp::QuadraticTo { x1, y1, x, y }
}

pub fn path_cubic(x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) -> PathOp {
    PathOp::CubicTo {
        x1,
        y1,
        x2,
        y2,
        x,
        y,
    }
}

pub fn path_close() -> PathOp {
    PathOp::Close
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Hsl {
    pub h: u32,
    pub s: u32,
    pub l: u32,
}

impl fmt::Display for Hsl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let h = self.h % 360;
        let s = self.s.min(100);
        let l = self.s.min(100);
        write!(f, "hsl({h}, {s}%, {l}%)")
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Color {
    Hsl(Hsl),
    Rgb(Rgb),
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Hsl(hsl) => write!(f, "{hsl}"),
            Color::Rgb(rgb) => write!(f, "{rgb}"),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Fill {
    None,
    Color(Color, f32),
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fill::None => write!(f, "fill:none;"),
            Fill::Color(color, opacity) => write!(f, "fill:{color};fill-opacity:{opacity};"),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Stroke {
    None,
    Color(Color, f32, f32),
}

impl fmt::Display for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stroke::None => write!(f, "stroke:none"),
            Stroke::Color(color, opacity, width) => write!(
                f,
                "stroke:{color};stroke-opacity:{opacity};stroke-width:{width}"
            ),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Style {
    pub fill: Fill,
    pub stroke: Stroke,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: fill(black(), 1.0),
            stroke: Stroke::None,
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.fill, self.stroke)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Rect<'a> {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub style: Style,
    pub border_radius: f32,
    pub title: Option<&'a str>,
}

impl<'a> Rect<'a> {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn border_radius(mut self, border_radius: f32) -> Self {
        self.border_radius = border_radius;
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

impl<'a> fmt::Display for Rect<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(title) = self.title {
            write!(
                f,
                r#"<rect x="{}" y="{}" width="{}" height="{}" ry="{}" style="{}"><title>{title}</title></rect>""#,
                self.x, self.y, self.w, self.h, self.border_radius, self.style
            )
        } else {
            write!(
                f,
                r#"<rect x="{}" y="{}" width="{}" height="{}" ry="{}" style="{}" />""#,
                self.x, self.y, self.w, self.h, self.border_radius, self.style
            )
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub style: Style,
}

impl Circle {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"<circle cx="{}" cy="{}" r="{}" style="{}" />""#,
            self.x, self.y, self.r, self.style,
        )
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct PathBegin {
    style: Style,
}

impl PathBegin {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl fmt::Display for PathBegin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"<path style="{}" d=""#, self.style)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PathOp {
    MoveTo {
        x: f32,
        y: f32,
    },
    LineTo {
        x: f32,
        y: f32,
    },
    QuadraticTo {
        x1: f32,
        y1: f32,
        x: f32,
        y: f32,
    },
    CubicTo {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x: f32,
        y: f32,
    },
    Close,
}

impl fmt::Display for PathOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathOp::MoveTo { x, y } => write!(f, "M {x} {y} "),
            PathOp::LineTo { x, y } => write!(f, "L {x} {y} "),
            PathOp::QuadraticTo { x1, y1, x, y } => write!(f, "Q {x1} {y1} {x} {y} "),
            PathOp::CubicTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => write!(f, "C {x1} {y1} {x2} {y2} {x} {y} "),
            PathOp::Close => write!(f, "Z "),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct PathEnd;

impl fmt::Display for PathEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"" />"#)
    }
}

pub struct Text<'a, T>
where
    T: fmt::Display,
{
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub style: Style,
    pub text: &'a T,
}

impl<'a, T> fmt::Display for Text<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"<text x="{}" y="{}" style="font-size:{};{}">{}</text>"#,
            self.x, self.y, self.size, self.style, self.text
        )
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct SvgBegin {
    pub w: f32,
    pub h: f32,
}

impl fmt::Display for SvgBegin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
            self.w, self.h,
        )
    }
}

#[derive(Copy, Clone)]
pub struct SvgEnd;

impl fmt::Display for SvgEnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "</svg>")
    }
}
