extern crate geo_booleanop;
extern crate geo_normalized;
extern crate geo_types;

use flo_curves::bezier::{de_casteljau3, de_casteljau4};
use flo_curves::{Coord2, Coordinate2D};
use geo_types::{
    Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPolygon,
    Polygon, Rect,
};
use std::convert::From;
use std::fmt;
use svgtypes::{PathParser, PathSegment, PointsParser};
use xml::reader::{EventReader, XmlEvent};

pub enum SvgError {
    ParseError(std::num::ParseFloatError),
    SvgInvalidType(SvgUnsupportedGeometryTypeError),
    SvgGeomCollectionForGeometry(SvgGeometryCollectionForGeometryError),
    InvalidSvgError(InvalidSvgError),
}

impl From<std::num::ParseFloatError> for SvgError {
    fn from(error: std::num::ParseFloatError) -> Self {
        SvgError::ParseError(error)
    }
}

pub struct SvgUnsupportedGeometryTypeError;

// Implement std::fmt::Display for AppError
impl fmt::Display for SvgUnsupportedGeometryTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The SVG could not be parsed to a valid Geometry type") // user-facing output
    }
}

pub struct SvgGeometryCollectionForGeometryError;

// Implement std::fmt::Display for AppError
impl fmt::Display for SvgGeometryCollectionForGeometryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The SVG could only be parsed as a GEOMETRYCOLLECTION") // user-facing output
    }
}

// Implement std::fmt::Debug for AppError
impl fmt::Debug for SvgUnsupportedGeometryTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!()) // programmer-facing output
    }
}

pub struct InvalidSvgError;

// Implement std::fmt::Display for AppError
impl fmt::Display for InvalidSvgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The SVG input is invalid") // user-facing output
    }
}

// Implement std::fmt::Debug for AppError
impl fmt::Debug for InvalidSvgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!()) // programmer-facing output
    }
}

/// Returns a GeometryCollection parsed from the submitted SVG element
///
/// **Note** this function does not parse a full SVG string (e.g., `<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0L10 0L10 10L0 10Z"/></svg>`), it only parses the individual shape elements (e.g., `<path d="M0 0L10 0L10 10L0 10Z"/>`).  The following SVG elements are supported and produce the specified Geometry types:
///
/// * \<path\> &rarr; GeometryCollection
/// * \<polygon\> &rarr; GeometryCollection with a single Polygon
/// * \<polyline\> &rarr; GeometryCollection with a single LineString
/// * \<rect\> &rarr; GeometryCollection with a single Polygon
/// * \<line\> &rarr; GeometryCollection with a single Line
///
/// **Note** also that the current parsing of curves in a `<path>`is rather simple right now,
/// it just finds 100 points along the curve.
///
/// # Examples
///
/// Parsing a `<path>` element:
///
/// ```rust
/// use geo_types::{ Polygon, polygon };
/// use geo_svg_io::geo_svg_reader::svg_to_geometry_collection;
///
/// let poly: Polygon<f64> = polygon!(
///         exterior: [
///             (x: 0.0_f64, y: 0.0),
///             (x: 0.0, y: 60.0),
///             (x: 60.0, y: 60.0),
///             (x: 60.0, y: 0.0),
///             (x: 0.0, y: 0.0),],
///         interiors:[[
///             (x: 10.0, y: 10.0),
///             (x: 40.0, y: 1.0),
///             (x: 40.0, y: 40.0),
///             (x: 10.50, y: 40.0),
///             (x: 10.0, y: 10.0),]
///             ]
///         )
///         .into();
/// let svg_string =
///             String::from(r#"<path d="M0 0L0 60L60 60L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10"/>"#);
///
/// let parsed_svg = svg_to_geometry_collection(&svg_string);
/// assert_eq!(parsed_svg.is_ok(), true);
///
/// // Unwrap the GeometryCollection result
/// let geom = parsed_svg.ok().unwrap();
/// assert_eq!(1, geom.0.len());
///
/// // Read the geometry as a Polygon
/// let pl = geom.0[0].clone().into_polygon();
/// assert_eq!(true, pl.is_some());
/// assert_eq!(poly, pl.unwrap());
/// ```
///
/// Parsing a `<polygon>` element:
///
/// ```rust
/// use geo_types::{ Polygon, polygon };
/// use geo_svg_io::geo_svg_reader::svg_to_geometry_collection;
///
/// let poly: Polygon<f64> = polygon!(
///         exterior: [
///             (x: 0.0_f64, y: 0.0),
///             (x: 0.0, y: 60.0),
///             (x: 60.0, y: 60.0),
///             (x: 60.0, y: 0.0),
///             (x: 0.0, y: 0.0),],
///         interiors:[]
///         )
///         .into();
///
/// let svg_string = String::from(r#"<polygon points="0, 0 60, 0 60, 60 0, 60 0, 0"/>"#);
///
/// let parsed_svg = svg_to_geometry_collection(&svg_string);
/// assert_eq!(parsed_svg.is_ok(), true);
///
/// // Unwrap the GeometryCollection result
/// let geom = parsed_svg.ok().unwrap();
/// assert_eq!(1, geom.0.len());
///
/// // Read the geometry as a Polygon
/// let pl = geom.0[0].clone().into_polygon();
/// assert_eq!(true, pl.is_some());
/// assert_eq!(poly, pl.unwrap());
/// ```
///
pub fn svg_to_geometry_collection(svg: &str) -> Result<GeometryCollection<f64>, SvgError> {
    let parser = EventReader::new(svg.as_bytes());
    for e in parser {
        if let Ok(XmlEvent::StartElement {
            name, attributes, ..
        }) = e
        {
            // An SVG path element
            if name.local_name == "path" {
                for attr in attributes {
                    if attr.name.local_name == "d" {
                        let res = svg_d_path_to_geometry_collection(&attr.value)?;
                        return Ok(res);
                    }
                }
            }
            // An SVG polygon
            else if name.local_name == "polygon" {
                for attr in attributes {
                    if attr.name.local_name == "points" {
                        let res = svg_polygon_to_geometry(&attr.value)?;
                        return Ok(res.into());
                    }
                }
            }
            // An SVG polyline
            else if name.local_name == "polyline" {
                for attr in attributes {
                    if attr.name.local_name == "points" {
                        let res = svg_polyline_to_geometry(&attr.value)?;
                        return Ok(res.into());
                    }
                }
            }
            // An SVG rect
            else if name.local_name == "rect" {
                let mut x: Option<f64> = None;
                let mut y: Option<f64> = None;
                let mut width: Option<f64> = None;
                let mut height: Option<f64> = None;

                for attr in attributes {
                    if attr.name.local_name == "x" {
                        let x_val = attr.value.parse::<f64>()?;
                        x = Some(x_val);
                    } else if attr.name.local_name == "y" {
                        let y_val = attr.value.parse::<f64>()?;
                        y = Some(y_val);
                    } else if attr.name.local_name == "width" {
                        let width_val = attr.value.parse::<f64>()?;
                        width = Some(width_val);
                    } else if attr.name.local_name == "height" {
                        let height_val = attr.value.parse::<f64>()?;
                        height = Some(height_val);
                    }
                }

                if x.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }
                if y.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }
                if width.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }
                if height.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }
                let rect =
                    svg_rect_to_geometry(x.unwrap(), y.unwrap(), width.unwrap(), height.unwrap())?;

                return Ok(rect.into());
            }
            // An SVG line
            else if name.local_name == "line" {
                let mut start_x: Option<f64> = None;
                let mut start_y: Option<f64> = None;
                let mut end_x: Option<f64> = None;
                let mut end_y: Option<f64> = None;

                for attr in attributes {
                    if attr.name.local_name == "x1" {
                        let start_x_val = attr.value.parse::<f64>()?;
                        start_x = Some(start_x_val);
                    } else if attr.name.local_name == "y1" {
                        let start_y_val = attr.value.parse::<f64>()?;
                        start_y = Some(start_y_val);
                    } else if attr.name.local_name == "x2" {
                        let end_x_val = attr.value.parse::<f64>()?;
                        end_x = Some(end_x_val);
                    } else if attr.name.local_name == "y2" {
                        let end_y_val = attr.value.parse::<f64>()?;
                        end_y = Some(end_y_val);
                    }
                }

                if start_x.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }
                if start_y.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }
                if end_x.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }
                if end_y.is_none() {
                    return Err(SvgError::InvalidSvgError(InvalidSvgError));
                }

                return Ok(svg_line_to_geometry(
                    &start_x.unwrap(),
                    &start_y.unwrap(),
                    &end_x.unwrap(),
                    &end_y.unwrap(),
                )
                .into());
            }
        }
    }

    Err(SvgError::SvgInvalidType(SvgUnsupportedGeometryTypeError))
}

/// Returns a Geometry parsed from the submitted SVG element
///
/// **Note** this function does not parse a full SVG string (e.g., `<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0L10 0L10 10L0 10Z"/></svg>`), it only parses the individual shape elements (e.g., `<path d="M0 0L10 0L10 10L0 10Z"/>`).  The following SVG elements are supported and produce the specified Geometry types:
///
/// * \<path\> &rarr; Geometry with the autodetected Geometry type
/// * \<polygon\> &rarr; Polygon
/// * \<polyline\> &rarr; LineString
/// * \<rect\> &rarr; Polygon
/// * \<line\> &rarr; Line
///
/// **Note** also that the current parsing of curves in a `<path>`is rather simple right now,
/// it just finds 100 points along the curve.
///
/// # Examples
///
/// Parsing a `<path>` element:
///
/// ```rust
/// use geo_types::{ Polygon, polygon };
/// use geo_svg_io::geo_svg_reader::svg_to_geometry;
///
/// let poly: Polygon<f64> = polygon!(
///     exterior: [
///         (x: 0.0_f64, y: 0.0),
///         (x: 0.0, y: 60.0),
///         (x: 60.0, y: 60.0),
///         (x: 60.0, y: 0.0),
///         (x: 0.0, y: 0.0),],
///     interiors:[[
///         (x: 10.0, y: 10.0),
///         (x: 40.0, y: 1.0),
///         (x: 40.0, y: 40.0),
///         (x: 10.50, y: 40.0),
///         (x: 10.0, y: 10.0),]
///         ]
///     );
/// let svg_string =
///     String::from(r#"<path d="M0 0L0 60L60 60L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10"/>"#);
///
/// let parsed_svg = svg_to_geometry(&svg_string);
/// assert!(parsed_svg.is_ok());
/// let parsed_poly = parsed_svg.ok().unwrap().into_polygon();
/// assert!(parsed_poly.is_some());
/// assert_eq!(poly, parsed_poly.unwrap());
/// ```
///
/// Parsing a `<polygon>` element:
///
/// ```rust
/// use geo_types::{ Polygon, polygon };
/// use geo_svg_io::geo_svg_reader::svg_to_geometry;
///
/// let poly: Polygon<f64> = polygon!(
///     exterior: [
///         (x: 0.0_f64, y: 0.0),
///         (x: 0.0, y: 60.0),
///         (x: 60.0, y: 60.0),
///         (x: 60.0, y: 0.0),
///         (x: 0.0, y: 0.0),],
///     interiors:[]
///     );
/// let svg_string = String::from(r#"<polygon points="0, 0 60, 0 60, 60 0, 60 0, 0"/>"#);
///
/// let parsed_svg = svg_to_geometry(&svg_string);
/// assert!(parsed_svg.is_ok());
/// let parsed_poly = parsed_svg.ok().unwrap().into_polygon();
/// assert!(parsed_poly.is_some());
/// assert_eq!(poly, parsed_poly.unwrap());
/// ```
///
pub fn svg_to_geometry(svg: &str) -> Result<Geometry<f64>, SvgError> {
    let gc = svg_to_geometry_collection(svg)?;
    if gc.0.len() == 1 {
        return Ok(gc.0[0].clone());
    }
    Err(SvgError::SvgGeomCollectionForGeometry(
        SvgGeometryCollectionForGeometryError,
    ))
}

fn svg_polygon_to_geometry(point_string: &str) -> Result<Polygon<f64>, SvgError> {
    let points = PointsParser::from(point_string);
    let polygon = Polygon::new(
        LineString(
            points
                .map(|(x, y)| Coordinate { x, y })
                .collect::<Vec<Coordinate<f64>>>(),
        ),
        vec![],
    );

    if polygon.exterior().num_coords() == 0 {
        return Err(SvgError::InvalidSvgError(InvalidSvgError));
    }
    Ok(polygon)
}

fn svg_polyline_to_geometry(point_string: &str) -> Result<LineString<f64>, SvgError> {
    let points = PointsParser::from(point_string);
    let linestring = LineString(
        points
            .map(|(x, y)| Coordinate { x, y })
            .collect::<Vec<Coordinate<f64>>>(),
    );

    if linestring.num_coords() == 0 {
        return Err(SvgError::InvalidSvgError(InvalidSvgError));
    }
    Ok(linestring)
}

fn svg_rect_to_geometry(x: f64, y: f64, width: f64, height: f64) -> Result<Polygon<f64>, SvgError> {
    let max_x = x + width;
    let max_y = y + height;
    if x > max_x {
        return Err(SvgError::InvalidSvgError(InvalidSvgError));
    }
    if y > max_y {
        return Err(SvgError::InvalidSvgError(InvalidSvgError));
    }

    // geo_types::Rect is not part of the enum Geometry, so we cast it to Polygon upon return
    Ok(Polygon::from(Rect::new(
        Coordinate::<f64> { x, y },
        Coordinate::<f64> { x: max_x, y: max_y },
    )))
}

fn svg_line_to_geometry(start_x: &f64, start_y: &f64, end_x: &f64, end_y: &f64) -> Line<f64> {
    Line::new(
        Coordinate::<f64> {
            x: *start_x,
            y: *start_y,
        },
        Coordinate::<f64> {
            x: *end_x,
            y: *end_y,
        },
    )
}

/// Parses the `d`-string from an SVG `<path>` element into a GeometryCollection
///
/// **Note** that the current parsing of curves is rather simple right now, it just finds
/// 100 points along the curve.
///
/// # Examples
///
/// ```rust
/// use geo_svg_io::geo_svg_reader::svg_d_path_to_geometry_collection;
/// use geo_types::polygon;
///
/// let poly = polygon!(
///         exterior: [
///             (x: 0.0, y: 0.0),
///             (x: 0.0, y: 60.0),
///             (x: 60.0, y: 60.0),
///             (x: 60.0, y: 0.0),
///             (x: 0.0, y: 0.0),],
///         interiors:[[
///             (x: 10.0, y: 10.0),
///             (x: 40.0, y: 1.0),
///             (x: 40.0, y: 40.0),
///             (x: 10.50, y: 40.0),
///             (x: 10.0, y: 10.0),]
///             ]
///         );
///
/// let svg_string = String::from("M0 0l0 60l60 0L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10");
/// let parsed_svg = svg_d_path_to_geometry_collection(&svg_string);
/// assert_eq!(parsed_svg.is_ok(), true);
///
/// // Unwrap the GeometryCollection result
/// let geom = parsed_svg.ok().unwrap();
/// assert_eq!(1, geom.0.len());
///
/// // Read the geometry as a Polygon
/// let pl = geom.0[0].clone().into_polygon();
/// assert_eq!(true, pl.is_some());
/// assert_eq!(pl.unwrap(), poly);
/// ```
///
pub fn svg_d_path_to_geometry_collection(svg: &str) -> Result<GeometryCollection<f64>, SvgError> {
    // We will collect the separate paths (from M to M) into segments for parsing
    let mut path_segments = vec![] as Vec<Vec<Coordinate<f64>>>;
    let mut segment_count = 0;
    let mut first_segment = true;
    let zero_coord = Coordinate { x: 0_f64, y: 0_f64 }; // Default values to be added to relative coords
    let mut last_point: Option<Coordinate<f64>> = None; // Store last point for relative coordinates
    let mut last_control_point: Option<Coord2> = None; // Store last control point for S and T coordinates
    let p = PathParser::from(svg);
    for token in p {
        let t = token.unwrap();
        match t {
            PathSegment::MoveTo { .. } => {
                path_segments.push(vec![] as Vec<Coordinate<f64>>);
                if !first_segment {
                    segment_count += 1;
                } else {
                    first_segment = false;
                }
                let coord = Coordinate {
                    x: if t.is_relative() {
                        t.x().unwrap() + last_point.unwrap_or(zero_coord).x
                    } else {
                        t.x().unwrap()
                    },
                    y: if t.is_relative() {
                        t.y().unwrap() + last_point.unwrap_or(zero_coord).y
                    } else {
                        t.y().unwrap()
                    },
                };
                last_point = Some(coord);
                path_segments[segment_count].push(coord);
            }
            PathSegment::LineTo { .. } => {
                let coord = Coordinate {
                    x: if t.is_relative() {
                        t.x().unwrap() + last_point.unwrap_or(zero_coord).x
                    } else {
                        t.x().unwrap()
                    },
                    y: if t.is_relative() {
                        t.y().unwrap() + last_point.unwrap_or(zero_coord).y
                    } else {
                        t.y().unwrap()
                    },
                };
                last_point = Some(coord);
                path_segments[segment_count].push(coord);
            }
            PathSegment::HorizontalLineTo { .. } => {
                let coord = Coordinate {
                    x: if t.is_relative() {
                        t.x().unwrap() + last_point.unwrap_or(zero_coord).x
                    } else {
                        t.x().unwrap()
                    },
                    y: last_point.unwrap_or(zero_coord).y,
                };
                last_point = Some(coord);
                path_segments[segment_count].push(coord);
            }
            PathSegment::VerticalLineTo { .. } => {
                let coord = Coordinate {
                    x: last_point.unwrap_or(zero_coord).x,
                    y: if t.is_relative() {
                        t.y().unwrap() + last_point.unwrap_or(zero_coord).y
                    } else {
                        t.y().unwrap()
                    },
                };
                last_point = Some(coord);
                path_segments[segment_count].push(coord);
            }
            PathSegment::CurveTo {
                x,
                x1,
                x2,
                y,
                y1,
                y2,
                abs,
            } => {
                let last = last_point.unwrap_or(zero_coord);
                let start_point = calculate_svg_coord2(last.x, last.y, last, true);
                let control_1 = calculate_svg_coord2(x1, y1, last, abs);
                let control_2 = calculate_svg_coord2(x2, y2, last, abs);
                last_control_point = Some(control_2);
                let end_point = calculate_svg_coord2(x, y, last, abs);
                let end = Coordinate {
                    x: end_point.x(),
                    y: end_point.y(),
                };
                last_point = Some(end);
                // TODO: it is not great to just pick an arbitrary number of points along the curve
                // update this to use a recursive function instead to create more points until
                // they are collinear (enough)
                for x in 1..100 {
                    let arc_point = de_casteljau4(
                        x as f64 / 100_f64,
                        start_point,
                        control_1,
                        control_2,
                        end_point,
                    );
                    path_segments[segment_count].push(Coordinate {
                        x: arc_point.x(),
                        y: arc_point.y(),
                    });
                }
                path_segments[segment_count].push(end);
            }
            PathSegment::SmoothCurveTo { x2, x, y2, y, abs } => {
                let last = last_point.unwrap_or(zero_coord);
                let start_point = calculate_svg_coord2(last.x, last.y, last, true);
                let control_1 = reflect_point(last, last_control_point.unwrap_or(Coord2(0., 0.)));
                let control_2 = calculate_svg_coord2(x2, y2, last, abs);
                last_control_point = Some(control_2);
                let end_point = calculate_svg_coord2(x, y, last, abs);
                let end = Coordinate {
                    x: end_point.x(),
                    y: end_point.y(),
                };
                last_point = Some(end);
                // TODO: it is not great to just pick an arbitrary number of points along the curve
                // update this to use a recursive function instead to create more points until
                // they are collinear (enough)
                for x in 1..100 {
                    let arc_point = de_casteljau4(
                        x as f64 / 100_f64,
                        start_point,
                        control_1,
                        control_2,
                        end_point,
                    );
                    path_segments[segment_count].push(Coordinate {
                        x: arc_point.x(),
                        y: arc_point.y(),
                    });
                }
                path_segments[segment_count].push(end);
            }
            PathSegment::Quadratic { x1, x, y1, y, abs } => {
                let last = last_point.unwrap_or(zero_coord);
                let start_point = calculate_svg_coord2(last.x, last.y, last, true);
                let control_1 = calculate_svg_coord2(x1, y1, last, abs);
                last_control_point = Some(control_1);
                let end_point = calculate_svg_coord2(x, y, last, abs);
                let end = Coordinate {
                    x: end_point.x(),
                    y: end_point.y(),
                };
                last_point = Some(end);
                // TODO: it is not great to just pick an arbitrary number of points along the curve
                // update this to use a recursive function instead to create more points until
                // they are collinear (enough)
                for x in 1..100 {
                    let arc_point =
                        de_casteljau3(x as f64 / 100_f64, start_point, control_1, end_point);
                    path_segments[segment_count].push(Coordinate {
                        x: arc_point.x(),
                        y: arc_point.y(),
                    });
                }
                path_segments[segment_count].push(end);
            }
            PathSegment::SmoothQuadratic { x, y, abs } => {
                let last = last_point.unwrap_or(zero_coord);
                let start_point = calculate_svg_coord2(last.x, last.y, last, true);
                let control_1 = reflect_point(last, last_control_point.unwrap_or(Coord2(0., 0.)));
                last_control_point = Some(control_1);
                let end_point = calculate_svg_coord2(x, y, last, abs);
                let end = Coordinate {
                    x: end_point.x(),
                    y: end_point.y(),
                };
                last_point = Some(end);
                // TODO: it is not great to just pick an arbitrary number of points along the curve
                // update this to use a recursive function instead to create more points until
                // they are collinear (enough)
                for x in 1..100 {
                    let arc_point =
                        de_casteljau3(x as f64 / 100_f64, start_point, control_1, end_point);
                    path_segments[segment_count].push(Coordinate {
                        x: arc_point.x(),
                        y: arc_point.y(),
                    });
                }
                path_segments[segment_count].push(end);
            }
            // TODO: PathSegment::EllipticalArc
            PathSegment::ClosePath { .. } => {
                let coord = Coordinate {
                    x: path_segments[segment_count][0].x,
                    y: path_segments[segment_count][0].y,
                };
                last_point = Some(coord);
                path_segments[segment_count].push(coord);
            }
            _ => last_point = None,
        }
    }
    if path_segments.is_empty() {
        return Err(SvgError::InvalidSvgError(InvalidSvgError));
    }
    Ok(parse_path_segments_to_geom(&path_segments))
}

/// Parses the `d`-string from an SVG `<path>` element into a single Geometry
///
/// **Note** that the current parsing of curves is rather simple right now, it just finds
/// 100 points along the curve.
///
/// # Examples
///
/// ```rust
/// use geo_svg_io::geo_svg_reader::svg_d_path_to_geometry;
/// use geo_types::polygon;
///
/// let poly = polygon!(
///         exterior: [
///             (x: 0.0, y: 0.0),
///             (x: 0.0, y: 60.0),
///             (x: 60.0, y: 60.0),
///             (x: 60.0, y: 0.0),
///             (x: 0.0, y: 0.0),],
///         interiors:[[
///             (x: 10.0, y: 10.0),
///             (x: 40.0, y: 1.0),
///             (x: 40.0, y: 40.0),
///             (x: 10.50, y: 40.0),
///             (x: 10.0, y: 10.0),]
///             ]
///         );
///
/// let svg_string = String::from("M0 0l0 60l60 0L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10");
/// let parsed_svg = svg_d_path_to_geometry(&svg_string);
/// assert!(parsed_svg.is_ok());
/// let pl = parsed_svg.ok().unwrap().into_polygon();
/// assert!(pl.is_some());
/// assert_eq!(pl.unwrap(), poly);
/// ```
///
pub fn svg_d_path_to_geometry(svg: &str) -> Result<Geometry<f64>, SvgError> {
    let gc = svg_d_path_to_geometry_collection(svg)?;
    if gc.0.len() == 1 {
        return Ok(gc.0[0].clone());
    }
    Err(SvgError::SvgGeomCollectionForGeometry(
        SvgGeometryCollectionForGeometryError,
    ))
}

fn calculate_svg_coord2(x: f64, y: f64, last: Coordinate<f64>, abs: bool) -> Coord2 {
    Coord2(
        if abs { x } else { last.x + x },
        if abs { y } else { last.y + y },
    )
}

fn reflect_point(orig: Coordinate<f64>, pr: Coord2) -> Coord2 {
    let x_step = pr.x() - orig.x;
    let y_step = pr.y() - orig.y;

    Coord2(orig.x - x_step, orig.y - y_step)
}

fn parse_path_segments_to_geom(paths: &Vec<Vec<Coordinate<f64>>>) -> GeometryCollection<f64> {
    let mut lines = vec![] as Vec<Line<f64>>;
    let mut line_strings = vec![] as Vec<LineString<f64>>;
    let mut poly_line_strings = vec![] as Vec<LineString<f64>>;
    let mut polygons: MultiPolygon<f64> = (vec![] as Vec<Polygon<f64>>).into();

    for path in paths {
        let length = path.len();
        if length == 0 {
            continue;
        } else if length == 2 {
            lines.push(Line::new(path[0], path[1]));
        } else if !path.first().unwrap().eq(path.last().unwrap()) {
            line_strings.push(path.clone().into());
        } else {
            poly_line_strings.push(path.clone().into());
        }
    }

    if !poly_line_strings.is_empty() {
        if poly_line_strings.len() == 1 {
            polygons = Polygon::new(poly_line_strings[0].clone(), vec![]).into();
        } else {
            polygons = parse_polygon_rings_to_geom(&poly_line_strings);
        }
    }

    let number_of_geom_types = !lines.is_empty() as i32
        + !line_strings.is_empty() as i32
        + !poly_line_strings.is_empty() as i32;

    let mut geom_collection = vec![] as Vec<Geometry<f64>>;
    if !lines.is_empty() {
        let return_lines = map_lines_to_geometry(&lines);
        if number_of_geom_types == 1 {
            return GeometryCollection(vec![return_lines]);
        } else {
            geom_collection.push(return_lines);
        }
    } else if !line_strings.is_empty() {
        let return_line_strings = map_line_strings_to_geometry(&line_strings);
        if number_of_geom_types == 1 {
            return GeometryCollection(vec![return_line_strings]);
        } else {
            geom_collection.push(return_line_strings);
        }
    } else if !polygons.0.is_empty() {
        let return_polygons = map_polygons_to_geometry(polygons);
        if number_of_geom_types == 1 {
            return GeometryCollection(vec![return_polygons]);
        } else {
            geom_collection.push(return_polygons);
        }
    }

    GeometryCollection(geom_collection)
}

fn parse_polygon_rings_to_geom(rings: &Vec<LineString<f64>>) -> MultiPolygon<f64> {
    // Early return for empty vector
    if rings.len() == 0 {
        return (vec![] as Vec<Polygon<f64>>).into();
    }

    let mut ring_iter = rings.iter();
    let mut result_poly = MultiPolygon(vec![Polygon::new(
        ring_iter.next().unwrap().clone(),
        vec![],
    )]);

    result_poly
}

fn map_lines_to_geometry(lines: &Vec<Line<f64>>) -> Geometry<f64> {
    if lines.len() == 1 {
        lines[0].into()
    } else {
        let multi_line: MultiLineString<f64> = lines
            .iter()
            .map(|x| LineString(vec![x.start, x.end]))
            .collect();
        multi_line.into()
    }
}

fn map_line_strings_to_geometry(line_strings: &Vec<LineString<f64>>) -> Geometry<f64> {
    if line_strings.len() == 1 {
        line_strings[0].clone().into()
    } else {
        let multi_line: MultiLineString<f64> = MultiLineString(line_strings.to_vec());
        multi_line.into()
    }
}

fn map_polygons_to_geometry(polys: MultiPolygon<f64>) -> Geometry<f64> {
    if polys.0.len() == 1 {
        polys.0[0].clone().into()
    } else {
        polys.into()
    }
}

/** Tests */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo_svg_writer::ToSvg;
    use geo_types::{line_string, polygon};

    #[test]
    fn can_convert_svg_path() {
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
        let svg_string = String::from("M0 0l0 60l60 0L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10");
        let parsed_svg = svg_d_path_to_geometry_collection(&svg_string);
        assert_eq!(parsed_svg.is_ok(), true);
        let geom = parsed_svg.ok().unwrap();
        assert_eq!(1, geom.0.len());
        let pl = geom.0[0].clone().into_polygon();
        assert_eq!(true, pl.is_some());
        assert_eq!(pl.unwrap(), poly);
    }

    #[test]
    fn can_convert_svg_path_test() {
        let poly: Polygon<f64> = polygon!(
        exterior: [
            (x: 0.0_f64, y: 0.0),
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
        )
        .into();
        let svg_string =
            String::from(r#"<path d="M0 0L0 60L60 60L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10"/>"#);
        let parsed_svg = svg_to_geometry_collection(&svg_string);
        assert_eq!(parsed_svg.is_ok(), true);
        let geom = parsed_svg.ok().unwrap();
        assert_eq!(1, geom.0.len());
        let pl = geom.0[0].clone().into_polygon();
        assert_eq!(true, pl.is_some());
        assert_eq!(poly, pl.unwrap());
    }

    #[test]
    fn can_convert_svg_h_v_path_test() {
        let poly: Polygon<f64> = polygon!(
        exterior: [
            (x: 0.0_f64, y: 0.0),
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
        )
        .into();
        let svg_string =
            String::from(r#"<path d="M0 0v60h60v-60h-60M10 10L40 1L40 40L10.5 40L10 10"/>"#);
        let parsed_svg = svg_to_geometry_collection(&svg_string);
        assert_eq!(parsed_svg.is_ok(), true);
        let geom = parsed_svg.ok().unwrap();
        assert_eq!(1, geom.0.len());
        let pl = geom.0[0].clone().into_polygon();
        assert_eq!(true, pl.is_some());
        assert_eq!(poly, pl.unwrap());
    }

    #[test]
    fn can_convert_svg_c_s_path_test() {
        let solution = String::from(
            r#"<path d="M0 0L0.00895 0.89401L0.0356 1.7760799999999999L0.07964999999999998 2.6462699999999995L0.14079999999999998 3.5046399999999998L0.21875000000000003 4.35125L0.3132 5.186159999999999L0.42385000000000006 6.00943L0.5504 6.8211200000000005L0.6925499999999999 7.621289999999999L0.8500000000000001 8.41L1.02245 9.18731L1.2095999999999998 9.95328L1.4111500000000001 10.70797L1.6268000000000002 11.451440000000002L1.8562499999999997 12.18375L2.0992 12.90496L2.3553500000000005 13.61513L2.6244 14.314320000000002L2.90605 15.002590000000001L3.2 15.680000000000001L3.5059499999999995 16.34661L3.8236 17.002480000000002L4.15265 17.64767L4.492799999999999 18.282239999999998L4.84375 18.90625L5.2052000000000005 19.51976L5.576850000000001 20.122830000000004L5.9584 20.715519999999998L6.349549999999999 21.297889999999995L6.749999999999999 21.869999999999997L7.15945 22.43191L7.5776 22.98368L8.00415 23.525369999999995L8.4388 24.05704L8.88125 24.578750000000003L9.331199999999999 25.090559999999996L9.788350000000001 25.592530000000004L10.2524 26.084719999999997L10.723050000000002 26.567190000000004L11.200000000000001 27.04L11.68295 27.503210000000003L12.1716 27.956880000000005L12.66565 28.401070000000004L13.164800000000001 28.835840000000005L13.668750000000003 29.261250000000004L14.177200000000003 29.677360000000004L14.689849999999996 30.084229999999998L15.206399999999997 30.481919999999995L15.726550000000001 30.870490000000004L16.25 31.25L16.776449999999997 31.620509999999996L17.305600000000002 31.98208L17.83715 32.334770000000006L18.370800000000003 32.67864L18.906250000000004 33.013749999999995L19.443200000000004 33.34016L19.98135 33.65792999999999L20.520399999999995 33.967119999999994L21.060049999999997 34.26779L21.599999999999998 34.56L22.139950000000002 34.84381L22.6796 35.11928L23.21865 35.38647L23.756800000000002 35.64544L24.293750000000003 35.896249999999995L24.829200000000004 36.138960000000004L25.36285 36.373630000000006L25.8944 36.60032L26.42355 36.81909L26.95 37.03L27.47345 37.233109999999996L27.993599999999994 37.42847999999999L28.510149999999996 37.61617L29.022800000000007 37.796240000000004L29.53125 37.96875L30.035199999999996 38.13376L30.534350000000007 38.29133L31.028400000000005 38.441520000000004L31.517049999999998 38.58439L32 38.72L32.47695 38.84841L32.947599999999994 38.969680000000004L33.41164999999999 39.08387L33.8688 39.19104L34.31875 39.29125L34.7612 39.38456L35.19584999999999 39.47103L35.622400000000006 39.550720000000005L36.040549999999996 39.623689999999996L36.449999999999996 39.69L36.850449999999995 39.74971L37.241600000000005 39.80288L37.62315000000001 39.84957L37.99479999999999 39.88984L38.356249999999996 39.923750000000005L38.70719999999999 39.95136L39.04734999999999 39.97273L39.376400000000004 39.98792L39.69405 39.99699L40 40L40.29702 40.00596L40.588159999999995 40.023680000000006L40.87353999999999 40.052919999999986L41.153279999999995 40.093439999999994L41.427499999999995 40.14499999999999L41.696319999999986 40.207359999999994L41.95985999999999 40.28027999999999L42.21824000000001 40.363520000000015L42.47158 40.45684L42.72 40.56000000000001L42.963620000000006 40.672760000000004L43.20256 40.79488L43.43693999999999 40.92611999999999L43.66688 41.06624L43.8925 41.215L44.11392 41.37216L44.33126 41.537479999999995L44.54464000000001 41.71072L44.754180000000005 41.89164000000001L44.96 42.08L45.162220000000005 42.275560000000006L45.360960000000006 42.478080000000006L45.55634 42.68732L45.74848 42.90304L45.9375 43.125L46.12352 43.35296L46.30666 43.58668L46.48704 43.825919999999996L46.66477999999999 44.07043999999999L46.83999999999999 44.31999999999999L47.01281999999999 44.57436L47.18335999999999 44.83327999999999L47.35173999999999 45.09651999999999L47.51807999999999 45.363839999999996L47.682500000000005 45.635L47.845119999999994 45.909760000000006L48.006060000000005 46.18788L48.165440000000004 46.469120000000004L48.32338 46.753240000000005L48.480000000000004 47.040000000000006L48.63542000000001 47.32916L48.78976000000001 47.62048000000001L48.94314000000001 47.91372000000001L49.09568000000001 48.20864L49.2475 48.505L49.398720000000004 48.80256000000001L49.549459999999996 49.10108000000001L49.69984 49.400319999999994L49.84998 49.70004L50 50L50.15002 50.29996L50.300160000000005 50.59968L50.45054 50.89892L50.60128 51.19744L50.7525 51.495000000000005L50.90432 51.791360000000005L51.05686 52.08628L51.21024 52.37951999999999L51.364580000000004 52.67084L51.519999999999996 52.96L51.67662 53.24676L51.834559999999996 53.53088L51.993939999999995 53.81211999999999L52.15488 54.09024L52.3175 54.364999999999995L52.48192 54.636160000000004L52.64826000000001 54.90348L52.81664000000001 55.16672L52.98718000000001 55.42564L53.160000000000004 55.68000000000001L53.33521999999999 55.929559999999995L53.51295999999999 56.17408L53.69334 56.41332L53.87648 56.64704L54.0625 56.875L54.25152 57.09696L54.44366 57.31268000000001L54.63904000000001 57.52192L54.83778000000001 57.72444L55.04 57.92L55.245819999999995 58.10836L55.45536 58.28928L55.66873999999999 58.46252L55.88608 58.62783999999999L56.1075 58.785L56.33312 58.93376000000001L56.56306000000001 59.07388000000001L56.79744 59.20512L57.036379999999994 59.327239999999996L57.28 59.44L57.528420000000004 59.54316L57.781760000000006 59.63648L58.04014000000001 59.719719999999995L58.30368 59.792640000000006L58.5725 59.855000000000004L58.84672 59.90655999999999L59.12645999999999 59.94708L59.41184 59.976319999999994L59.70298 59.99404L60 60L60 0L0 0M10 10L20 10L20 20L10 20L10 10"/>"#,
        );
        let svg_string = String::from(
            r#"<path d="M0 0C0 30 30 40 40 40S50 60 60 60L60 0ZM10 10L20 10L20 20L10 20L10 10" />"#,
        );
        let parsed_svg = svg_to_geometry_collection(&svg_string);
        assert_eq!(true, parsed_svg.is_ok());
        let svg = parsed_svg.ok().unwrap().to_svg();
        assert_eq!(solution, svg);
    }

    #[test]
    fn can_convert_svg_q_t_path_test() {
        let solution = String::from(
            r#"<path d="M0 0L0.598 0.796L1.192 1.584L1.7819999999999998 2.364L2.368 3.136L2.95 3.9L3.5279999999999996 4.655999999999999L4.102 5.404L4.672000000000001 6.144000000000001L5.2379999999999995 6.8759999999999994L5.800000000000001 7.6L6.3580000000000005 8.316L6.911999999999999 9.024000000000001L7.462 9.724L8.008000000000001 10.416L8.549999999999999 11.1L9.088000000000001 11.776L9.622 12.444L10 12.914716981132074L10 10L20 10L20 20L15.858156028368795 20L16.2 20.4L16.678 20.956L17.152 21.503999999999998L17.622 22.044L18.088 22.576L18.55 23.1L19.007999999999996 23.616L19.462 24.124000000000002L19.912 24.624L20.358000000000004 25.116L20.8 25.6L21.238 26.076L21.672 26.544000000000004L22.102 27.003999999999998L22.528000000000002 27.456000000000003L22.950000000000003 27.9L23.368000000000006 28.336000000000006L23.781999999999996 28.763999999999996L24.191999999999997 29.183999999999997L24.598000000000003 29.596000000000004L25 30L25.397999999999996 30.395999999999997L25.792 30.784L26.182000000000002 31.164L26.568 31.536L26.950000000000003 31.9L27.328000000000003 32.256L27.701999999999998 32.604L28.071999999999996 32.944L28.438 33.275999999999996L28.799999999999997 33.6L29.158 33.916L29.512000000000004 34.224000000000004L29.862 34.524L30.208 34.816L30.55 35.1L30.888 35.376000000000005L31.222 35.644L31.552 35.904L31.878 36.156L32.2 36.400000000000006L32.518 36.635999999999996L32.831999999999994 36.864L33.141999999999996 37.084L33.44800000000001 37.296L33.75 37.5L34.047999999999995 37.696L34.342000000000006 37.884L34.632000000000005 38.064L34.918 38.236000000000004L35.2 38.4L35.478 38.556000000000004L35.751999999999995 38.704L36.02199999999999 38.843999999999994L36.288000000000004 38.976L36.550000000000004 39.1L36.808 39.216L37.062 39.324L37.312000000000005 39.42400000000001L37.558 39.516L37.8 39.6L38.038 39.675999999999995L38.272000000000006 39.744L38.502 39.804L38.727999999999994 39.855999999999995L38.95 39.9L39.168 39.936L39.38199999999999 39.964L39.592000000000006 39.984L39.797999999999995 39.996L40 40L40.199999999999996 40.002L40.4 40.008L40.599999999999994 40.017999999999994L40.8 40.032L41 40.05L41.199999999999996 40.071999999999996L41.39999999999999 40.09799999999999L41.60000000000001 40.128000000000014L41.8 40.162L42 40.2L42.2 40.242000000000004L42.4 40.288000000000004L42.599999999999994 40.337999999999994L42.8 40.391999999999996L43 40.45L43.2 40.512L43.4 40.577999999999996L43.6 40.648L43.80000000000001 40.72200000000001L44 40.8L44.2 40.882000000000005L44.400000000000006 40.968L44.599999999999994 41.058L44.8 41.152L45 41.25L45.2 41.352000000000004L45.400000000000006 41.458L45.599999999999994 41.568L45.8 41.681999999999995L46 41.8L46.199999999999996 41.922L46.39999999999999 42.047999999999995L46.599999999999994 42.178L46.8 42.312L47 42.45L47.199999999999996 42.592L47.400000000000006 42.738L47.599999999999994 42.888000000000005L47.800000000000004 43.042L48 43.2L48.2 43.362L48.400000000000006 43.528000000000006L48.60000000000001 43.69800000000001L48.80000000000001 43.872L49 44.05L49.2 44.232L49.400000000000006 44.418000000000006L49.599999999999994 44.608L49.8 44.80199999999999L50 45L50.2 45.202L50.400000000000006 45.408L50.599999999999994 45.617999999999995L50.8 45.83200000000001L51 46.05L51.199999999999996 46.272000000000006L51.400000000000006 46.498000000000005L51.599999999999994 46.727999999999994L51.8 46.962L52 47.2L52.2 47.44200000000001L52.400000000000006 47.688L52.6 47.938L52.8 48.192L53 48.45L53.199999999999996 48.712L53.400000000000006 48.97800000000001L53.6 49.248L53.80000000000001 49.52199999999999L54 49.8L54.2 50.081999999999994L54.4 50.368L54.599999999999994 50.658L54.8 50.952L55 51.25L55.2 51.55200000000001L55.400000000000006 51.858000000000004L55.6 52.168L55.800000000000004 52.482000000000006L56 52.800000000000004L56.199999999999996 53.122L56.4 53.448L56.599999999999994 53.778000000000006L56.8 54.111999999999995L57 54.449999999999996L57.2 54.792L57.400000000000006 55.138000000000005L57.6 55.48799999999999L57.8 55.842L58 56.2L58.2 56.562000000000005L58.400000000000006 56.928L58.60000000000001 57.298L58.800000000000004 57.672L59 58.05L59.199999999999996 58.431999999999995L59.39999999999999 58.818L59.6 59.208L59.8 59.602L60 60L60 0L0 0"/>
<path d="M10 12.914716981132074L10 20L15.858156028368795 20L15.717999999999998 19.836L15.232 19.264000000000003L14.742 18.684L14.248000000000001 18.096L13.75 17.5L13.248 16.896L12.742 16.284000000000002L12.232 15.664000000000001L11.718 15.036000000000001L11.200000000000001 14.4L10.678 13.756L10.152000000000001 13.104L10 12.914716981132074"/>"#,
        );
        let svg_string = String::from(
            r#"<path d="M0 0Q30 40 40 40T60 60L60 0ZM10 10L20 10L20 20L10 20L10 10" />"#,
        );
        let parsed_svg = svg_to_geometry_collection(&svg_string);
        assert_eq!(true, parsed_svg.is_ok());
        let svg = parsed_svg.ok().unwrap().to_svg();
        assert_eq!(solution, svg);
    }

    #[test]
    fn can_convert_svg_polygon_test() {
        let poly: Polygon<f64> = polygon!(
        exterior: [
            (x: 0.0_f64, y: 0.0),
            (x: 0.0, y: 60.0),
            (x: 60.0, y: 60.0),
            (x: 60.0, y: 0.0),
            (x: 0.0, y: 0.0),],
        interiors:[]
        )
        .into();
        let svg_string = String::from(r#"<polygon points="0, 0 60, 0 60, 60 0, 60 0, 0"/>"#);
        let parsed_svg = svg_to_geometry_collection(&svg_string);
        assert_eq!(parsed_svg.is_ok(), true);
        let geom = parsed_svg.ok().unwrap();
        assert_eq!(1, geom.0.len());
        let pl = geom.0[0].clone().into_polygon();
        assert_eq!(true, pl.is_some());
        assert_eq!(poly, pl.unwrap());
    }

    #[test]
    fn can_convert_svg_polyline_test() {
        let line: LineString<f64> = line_string![
            (x: 0.0_f64, y: 0.0),
            (x: 0.0, y: 60.0),
            (x: 60.0, y: 60.0),
            (x: 60.0, y: 0.0),]
        .into();
        let svg_string = String::from(r#"<polyline points="0, 0 0, 60 60, 60 60, 0"/>"#);
        let parsed_svg = svg_to_geometry_collection(&svg_string);
        assert_eq!(parsed_svg.is_ok(), true);
        let geom = parsed_svg.ok().unwrap();
        assert_eq!(1, geom.0.len());
        let pl = geom.0[0].clone().into_line_string();
        assert_eq!(true, pl.is_some());
        assert_eq!(line, pl.unwrap());
    }

    #[test]
    fn can_convert_svg_rect_test() {
        let poly: Polygon<f64> = polygon!(
        exterior: [
            (x: 0.0_f64, y: 0.0),
            (x: 0.0, y: 60.0),
            (x: 60.0, y: 60.0),
            (x: 60.0, y: 0.0),
            (x: 0.0, y: 0.0),],
        interiors:[]
        )
        .into();
        let svg_string = String::from(r#"<rect x="0" y="0" width="60" height="60"/>"#);
        let parsed_svg = svg_to_geometry_collection(&svg_string);
        assert_eq!(parsed_svg.is_ok(), true);
        let geom = parsed_svg.ok().unwrap();
        assert_eq!(1, geom.0.len());
        let pl = geom.0[0].clone().into_polygon();
        assert_eq!(true, pl.is_some());
        assert_eq!(poly, pl.unwrap());
    }

    #[test]
    fn can_convert_svg_path_to_single_geom() {
        let poly: Polygon<f64> = polygon!(
            exterior: [
                (x: 0.0_f64, y: 0.0),
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
        let svg_string =
            String::from(r#"<path d="M0 0L0 60L60 60L60 0L0 0M10 10L40 1L40 40L10.5 40L10 10"/>"#);

        let parsed_svg = svg_to_geometry(&svg_string);
        assert!(parsed_svg.is_ok());
        let parsed_poly = parsed_svg.ok().unwrap().into_polygon();
        assert!(parsed_poly.is_some());
        assert_eq!(poly, parsed_poly.unwrap());
    }

    #[test]
    fn can_convert_svg_polygon_to_single_geom() {
        let poly: Polygon<f64> = polygon!(
            exterior: [
                (x: 0.0_f64, y: 0.0),
                (x: 0.0, y: 60.0),
                (x: 60.0, y: 60.0),
                (x: 60.0, y: 0.0),
                (x: 0.0, y: 0.0),],
            interiors:[]
            );
        let svg_string = String::from(r#"<polygon points="0, 0 60, 0 60, 60 0, 60 0, 0"/>"#);

        let parsed_svg = svg_to_geometry(&svg_string);
        assert!(parsed_svg.is_ok());
        let parsed_poly = parsed_svg.ok().unwrap().into_polygon();
        assert!(parsed_poly.is_some());
        assert_eq!(poly, parsed_poly.unwrap());
    }
}
