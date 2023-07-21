use serde::{Deserialize, Serialize};

use crate::prelude::{Disc, Line, Rectangle, RegularPolygon, ShapePainter};

#[derive(Serialize, Deserialize)]
pub struct Shapes(pub Vec<Shape>);

impl Shapes {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, shape: Shape) -> &mut Self {
        self.0.push(shape);
        self
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }

    pub fn deserialize(shapes: &str) -> serde_json::Result<Self> {
        serde_json::from_str(shapes)
    }

    pub fn draw(&self, mut painter: ShapePainter) {
        self.0.iter().for_each(|shape| shape.draw(&mut painter));
    }
}

#[derive(Serialize, Deserialize)]
pub enum Shape {
    Disc(Disc),
    Line(Line),
    Rect(Rectangle),
    Ngon(RegularPolygon),
}

impl Shape {
    pub fn draw(&self, painter: &mut ShapePainter) {
        match self {
            Shape::Disc(disc) => disc.draw(painter),
            Shape::Line(line) => line.draw(painter),
            Shape::Rect(rect) => rect.draw(painter),
            Shape::Ngon(ngon) => ngon.draw(painter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shapes_serialize_deserialize() {
        let mut shapes = Shapes::new();

        shapes
            .add(Shape::Disc(Disc::default()))
            .add(Shape::Line(Line::default()))
            .add(Shape::Ngon(RegularPolygon::default()))
            .add(Shape::Rect(Rectangle::default()));

        let serialized = shapes.serialize();
        let _deserialized = Shapes::deserialize(&serialized).unwrap();

        dbg!(serialized);
    }
}
