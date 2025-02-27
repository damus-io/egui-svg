use egui::emath::TSTransform;
use egui::epaint::{PathShape, PathStroke};
use egui::{pos2, vec2, Color32, Painter, Response, Sense, Shape, Ui, Vec2, Widget};
use usvg::{Options, Paint, Tree};

pub struct SVG {
    tree: Tree,
    scale: Option<f32>,
}

impl SVG {
    pub fn new(bytes: &'static [u8]) -> Result<Self, usvg::Error> {
        Ok(Self {
            tree: Tree::from_data(bytes, &Options::default())?,
            scale: None,
        })
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        // scale x/y until fits inside size
        let bbox = self.tree.root().abs_bounding_box();
        let scale_x = size.x / bbox.width();
        let scale_y = size.y / bbox.height();
        self.scale = Some(scale_x.min(scale_y));
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }

    fn render_nodes(&self, group: &usvg::Group, painter: &Painter) {
        for node in group.children() {
            self.render_node(node, painter);
        }
    }

    fn render_group(&self, group: &usvg::Group, painter: &Painter) {
        if !group.should_isolate() {
            self.render_nodes(group, painter);
        }
    }

    fn render_node(&self, node: &usvg::Node, painter: &Painter) {
        match node {
            usvg::Node::Group(ref group) => {
                self.render_group(group, painter);
            }
            usvg::Node::Text(ref text) => {
                self.render_group(text.flattened(), painter);
            }
            usvg::Node::Path(ref path) => {
                self.render_path(path, painter);
            }
            usvg::Node::Image(ref _image) => {
                todo!()
            }
        }
    }

    fn render_path(&self, path: &usvg::Path, painter: &Painter) {
        if !path.is_visible() {
            return;
        }

        // convert to PathShape
        let points = path
            .data()
            .points()
            .iter()
            .map(|p| pos2(p.x, p.y))
            .collect();
        let fill = if let Some(f) = path.fill() {
            Self::paint_to_color(f.paint())
        } else {
            Color32::default()
        };
        let stroke = if let Some(s) = path.stroke() {
            PathStroke::new(s.width().get(), Self::paint_to_color(s.paint()))
        } else {
            PathStroke::default()
        };
        let mut shape: Shape = PathShape::convex_polygon(points, fill, stroke).into();
        let transform = path.abs_transform();
        if transform.has_translate() {
            shape.translate(vec2(transform.tx, transform.ty));
        }
        if transform.has_scale() {
            shape.scale(transform.sx.min(transform.sy));
        }
        if let Some(scale) = self.scale {
            shape.transform(TSTransform::from_scaling(scale));
        }
        painter.add(shape);
    }

    fn paint_to_color(paint: &Paint) -> Color32 {
        match paint {
            Paint::Color(ref c) => Color32::from_rgb(c.red, c.green, c.blue),
            Paint::LinearGradient(_) => todo!(),
            Paint::RadialGradient(_) => todo!(),
            Paint::Pattern(_) => todo!(),
        }
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let group = self.tree.root();
        let size = Self::size_from_group(group, self.scale);
        let (response, painter) = ui.allocate_painter(size, Sense::click());
        self.render_group(group, &painter);
        response
    }

    pub fn size_from_group(group: &usvg::Group, scale: Option<f32>) -> Vec2 {
        let bbox = group.abs_bounding_box();

        if let Some(s) = scale {
            vec2(bbox.width() * s, bbox.height() * s)
        } else {
            vec2(bbox.width(), bbox.height())
        }
    }
}

impl Widget for SVG {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
