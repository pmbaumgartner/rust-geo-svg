extern crate geo_normalized;
extern crate geo_types;

use geo_types::{
    Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPolygon,
    Polygon, Rect, Triangle,
};
use geo_normalized::Normalized;
use std::fmt;
use num_traits;

pub trait ToSvg {
    fn to_svg(&self) -> String;
}

pub trait ToSvgString {
    fn to_svg_string(&self) -> String;
}

/** Geometries */

impl<T: num_traits::Float + fmt::Display> ToSvg for GeometryCollection<T> {
    fn to_svg(&self) -> String {
        if self.is_empty() {
            "".into()
        } else {
            self.0
                .iter()
                .map(|p| p.to_svg())
                .collect::<Vec<String>>()
                .join("\n")
        }
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for GeometryCollection<T> {
    fn to_svg_string(&self) -> String {
        if self.is_empty() {
            "".into()
        } else {
            self.0
                .iter()
                .map(|p| p.to_svg_string())
                .collect::<Vec<String>>()
                .join("")
        }
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvg for Geometry<T> {
    fn to_svg(&self) -> String {
        match self {
            Geometry::MultiPolygon { .. } => self.clone().into_multi_polygon().unwrap().to_svg(),
            Geometry::Polygon { .. } => self.clone().into_polygon().unwrap().to_svg(),
            Geometry::MultiLineString { .. } => {
                self.clone().into_multi_line_string().unwrap().to_svg()
            }
            Geometry::LineString { .. } => self.clone().into_line_string().unwrap().to_svg(),
            _ => "".into(),
        }
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for Geometry<T> {
    fn to_svg_string(&self) -> String {
        match self {
            Geometry::MultiPolygon { .. } => {
                self.clone().into_multi_polygon().unwrap().to_svg_string()
            }
            Geometry::Polygon { .. } => self.clone().into_polygon().unwrap().to_svg_string(),
            Geometry::MultiLineString { .. } => self
                .clone()
                .into_multi_line_string()
                .unwrap()
                .to_svg_string(),
            Geometry::LineString { .. } => self.clone().into_line_string().unwrap().to_svg_string(),
            Geometry::Line { .. } => self.clone().into_line().unwrap().to_svg_string(),
            _ => "".into(),
        }
    }
}

/** Polygons */

impl<T: num_traits::Float + fmt::Display> ToSvg for MultiPolygon<T> {
    fn to_svg(&self) -> String {
        multi_polygon_to_svg(self)
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for MultiPolygon<T> {
    fn to_svg_string(&self) -> String {
        multi_polygon_to_svg_string(self)
    }
}

fn multi_polygon_to_svg<T: num_traits::Float + fmt::Display>(poly: &MultiPolygon<T>) -> String {
    if poly.0.is_empty() {
        "".into()
    } else {
        poly.0
            .iter()
            .map(|p| polygon_to_svg(&p))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

fn multi_polygon_to_svg_string<T: num_traits::Float + fmt::Display>(
    poly: &MultiPolygon<T>,
) -> String {
    if poly.0.is_empty() {
        "".into()
    } else {
        poly.0
            .iter()
            .map(|p| polygon_to_svg_string(&p))
            .collect::<Vec<String>>()
            .join("")
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvg for Polygon<T> {
    fn to_svg(&self) -> String {
        polygon_to_svg(self)
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for Polygon<T> {
    fn to_svg_string(&self) -> String {
        polygon_to_svg_string(self)
    }
}

fn polygon_to_svg<T: num_traits::Float + fmt::Display>(poly: &Polygon<T>) -> String {
    if poly.exterior().0.is_empty() {
        "".into()
    } else {
        format!("<path d=\"M{}\"/>", polygon_rings_to_svg(poly))
    }
}

fn polygon_to_svg_string<T: num_traits::Float + fmt::Display>(poly: &Polygon<T>) -> String {
    if poly.exterior().0.is_empty() {
        "".into()
    } else {
        format!("M{}", polygon_rings_to_svg(poly))
    }
}

fn polygon_rings_to_svg<T: num_traits::Float + fmt::Display>(poly: &Polygon<T>) -> String {
    let mut lines: Vec<LineString<T>> = poly.interiors().into();
    let exterior: &LineString<T> = poly.exterior();
    lines.insert(0, exterior.clone());

    lines
        .iter()
        .map(|l| poly_ring_to_svg(&l))
        .collect::<Vec<String>>()
        .join("M")
}

fn poly_ring_to_svg<T: num_traits::Float + fmt::Display>(line: &LineString<T>) -> String {
    line.0
        .iter()
        .map(|c| coord_to_svg(&c))
        .collect::<Vec<String>>()
        .join("L")
}

/** Rect */

impl<T: num_traits::Float + fmt::Display> ToSvg for Rect<T> {
    fn to_svg(&self) -> String {
        rect_to_svg(self)
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for Rect<T> {
    fn to_svg_string(&self) -> String {
        rect_to_svg_string(self)
    }
}

fn rect_to_svg<T: num_traits::Float + fmt::Display>(rect: &Rect<T>) -> String {
    format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
        rect.min.x,
        rect.min.y,
        rect.width(),
        rect.height()
    )
}

fn rect_to_svg_string<T: num_traits::Float + fmt::Display>(rect: &Rect<T>) -> String {
    format!(
        "M{} {}L{} {}L{} {}L{} {}Z",
        rect.min.x,
        rect.min.y,
        rect.min.x,
        rect.min.y + rect.height(),
        rect.min.x + rect.width(),
        rect.min.y + rect.height(),
        rect.min.x + rect.width(),
        rect.min.y,
    )
}

/** Triangle */

impl<T: num_traits::Float + fmt::Display> ToSvg for Triangle<T> {
    fn to_svg(&self) -> String {
        triangle_to_svg(self)
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for Triangle<T> {
    fn to_svg_string(&self) -> String {
        triangle_to_svg_string(self)
    }
}

fn triangle_to_svg<T: num_traits::Float + fmt::Display>(triangle: &Triangle<T>) -> String {
    format!(
        "<polygon points=\"{},{} {},{} {},{}\"/>",
        triangle.0.x, triangle.0.y, triangle.1.x, triangle.1.y, triangle.2.x, triangle.2.y
    )
}

fn triangle_to_svg_string<T: num_traits::Float + fmt::Display>(triangle: &Triangle<T>) -> String {
    format!(
        "M{} {}L{} {}L{} {}Z",
        triangle.0.x, triangle.0.y, triangle.1.x, triangle.1.y, triangle.2.x, triangle.2.y
    )
}

/** Lines */

impl<T: num_traits::Float + fmt::Display> ToSvg for MultiLineString<T> {
    fn to_svg(&self) -> String {
        multi_linestring_to_svg(self)
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for MultiLineString<T> {
    fn to_svg_string(&self) -> String {
        multi_linestring_to_svg_string(self)
    }
}

fn multi_linestring_to_svg<T: num_traits::Float + fmt::Display>(
    multi_line: &MultiLineString<T>,
) -> String {
    if multi_line.0.is_empty() {
        "".into()
    } else {
        multi_line
            .0
            .iter()
            .map(|l| linestring_to_svg(&l))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

fn multi_linestring_to_svg_string<T: num_traits::Float + fmt::Display>(
    multi_line: &MultiLineString<T>,
) -> String {
    if multi_line.0.is_empty() {
        "".into()
    } else {
        multi_line
            .0
            .iter()
            .map(|l| linestring_to_svg_string(&l))
            .collect::<Vec<String>>()
            .join("")
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvg for LineString<T> {
    fn to_svg(&self) -> String {
        linestring_to_svg(self)
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for LineString<T> {
    fn to_svg_string(&self) -> String {
        linestring_to_svg_string(self)
    }
}

fn linestring_to_svg<T: num_traits::Float + fmt::Display>(line: &LineString<T>) -> String {
    if line.0.is_empty() {
        "".into()
    } else {
        format!("<polyline points=\"{}\"/>", line_to_svg(line))
    }
}

fn linestring_to_svg_string<T: num_traits::Float + fmt::Display>(line: &LineString<T>) -> String {
    if line.0.is_empty() {
        "".into()
    } else {
        format!("M{}", line_to_svg_string(line))
    }
}

fn line_to_svg<T: num_traits::Float + fmt::Display>(line: &LineString<T>) -> String {
    line.0
        .iter()
        .map(|c| coord_to_svg_point(&c))
        .collect::<Vec<String>>()
        .join(" ")
}

fn line_to_svg_string<T: num_traits::Float + fmt::Display>(line: &LineString<T>) -> String {
    line.0
        .iter()
        .map(|c| coord_to_svg(&c))
        .collect::<Vec<String>>()
        .join("L")
}

/** Line */

impl<T: num_traits::Float + fmt::Display> ToSvg for Line<T> {
    fn to_svg(&self) -> String {
        single_line_to_svg(self)
    }
}

impl<T: num_traits::Float + fmt::Display> ToSvgString for Line<T> {
    fn to_svg_string(&self) -> String {
        single_line_to_svg_string(self)
    }
}

fn single_line_to_svg<T: num_traits::Float + fmt::Display>(line: &Line<T>) -> String {
    format!(
        "<line x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
        line.start.x, line.end.x, line.start.y, line.end.y
    )
}

fn single_line_to_svg_string<T: num_traits::Float + fmt::Display>(line: &Line<T>) -> String {
    format!(
        "M{} {}L{} {}",
        line.start.x, line.end.x, line.start.y, line.end.y
    )
}

/** Points */

fn coord_to_svg<T: num_traits::Float + fmt::Display>(coord: &Coordinate<T>) -> String {
    format!("{} {}", coord.x, coord.y)
}

fn coord_to_svg_point<T: num_traits::Float + fmt::Display>(coord: &Coordinate<T>) -> String {
    format!("{},{}", coord.x, coord.y)
}

/** Tests */

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{line_string, point, polygon};

    #[test]
    fn can_format_geom_collection() {
        let poly = Geometry::Polygon(polygon![
            (x: 1.0, y: 1.0),
            (x: 4.0, y: 1.0),
            (x: 4.0, y: 4.0),
            (x: 1.0, y: 4.0),
            (x: 1.0, y: 1.0),
        ]);
        let line = Geometry::LineString(line_string![
            (x: 11.0, y: 21.0),
            (x: 34.0, y: 21.0),
            (x: 24.0, y: 54.0),
            (x: 31.50, y: 34.0),
        ]);
        let gc = GeometryCollection(vec![line, poly]);
        let wkt_out = gc.to_svg();
        let expected = String::from(
            r#"<polyline points="11,21 34,21 24,54 31.5,34"/>
<path d="M1 1L4 1L4 4L1 4L1 1"/>"#,
        );
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_empty_geom_collection() {
        let gc = GeometryCollection(vec![] as Vec<Geometry<f64>>);
        let wkt_out = gc.to_svg();
        let expected = String::from("");
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_multi_polygon() {
        let poly1 = polygon![
            (x: 1.0, y: 1.0),
            (x: 4.0, y: 1.0),
            (x: 4.0, y: 4.0),
            (x: 1.0, y: 4.0),
            (x: 1.0, y: 1.0),
        ];
        let poly2 = polygon!(
        exterior: [
            (x: 0.0, y: 0.0),
            (x: 6.0, y: 0.0),
            (x: 6.0, y: 6.0),
            (x: 0.0, y: 6.0),
            (x: 0.0, y: 0.0),],
        interiors:[[
            (x: 1.0, y: 1.0),
            (x: 4.0, y: 1.0),
            (x: 4.0, y: 4.0),
            (x: 1.50, y: 4.0),
            (x: 1.0, y: 1.0),]
            ]
        );
        let mp = MultiPolygon(vec![poly1, poly2]);
        let wkt_out = mp.to_svg();
        let expected = String::from(
            r#"<path d="M1 1L4 1L4 4L1 4L1 1"/>
<path d="M0 0L6 0L6 6L0 6L0 0M1 1L4 1L4 4L1.5 4L1 1"/>"#,
        );
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_empty_multi_polygon() {
        let mp = MultiPolygon(vec![] as Vec<Polygon<f64>>);
        let wkt_out = mp.to_svg();
        let expected = String::from("");
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_polygon() {
        let poly = polygon![
            (x: 1.0, y: 1.0),
            (x: 40.0, y: 1.0),
            (x: 40.0, y: 40.0),
            (x: 1.0, y: 40.0),
            (x: 1.0, y: 1.0),
        ];
        let wkt_out = poly.to_svg();
        let expected = String::from(r#"<path d="M1 1L40 1L40 40L1 40L1 1"/>"#);
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_empty_polygon() {
        let poly: Polygon<f64> =
            Polygon::new(LineString::from(vec![] as Vec<Coordinate<f64>>), vec![]);
        let wkt_out = poly.to_svg();
        let expected = String::from("");
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_polygon_with_hole() {
        let poly = polygon!(
        exterior: [
            (x: 0.0, y: 0.0),
            (x: 0.0, y: 60.0),
            (x: 60.0, y: 60.0),
            (x: 60.0, y: 0.0),
            (x: 0.0, y: 0.0),],
        interiors:[[
            (x: 10.0, y: 10.0),
            (x: 40.0, y: 1.0),
            (x: 40.0, y: 40.0),
            (x: 10.50, y: 40.0),
            (x: 10.0, y: 10.0),]
            ]
        );
        let wkt_out = poly.to_svg();
        let expected =
            String::from(r#"<path d="M0 0L0 60L60 60L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10"/>"#);
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_multi_line_string() {
        let line1 = line_string![
            (x: 1.0, y: 1.0),
            (x: 4.0, y: 1.0),
            (x: 4.0, y: 4.0),
            (x: 1.50, y: 4.0),
        ];
        let line2 = line_string![
            (x: 11.0, y: 21.0),
            (x: 34.0, y: 21.0),
            (x: 24.0, y: 54.0),
            (x: 31.50, y: 34.0),
        ];
        let ml = MultiLineString(vec![line1, line2]);
        let wkt_out = ml.to_svg();
        let expected = String::from(
            r#"<polyline points="1,1 4,1 4,4 1.5,4"/>
<polyline points="11,21 34,21 24,54 31.5,34"/>"#,
        );
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_empty_multi_line_string() {
        let ml = MultiLineString(vec![] as Vec<LineString<f64>>);
        let wkt_out = ml.to_svg();
        let expected = String::from("");
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_line_string() {
        let line = line_string![
            (x: 1.0, y: 1.0),
            (x: 4.0, y: 1.0),
            (x: 4.0, y: 4.0),
            (x: 1.50, y: 4.0),
            (x: 1.0, y: 1.0),
        ];
        let wkt_out = line.to_svg();
        let expected = String::from(r#"<polyline points="1,1 4,1 4,4 1.5,4 1,1"/>"#);
        assert_eq!(wkt_out, expected);
    }

    #[test]
    fn can_format_empty_line_string() {
        let line = LineString::from(vec![] as Vec<Coordinate<f64>>);
        let wkt_out = line.to_svg();
        let expected = String::from("");
        assert_eq!(wkt_out, expected);
    }

    //TODO: add tests for Line, Triangle, and Rect
}
