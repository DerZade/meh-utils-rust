use geo::contains::Contains;
use geo::{CoordFloat, Coordinate, GeoFloat, Geometry, GeoNum, Line, Point, Rect};
use geo::algorithm::line_intersection::{line_intersection, LineIntersection};
use geo::map_coords::MapCoords;

#[cfg(test)]
mod tests {
    use geo::{Coordinate, Geometry, Line, Point, Rect};
    use crate::mvt::clip_feature::Clip;

    #[test]
    fn clip_point_returns_none_if_point_outside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 10.0},
        );
        let point = geo::Geometry::Point(Point(Coordinate {x: 6.0, y: 5.0}));

        let clipped  = point.clip(&rect);

        assert!(clipped.is_none());
    }

    #[test]
    fn clip_point_returns_point_if_point_inside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 10.0},
        );
        let point = geo::Geometry::Point(Point(Coordinate {x: 1.0, y: 5.0}));

        let clipped = point.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), point);
    }

    #[test]
    fn clip_line_returns_none_if_linestring_outside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 10.0},
        );

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 4.0, y: -3.0},
            Coordinate {x: 8.0, y: 5.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_none());
    }

    #[test]
    fn clip_line_returns_complete_line_if_line_inside_of_box() {
        let rect = Rect::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 10.0},
        );

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
        let rect = Rect::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 10.0},
        );

        let line = geo::Geometry::Line(Line::new(
            Coordinate {x: 0.0, y: 0.0},
            Coordinate {x: 5.0, y: 0.0},
        ));

        let clipped = line.clip(&rect);

        assert!(clipped.is_some());
        assert_eq!(clipped.unwrap(), line);
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
        if contains(rect, &self.start) && contains(rect, &self.end) {
            Some(self.clone())
        } else {
            let box_lines = rect.to_polygon().exterior().lines().collect::<Vec<Line<T>>>();
             box_lines.into_iter().for_each(|box_line| {
                let intersection = line_intersection(box_line.clone(), self.clone());

            });
            // TODO
            None
        }
    }
}

impl<T: GeoFloat> Clip<T> for Point<T> {
    type Output = Point<T>;

    fn clip(&self, rect: &Rect<T>) -> Option<Self::Output> {
        rect.contains(self).then(|| {self.clone()})
    }
}