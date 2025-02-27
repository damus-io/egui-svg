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

    fn render_nodes(group: &usvg::Group, painter: &Painter, scale: Option<f32>) {
        for node in group.children() {
            Self::render_node(node, painter, scale);
        }
    }

    fn render_group(group: &usvg::Group, painter: &Painter, scale: Option<f32>) {
        if !group.should_isolate() {
            Self::render_nodes(group, painter, scale);
        }
    }

    fn render_node(node: &usvg::Node, painter: &Painter, scale: Option<f32>) {
        match node {
            usvg::Node::Group(ref group) => {
                Self::render_group(group, painter, scale);
            }
            usvg::Node::Text(ref text) => {
                Self::render_group(text.flattened(), painter, scale);
            }
            usvg::Node::Path(ref path) => {
                Self::render_path(path, painter, scale);
            }
            usvg::Node::Image(ref _image) => {
                todo!()
            }
        }
    }

    fn render_path(path: &usvg::Path, painter: &Painter, scale: Option<f32>) {
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
        if let Some(scale) = scale {
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

    pub fn show_tree(ui: &mut Ui, tree: &Tree, scale: Option<f32>) -> Response {
        let group = tree.root();
        let size = Self::size_from_group(group, scale);
        let (response, painter) = ui.allocate_painter(size, Sense::click());
        Self::render_group(group, &painter, scale);
        response
    }

    pub fn show_scaled(&self, ui: &mut Ui, scale: f32) -> Response {
        Self::show_tree(ui, &self.tree, Some(scale))
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        Self::show_tree(ui, &self.tree, self.scale)
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
