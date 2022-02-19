use geo::{Coordinate, GeoFloat, Geometry, Line, MultiPolygon, Point, Polygon, Rect};
use geo::algorithm::line_intersection::{line_intersection, LineIntersection};
use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::algorithm::map_coords::MapCoords;
use geo_clipper::{Clipper};
use num_traits::ToPrimitive;

#[cfg(test)]
mod tests {
    use anyhow::Error;
    use rstest::rstest;
    use geo::{Coordinate, Line, LineString, MultiPolygon, Point, Polygon, Rect};
    use geo::algorithm::translate::Translate;
    use crate::mvt::clip_feature::{Clip, ClipFloat};

    fn multipoly_to_rect<T: ClipFloat>(multi_polygon: &MultiPolygon<T>, idx: usize) -> anyhow::Result<Rect<T>> {
        let first_poly: &Polygon<T> = multi_polygon.0.get(idx).ok_or(Error::msg("multipolygon is empty"))?;

        if first_poly.interiors().len() > 0 {
            Err(Error::msg("poly has interior features"))
        } else {
            let exterior = first_poly.exterior();
            if exterior.0.len() != 5 {
                Err(Error::msg("polygon is no rectangle"))
            } else {
                let c_max: Coordinate<T> = Coordinate {x: f32::MAX.into(), y: f32::MAX.into()};
                let min = exterior.0.iter().fold(c_max,|a: Coordinate<T>, b: &Coordinate<T>| {
                    Coordinate {x: a.x.min(b.x), y: a.y.min(b.y) }
                });
                let c_min: Coordinate<T> = Coordinate {x: (-99999999999.9).into(), y: (-999999999.9).into()};
                let max = exterior.0.iter().fold(c_min,|a: Coordinate<T>, b: &Coordinate<T>| {
                    Coordinate { x: a.x.max(b.x), y: a.y.max(b.y) }
                });

                Ok(Rect::new(min, max))
            }
        }
    }

    fn box_0_0_to_5_10() -> Rect<f64> {
        Rect::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 10.0},
        )
    }

    #[test]
    fn clip_point_returns_none_if_point_outside_of_box() {
        let rect = box_0_0_to_5_10();
        let point = geo::Geometry::Point(Point(Coordinate {x: 6.0, y: 5.0}));

        let clipped  = point.clip(&rect);

        assert!(clipped.is_none());
    }

    #[rstest]
    #[case(0.0, 0.0)]
    #[case(1.0, 5.0)]
    #[case(5.0, 10.0)]
    fn clip_point_returns_point_if_point_inside_of_box(#[case] x: f64, #[case] y: f64) {
        let rect = box_0_0_to_5_10();
        let point = geo::Geometry::Point(Point(Coordinate {x, y}));

        let clipped = point.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), point);
    }

    #[test]
    fn clip_line_returns_none_if_linestring_outside_of_box() {
        let rect = box_0_0_to_5_10();

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 4.0, y: -3.0},
            Coordinate {x: 8.0, y: 5.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_none());
    }

    #[test]
    fn clip_line_returns_complete_line_if_line_inside_of_box() {
        let rect = box_0_0_to_5_10();

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 1.0, y: 1.0},
            Coordinate {x: 3.0, y: 3.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), line);
    }

    #[test]
    fn clip_line_returns_complete_line_if_line_on_edge_of_box() {
        let rect = box_0_0_to_5_10();

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 0.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), line);
    }

    #[test]
    fn clip_line_returns_clipped_line_if_line_passes_through_box() {
        let rect = box_0_0_to_5_10();

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: -2.5, y: 0.0},
            Coordinate {x: 7.5, y: 10.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        if let Some(geo::Geometry::Line(clipped_line)) = clipped {
            assert_eq!(clipped_line.start, Coordinate {x: 0.0, y: 2.5});
            assert_eq!(clipped_line.end, Coordinate {x: 5.0, y: 7.5});
        } else {
            assert!(false, "no line!");
        }

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 7.5, y: 10.0},
            Coordinate {x: -2.5, y: 0.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        if let Some(geo::Geometry::Line(clipped_line)) = clipped {
            assert_eq!(clipped_line.start, Coordinate {x: 5.0, y: 7.5});
            assert_eq!(clipped_line.end, Coordinate {x: 0.0, y: 2.5});
        } else {
            assert!(false, "no line!");
        }
    }

    #[test]
    fn clip_line_returns_clipped_line_if_line_starts_within_but_goes_outside_the_box() {
        let rect = box_0_0_to_5_10();

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 2.5, y: 5.0},
            Coordinate {x: 7.5, y: 10.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        if let Some(geo::Geometry::Line(clipped_line)) = clipped {
            assert_eq!(clipped_line.start, Coordinate {x: 2.5, y: 5.0});
            assert_eq!(clipped_line.end, Coordinate {x: 5.0, y: 7.5});
        } else {
            assert!(false, "no line!");
        }

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 7.5, y: 10.0},
            Coordinate {x: 2.5, y: 5.0},

        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        if let Some(geo::Geometry::Line(clipped_line)) = clipped {
            assert_eq!(clipped_line.end, Coordinate {x: 2.5, y: 5.0});
            assert_eq!(clipped_line.start, Coordinate {x: 5.0, y: 7.5});
        } else {
            assert!(false, "no line!");
        }
    }


    #[test]
    fn clip_polygon_that_surrounds_box_will_return_box() {
        let rect = box_0_0_to_5_10();

        let polygon = geo::Geometry::Polygon(polygon_contains_the_box());

        let clipped = polygon.clip(&rect);

        assert!(clipped.is_some());
        let clipped_rect = match clipped {
            Some(geo::Geometry::MultiPolygon(v)) => multipoly_to_rect(&v, 0),
            _ => Err(Error::msg("mee"))
        };
        assert_eq!(rect, clipped_rect.unwrap());
        // assert_eq!(clipped.unwrap(), geo::Geometry::MultiPolygon(geo::MultiPolygon(vec![rect.to_polygon()])));
    }

    #[rstest]
    #[case (-10.0, -10.0)]
    #[case (10.0, 10.0)]
    #[case (0.0, 10.0)]
    #[case (10.0, 0.0)]
    fn clip_polygon_thats_outside_the_box_will_return_none(#[case] offset_x: f64, #[case] offset_y: f64) {
        let rect = box_0_0_to_5_10();
        // somewhat irregular pentagon
        let polygon = geo::Geometry::Polygon(small_polygon());
        let polygon_trans = polygon.translate(offset_x, offset_y);

        let clipped = polygon_trans.clip(&rect);

        assert!(clipped.is_none());
    }

    fn polygon_contains_the_box() -> geo::Polygon<f64> {
        Polygon::new(
            LineString(vec![
                Coordinate {x: -1.0, y: -1.0},
                Coordinate {x: -1.0, y: 11.0},
                Coordinate {x: 6.0, y: 11.0},
                Coordinate {x: 6.0, y: -1.0},
            ]),
            vec![],
        )
    }

    fn small_polygon() -> geo::Polygon<f64> {
        Polygon::new(
            LineString(vec![
                Coordinate {x: 1.0, y: 1.0},
                Coordinate {x: 2.0, y: 1.0},
                Coordinate {x: 3.0, y: 2.0},
                Coordinate {x: 1.5, y: 3.0},
                Coordinate {x: 0.0, y: 2.0},
            ]),
            vec![],
        )
    }

    fn polygon_partially_in_box() -> geo::Polygon<f64> {
        Polygon::new(
            LineString(vec![
                Coordinate {x: -5.0, y: -5.0},
                Coordinate {x: 2.5, y: -5.0},
                Coordinate {x: 2.5, y: 20.0},
                Coordinate {x: -5.0, y: 20.0},
                Coordinate {x: -5.0, y: -5.0},
            ]),
            vec![],
        )
    }

    fn polygon_partially_in_box_clipped() -> geo::Rect<f64> {
        Rect::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 2.5, y: 10.0},
        )
    }

    #[test]
    fn clip_polygon_thats_partially_in_the_box_will_return_clipped() {
        let rect = box_0_0_to_5_10();
        // somewhat irregular pentagon
        let polygon = geo::Geometry::Polygon(polygon_partially_in_box());

        let clipped = polygon.clip(&rect);

        assert!(clipped.is_some());
        let expected = polygon_partially_in_box_clipped();
        let clipped_rect = match clipped {
            Some(geo::Geometry::MultiPolygon(v)) => multipoly_to_rect(&v, 0),
            _ => Err(Error::msg("mee"))
        };
        assert_eq!(expected, clipped_rect.unwrap());
    }

    #[test]
    fn clip_multipolygon_clips_them_all() {
        let rect = box_0_0_to_5_10();
        // somewhat irregular pentagon
        let polygon = geo::Geometry::MultiPolygon(MultiPolygon(vec![
            small_polygon().translate(25.0, 25.0),
            polygon_partially_in_box(),
            polygon_contains_the_box(),
        ]));

        let clipped = polygon.clip(&rect);

        assert!(clipped.is_some());
        match clipped {
            Some(geo::Geometry::MultiPolygon(mpg)) => {
                assert_eq!(2, mpg.0.len());
                assert_eq!(multipoly_to_rect(&mpg, 0).unwrap(), polygon_partially_in_box_clipped());
                assert_eq!(multipoly_to_rect(&mpg, 1).unwrap(), rect);
            },
            _ => assert!(false)
        }
    }
}

pub trait FromF64 {
    fn fromf64(x: &f64) -> Self;
}
impl FromF64 for f32 {
    fn fromf64(x: &f64) -> Self {
        match x.to_f32() {
            Some(f) => f,
            None => f32::MAX
        }
    }
}
impl FromF64 for f64 {
    fn fromf64(x: &f64) -> Self {
        x.clone()
    }
}

pub trait ClipFloat: GeoFloat + std::convert::From<f32> + FromF64 {}
impl<T> ClipFloat for T where T: GeoFloat + std::convert::From<f32> + FromF64 {

}

/// it would be neat to generalize this to a Diff trait (subtract one geometry from another!)
/// but that would be overkill much. do not need.
pub trait Clip<T: ClipFloat, Rhs=Self> {
    type Output;
    fn clip(&self, rect: &Rect<T>) ->  Option<Self::Output>;
}

impl<T: ClipFloat> Clip<T> for Geometry<T> {
    type Output=Geometry<T>;
    fn clip(&self, rect: &Rect<T>) -> Option<Geometry<T>> {
        match self {
            Geometry::Point(pt) => pt.clip(rect).map(|p| {Geometry::Point(p)}),
            Geometry::Line(l) => l.clip(rect).map(|l| {Geometry::Line(l)}),
            Geometry::Polygon(pg) => pg.clip(rect).map(|pg| {Geometry::MultiPolygon(pg)}),
            Geometry::MultiPolygon(mpg) => {
                let polys = mpg
                    .iter()
                    .filter_map(|pg| { pg.clip(rect)});

                Some(Geometry::MultiPolygon(polys.flatten().collect()))
            },
            _ => None,
        }
    }
}

fn contains<T: ClipFloat>(rect: &Rect<T>, coord: &Coordinate<T>) -> bool {
    coord.x >= rect.min().x
        && coord.x <= rect.max().x
        && coord.y >= rect.min().y
        && coord.y <= rect.max().y
}

impl<T: ClipFloat> Clip<T> for Line<T> {
    type Output = Line<T>;

    fn clip(&self, rect: &Rect<T>) -> Option<Self::Output> {
        let box_lines = rect.to_polygon().exterior().lines().collect::<Vec<Line<T>>>();
        let intersections = box_lines.into_iter().filter_map(|box_line| {
            line_intersection(box_line.clone(), self.clone())
        }).collect::<Vec<LineIntersection<T>>>();

        let start_contained = contains(rect, &self.start);
        let end_contained = contains(rect, &self.end);

        let (collinears, single_points): (Vec<LineIntersection<T>>,Vec<LineIntersection<T>>) = intersections.into_iter().partition(|i| {
            match i {
                LineIntersection::Collinear {intersection: _} => true,
                LineIntersection::SinglePoint {intersection: _, is_proper: _ } => false,
            }
        });
        if let Some(LineIntersection::Collinear {intersection}) = collinears.first() {
            Some(intersection.clone())
        } else {
            let single_point_coordinates: Vec<Coordinate<T>> = single_points.into_iter().filter_map(|sp| {match sp {
                LineIntersection::SinglePoint {intersection, is_proper: _} => Some(intersection.clone()),
                _ => None
            }}).collect();

            match (single_point_coordinates.get(0), single_point_coordinates.get(1)) {
                (None, None) => {
                    if start_contained && end_contained {
                        Some(self.clone())
                    } else {
                        None
                    }
                },
                (Some(intersection), None) => {
                    if start_contained {
                        Some(Line::new(self.start.clone(), intersection.clone()))
                    } else {
                        Some(Line::new(intersection.clone(), self.end.clone()))
                    }

                },
                (Some(i1), Some(i2)) => {
                    if i1.euclidean_distance(&self.start) < i2.euclidean_distance(&self.start) {
                        Some(Line::new(i1.clone(), i2.clone()))
                    } else {
                        Some(Line::new(i2.clone(), i1.clone()))
                    }
                }
                (None, Some(_)) => { panic!("this should never happen")}
            }
        }
    }
}

impl<T: ClipFloat> Clip<T> for Point<T> {
    type Output = Point<T>;

    fn clip(&self, rect: &Rect<T>) -> Option<Self::Output> {
        let (px, py) = self.x_y();
        let (rx1, ry1) = rect.min().x_y();
        let (rx2, ry2) = rect.max().x_y();

        if (px >= rx1) && (px <= rx2) && (py >= ry1) && (py <= ry2) {
            Some(self.clone())
        } else {
            None
        }
    }
}

impl<T: ClipFloat> Clip<T> for Polygon<T> {
    type Output = MultiPolygon<T>;

    fn clip(&self, rect: &Rect<T>) -> Option<Self::Output> {
        let map_f64 = |(a, b): &(T, T)| {(a.to_f64().unwrap(), b.to_f64().unwrap())};
        let rect_poly = rect.to_polygon().map_coords(map_f64);
        let clipped = rect_poly.intersection(&self.map_coords(map_f64), 100.0);
        if clipped.0.is_empty() {
            None
        } else {
            Some(clipped.map_coords(|(a, b)| {
                (T::fromf64(a), T::fromf64(b))
            }))
        }
    }
}