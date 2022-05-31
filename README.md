# rust-geo-svg

Functionality to convert between SVG and geo-types.

This is a fork of the original crate, updated for newer versions of `geo` and `geo_types`.

## SVG to Geometry

This package provides a functions to read a string containing an SVG element or `d` string and parse it to a geometry.

### svg_to_geometry(svg: &str)

  **Note** this function does not parse a full SVG string (e.g., `<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0L10 0L10 10L0 10Z"/></svg>`), it only parses the individual shape elements (e.g., `<path d="M0 0L10 0L10 10L0 10Z"/>`).  The following SVG elements are supported and produce the specified Geometry types:

* \<path\> &rarr; Geometry with the autodetected Geometry type
* \<polygon\> &rarr; Polygon
* \<polyline\> &rarr; LineString
* \<rect\> &rarr; Polygon
* \<line\> &rarr; Line

#### Examples

```rust
use geo_types::{ Polygon, polygon };
use geo_svg_io::geo_svg_reader::svg_to_geometry;

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
```

```rust
use geo_types::{ Polygon, polygon };
use geo_svg_io::geo_svg_reader::svg_to_geometry;

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
```

### svg_to_geometry_collection(svg: &str)

  **Note** this function does not parse a full SVG string (e.g., `<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0L10 0L10 10L0 10Z"/></svg>`), it only parses the individual shape elements (e.g., `<path d="M0 0L10 0L10 10L0 10Z"/>`).  The following SVG elements are supported and produce the specified Geometry types:

* \<path\> &rarr; GeometryCollection
* \<polygon\> &rarr; GeometryCollection with a single Polygon
* \<polyline\> &rarr; GeometryCollection with a single LineString
* \<rect\> &rarr; GeometryCollection with a single Polygon
* \<line\> &rarr; GeometryCollection with a single Line

#### Examples

```rust
use geo_types::{ Polygon, polygon };
use geo_svg_io::geo_svg_reader::svg_to_geometry_collection;

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
assert!(parsed_svg.is_ok());

// Unwrap the GeometryCollection result
let geom = parsed_svg.ok().unwrap();
assert_eq!(1, geom.0.len());

// Read the geometry as a Polygon
let pl = geom.0[0].clone().into_polygon();
assert_eq!(true, pl.is_some());
assert_eq!(poly, pl.unwrap());
```

```rust
use geo_types::{ Polygon, polygon };
use geo_svg_io::geo_svg_reader::svg_to_geometry;

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
assert!(parsed_svg.is_ok());

// Unwrap the GeometryCollection result
let geom = parsed_svg.ok().unwrap();
assert_eq!(1, geom.0.len());

// Read the geometry as a Polygon
let pl = geom.0[0].clone().into_polygon();
assert_eq!(true, pl.is_some());
assert_eq!(poly, pl.unwrap());
```

### svg_d_path_to_geometry(svg: &str)
A `<path>` element `d` string can be parsed directly into a Geometry by the `svg_d_path_to_geometry(svg: &str)` function.  The output will always be a GeometryCollection.

#### Examples

```rust
use geo_svg_io::geo_svg_reader::svg_d_path_to_geometry_collection;
use geo_types::polygon;

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
let parsed_svg = svg_d_path_to_geometry(&svg_string);
assert!(parsed_svg.is_ok());
let pl = parsed_svg.ok().unwrap().into_polygon();
assert!(pl.is_some());
assert_eq!(pl.unwrap(), poly);
```

### svg_d_path_to_geometry_collection(svg: &str)
A `<path>` element `d` string can be parsed directly into a GeometryCollection by the `svg_d_path_to_geometry_collection(svg: &str)` function.  The output will always be a GeometryCollection.

#### Examples

```rust
use geo_svg_io::geo_svg_reader::svg_d_path_to_geometry_collection;
use geo_types::polygon;

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
assert!(parsed_svg.is_ok());

// Unwrap the GeometryCollection result
let geom = parsed_svg.ok().unwrap();
assert_eq!(1, geom.0.len());

// Read the geometry as a Polygon
let pl = geom.0[0].clone().into_polygon();
assert_eq!(true, pl.is_some());
assert_eq!(pl.unwrap(), poly);
```

### Error handling
Both function return a Result which will either contain the parsed Geometry or an Error of the `SvgError` Enum. An error may result from passing an unsupported SVG element type, from an improperly formed SVG element, or from an inability to parse a `float` from the supplied string.

## Geometry to SVG
This package provides two traits for converting a Geometry to SVG.  **Note** that the parsing of curves in `<path>` `d`-strings is simplistic. It plots 100 points along the curve.  

**TODO** update this functionality to use a recursive function instead to create points until they are collinear (enough).

### ToSvg
Using `to_svg()` from any Geometry type with produce an SVG element of the simplest type possible:

* Polygon &rarr; \<path\>
* LineString &rarr; \<polyline\>
* Line &rarr; \<line\>
* Triangle &rarr; \<polygon\> with three points
* Rect &rarr; \<rect\> with `x`, `y`, `width`, and `height`

Complex Geometry types will return multiple SVG elements separated by `newline`s:

* GeometryCollection &rarr; `newline` separated SVG elements corresponding to the individual Geometries it contains
* MultiPolygon &rarr; `newline` separated <path> elements
* MultiLineString &rarr; `newline` separated <polyline> elements

#### Example

```rust
use geo_types::{ MultiPolygon, polygon };
use geo_svg_io::geo_svg_writer::ToSvg;

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
```

### ToSvgString
Using `to_svg_string()` from any Geometry type will produce an SVG `d` string for all the points of that geometry, which can be used in an SVG path element

#### Examples
```rust
use geo_types::{polygon, MultiPolygon};
use geo_svg_io::geo_svg_writer::ToSvgString;

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
let wkt_out = mp.to_svg_string();
let expected = String::from(
    "M1 1L4 1L4 4L1 4L1 1M0 0L6 0L6 6L0 6L0 0M1 1L4 1L4 4L1.5 4L1 1"
);
assert_eq!(wkt_out, expected);
```

# Similar projects

For a similar project that provides higher level functionality to build SVG's from Rust geo-types, see https://github.com/lelongg/geo-svg