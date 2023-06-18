use crate::feature::FeatureCollection;
use crate::mvt::MvtGeoFloatType;
use geo::area::Area;
use geo::euclidean_length::EuclideanLength;
use geo::simplify::Simplify;
use geo::{GeoFloat, Geometry, GeometryCollection};

#[cfg(test)]
mod tests {
    use crate::feature::{Feature, FeatureCollection, Simplifiable};
    use geo::{Coordinate, Geometry, Point};
    use std::collections::HashMap;

    #[test]
    fn remove_empty_keeps_points() {
        let mut collection = FeatureCollection::new();
        collection.push(Feature {
            geometry: Geometry::Point(Point(Coordinate::from((1.1, 0.0)))),
            properties: HashMap::new(),
        });

        collection.remove_empty(9999.0, 9999.0);
        assert_eq!(1, collection.len());
    }

    #[test]
    fn remove_empty_does_nothing_on_empty_collection() {
        let mut collection = FeatureCollection::new();
        assert_eq!(0, collection.len());

        collection.remove_empty(1.0, 1.0);

        assert_eq!(0, collection.len());
    }

    #[test]
    fn remove_empty_removes_empty_line() {
        let mut collection = FeatureCollection::new();
        let feature = Feature {
            geometry: Geometry::LineString(geo::LineString(vec![])),
            properties: HashMap::new(),
        };
        collection.push(feature);

        assert_eq!(1, collection.len());

        collection.remove_empty(1.0, 1.0);

        assert_eq!(0, collection.len());
    }

    #[test]
    fn remove_empty_removes_short_line() {
        let mut collection = FeatureCollection::new();
        let feature = Feature {
            geometry: Geometry::LineString(geo::LineString(vec![
                Coordinate { x: 1.1, y: 1.1 },
                Coordinate { x: 1.2, y: 1.1 },
            ])),
            properties: HashMap::new(),
        };
        collection.push(feature);

        collection.remove_empty(0.2, 0.0);

        assert_eq!(0, collection.len());
    }

    #[test]
    fn remove_empty_keeps_long_line() {
        let mut collection = FeatureCollection::new();
        let feature = Feature {
            geometry: Geometry::LineString(geo::LineString(vec![
                Coordinate { x: 1.1, y: 1.1 },
                Coordinate { x: 1.4, y: 1.1 },
            ])),
            properties: HashMap::new(),
        };
        collection.push(feature);

        collection.remove_empty(0.2, 0.0);

        assert_eq!(1, collection.len());
    }

    #[test]
    fn remove_empty_removes_small_polygon() {
        let mut collection = FeatureCollection::new();
        let feature = Feature {
            geometry: Geometry::Polygon(geo::Polygon::new(
                geo::LineString(vec![
                    Coordinate { x: 0.0, y: 1.0 },
                    Coordinate { x: 1.0, y: 1.0 },
                    Coordinate { x: 1.0, y: 0.0 },
                    Coordinate { x: 0.0, y: 0.0 },
                ]),
                vec![],
            )),
            properties: HashMap::new(),
        };
        collection.push(feature);

        collection.remove_empty(0.0, 0.9);

        assert_eq!(1, collection.len());

        collection.remove_empty(0.0, 1.1);

        assert_eq!(0, collection.len());
    }
}

pub trait Simplifiable<T> {
    /// using https://de.wikipedia.org/wiki/Douglas-Peucker-Algorithmus
    fn simplify(&mut self, epsilon: T);
    fn remove_empty(&mut self, line_limit: T, area_limit: T);
}

fn simplify_geo_collection<T: GeoFloat>(
    collection: &GeometryCollection<T>,
    epsilon: &T,
) -> GeometryCollection<T> {
    return collection
        .iter()
        .filter_map(|geo| simplify_geo(geo, epsilon))
        .collect();
}

fn simplify_geo<T: GeoFloat>(geometry: &Geometry<T>, epsilon: &T) -> Option<geo::Geometry<T>> {
    match geometry {
        Geometry::LineString(g) => Some(geo::Geometry::LineString(g.simplify(epsilon))),
        Geometry::Polygon(g) => Some(geo::Geometry::Polygon(g.simplify(epsilon))),
        Geometry::MultiLineString(g) => Some(geo::Geometry::MultiLineString(g.simplify(epsilon))),
        Geometry::MultiPolygon(g) => Some(geo::Geometry::MultiPolygon(g.simplify(epsilon))),
        Geometry::GeometryCollection(g) => Some(geo::Geometry::GeometryCollection(
            simplify_geo_collection(g, epsilon),
        )),
        _ => None,
    }
}

impl Simplifiable<MvtGeoFloatType> for FeatureCollection {
    fn simplify(&mut self, epsilon: MvtGeoFloatType) {
        self.0.iter_mut().for_each(|f| {
            let opt = simplify_geo(&f.geometry, &epsilon);

            if let Some(geo) = opt {
                f.geometry = geo;
            }
        });
    }

    // TL;DR:

    // The iterator returned by into_iter may yield any of T, &T or &mut T, depending on the context.
    // The iterator returned by iter will yield &T, by convention.
    // The iterator returned by iter_mut will yield &mut T, by convention.

    ///
    /// removes lines shorter/smaller than the limit
    ///
    /// here the Go implementation, see https://github.com/paulmach/orb/blob/master/encoding/mvt/simplify.go
    ///
    fn remove_empty(&mut self, line_limit: MvtGeoFloatType, area_limit: MvtGeoFloatType) {
        for i in (0..self.len()).rev() {
            let f = self.get(i).unwrap();
            let keep = match &f.geometry {
                Geometry::GeometryCollection(_) => todo!(),
                Geometry::Line(l) => l.euclidean_length() > line_limit,
                Geometry::LineString(ls) => ls.euclidean_length() > line_limit,
                Geometry::MultiLineString(_) => todo!(),
                Geometry::MultiPoint(_) => true,
                Geometry::MultiPolygon(pg) => pg.unsigned_area() > area_limit,
                Geometry::Point(_) => true,
                Geometry::Polygon(pg) => pg.unsigned_area() > area_limit,
                Geometry::Rect(_) => todo!(),
                Geometry::Triangle(_) => todo!(),
            };
            if !keep {
                self.swap_remove(i);
            }
        }
    }
}
