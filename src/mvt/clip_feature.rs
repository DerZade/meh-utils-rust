use geo::{Coordinate, GeoFloat, Geometry, Line, Point, Polygon, Rect};
use geo::algorithm::line_intersection::{line_intersection, LineIntersection};
use geo::algorithm::euclidean_distance::EuclideanDistance;

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use geo::{Coordinate, Line, LineString, Point, Polygon, Rect};
    use crate::mvt::clip_feature::Clip;

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

        let polygon = geo::Geometry::Polygon(Polygon::new(
            LineString(vec![
                Coordinate {x: -1.0, y: -1.0},
                Coordinate {x: -1.0, y: 11.0},
                Coordinate {x: 6.0, y: 11.0},
                Coordinate {x: 6.0, y: -1.0},
            ]),
            vec![],
        ));

        let clipped = polygon.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(),geo::Geometry::Polygon(rect.to_polygon()));
    }
}

// it would be neat to generalize this to a Diff trait (subtract one geometry from another!)
// but that would be overkill much. do not need.
pub trait Clip<T: GeoFloat, Rhs=Self> {
    type Output;
    fn clip(&self, rect: &Rect<T>) ->  Option<Self::Output>;
}

impl<T: GeoFloat> Clip<T> for Geometry<T> {
    type Output=Geometry<T>;
    fn clip(&self, rect: &Rect<T>) -> Option<Geometry<T>> {
        match self {
            Geometry::Point(pt) => pt.clip(rect).map(|p| {Geometry::Point(p)}),
            Geometry::Line(l) => l.clip(rect).map(|l| {Geometry::Line(l)}),
            Geometry::Polygon(pg) => pg.clip(rect).map(|pg| {Geometry::Polygon(pg)}),
            _ => None,
        }
    }
}

fn contains<T: GeoFloat>(rect: &Rect<T>, coord: &Coordinate<T>) -> bool {
    coord.x >= rect.min().x
        && coord.x <= rect.max().x
        && coord.y >= rect.min().y
        && coord.y <= rect.max().y
}

impl<T: GeoFloat> Clip<T> for Line<T> {
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

impl<T: GeoFloat> Clip<T> for Point<T> {
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

impl<T: GeoFloat> Clip<T> for Polygon<T> {
    type Output = Polygon<T>;

    fn clip(&self, rect: &Rect<T>) -> Option<Self::Output> {
        Some(rect.clone().to_polygon())
    }
}