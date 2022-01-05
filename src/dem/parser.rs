use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{line_ending, space0, space1, u32 as u32_parser},
    combinator::map,
    error::ParseError,
    multi::separated_list0,
    number::complete::float,
    sequence::{preceded, separated_pair, terminated},
    AsChar, Compare, IResult, InputLength, InputTake, InputTakeAtPosition, Parser,
};

use super::{raster::Origin, DEMRaster};

// #[derive(Debug, PartialEq)]
#[derive(thiserror::Error, Debug)]
pub enum DEMParserError {
    #[error("Missing NCOLS-Header")]
    MissingNColsHeader,

    #[error("Missing NROWS-Header")]
    MissingNRowsHeader,

    #[error("Missing CELLSIZE-Header")]
    MissingCellSizeHeader,

    #[error("Expected either XLLCENTER- & YLLCENTER-Header or XLLCORNER- & YLLCORNER-Header")]
    MissingOrigin,

    #[error("Row {} is too short", .0)]
    RowTooShort(usize),

    #[error("One or more rows are missing")]
    MissingRow,

    // CELLSIZE-Header is smaller or equals than Zero
    #[error("CELLSIZE-Header is >= 0")]
    CellSizeInvalid,

    #[error("NOM returned an incomplete-error")]
    NomIncomplete,

    #[error("NOM returned an error: {}", .0.description())]
    Nom(nom::error::ErrorKind),
}

impl<I> ParseError<I> for DEMParserError {
    fn from_error_kind(_: I, kind: nom::error::ErrorKind) -> Self {
        DEMParserError::Nom(kind)
    }

    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl Into<nom::Err<DEMParserError>> for DEMParserError {
    fn into(self) -> nom::Err<DEMParserError> {
        nom::Err::Failure(self)
    }
}

impl From<nom::Err<DEMParserError>> for DEMParserError {
    fn from(e: nom::Err<DEMParserError>) -> Self {
        match e {
            nom::Err::Incomplete(_) => Self::NomIncomplete,
            nom::Err::Error(dem_err) => dem_err,
            nom::Err::Failure(dem_err) => dem_err,
        }
    }
}

#[derive(Debug)]
enum DEMHeader {
    NCols(usize),
    NRows(usize),
    XLLCenter(f32),
    XLLCorner(f32),
    YLLCenter(f32),
    YLLCorner(f32),
    CellSize(f32),
    NoDataValue(f32),
}

#[derive(Debug)]
pub struct DEMParser {}

impl DEMParser {
    // For clarification: I don't know what the fuck I'm doing here 😅
    // https://i.kym-cdn.com/photos/images/original/000/234/765/b7e.jpg
    fn header_line_factory<T, I, P, O>(
        name: T,
        parser: P,
    ) -> impl FnMut(I) -> IResult<I, (I, O), DEMParserError>
    where
        T: InputLength + Copy,
        P: Parser<I, O, DEMParserError> + Copy,
        I: InputTake + Compare<T> + InputTakeAtPosition,
        I: Compare<&'static str>
            + nom::InputLength
            + nom::InputIter
            + nom::Slice<std::ops::RangeTo<usize>>
            + nom::Slice<std::ops::RangeFrom<usize>>
            + nom::Slice<std::ops::Range<usize>>,
        <I as InputTakeAtPosition>::Item: AsChar + Clone,
    {
        move |input: I| {
            terminated::<_, _, _, DEMParserError, _, _>(
                separated_pair(tag_no_case(name), space1, parser),
                line_ending,
            )(input)
        }
    }

    fn ncols_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("NCOLS", u32_parser),
            |(_, val)| DEMHeader::NCols(val as usize),
        )(input)
    }

    fn nrows_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("NROWS", u32_parser),
            |(_, val)| DEMHeader::NRows(val as usize),
        )(input)
    }

    fn x_center_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("XLLCENTER", float),
            |(_, val)| DEMHeader::XLLCenter(val),
        )(input)
    }

    fn x_corner_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("XLLCORNER", float),
            |(_, val)| DEMHeader::XLLCorner(val),
        )(input)
    }

    fn y_center_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("YLLCENTER", float),
            |(_, val)| DEMHeader::YLLCenter(val),
        )(input)
    }

    fn y_corner_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("YLLCORNER", float),
            |(_, val)| DEMHeader::YLLCorner(val),
        )(input)
    }

    fn no_data_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("NODATA_VALUE", float),
            |(_, val)| DEMHeader::NoDataValue(val),
        )(input)
    }

    fn cell_size_header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        map(
            DEMParser::header_line_factory("CELLSIZE", float),
            |(_, val)| DEMHeader::CellSize(val),
        )(input)
    }

    fn header_line(input: &str) -> IResult<&str, DEMHeader, DEMParserError> {
        alt((
            DEMParser::ncols_header_line,
            DEMParser::ncols_header_line,
            DEMParser::nrows_header_line,
            DEMParser::x_center_header_line,
            DEMParser::x_corner_header_line,
            DEMParser::y_center_header_line,
            DEMParser::y_corner_header_line,
            DEMParser::cell_size_header_line,
            DEMParser::no_data_header_line,
        ))(input)
    }

    fn data_line(input: &str) -> IResult<&str, Vec<f32>, DEMParserError> {
        terminated(
            separated_list0(space1, float),
            preceded(space0, line_ending),
        )(input)
    }

    fn header(mut input: &str) -> IResult<&str, (usize, usize, Origin, f32, f32), DEMParserError> {
        let mut columns: Option<usize> = None;
        let mut rows: Option<usize> = None;
        let mut x_center: Option<f32> = None;
        let mut y_center: Option<f32> = None;
        let mut x_corner: Option<f32> = None;
        let mut y_corner: Option<f32> = None;
        let mut cell_size: Option<f32> = None;
        let mut no_data_value: Option<f32> = None;

        loop {
            match DEMParser::header_line(input) {
                Err(nom::Err::Error(_)) => break, // Normal Error: Maybe this was the last header line?
                Err(err) => return Err(err),
                Ok((remaining_input, header)) => {
                    input = remaining_input;

                    match header {
                        DEMHeader::NCols(val) => {
                            columns = Some(val);
                        }
                        DEMHeader::NRows(val) => {
                            rows = Some(val);
                        }
                        DEMHeader::XLLCenter(val) => {
                            x_center = Some(val);
                        }
                        DEMHeader::XLLCorner(val) => {
                            x_corner = Some(val);
                        }
                        DEMHeader::YLLCenter(val) => {
                            y_center = Some(val);
                        }
                        DEMHeader::YLLCorner(val) => {
                            y_corner = Some(val);
                        }
                        DEMHeader::CellSize(val) => {
                            cell_size = Some(val);
                        }
                        DEMHeader::NoDataValue(val) => {
                            no_data_value = Some(val);
                        }
                    }
                }
            }
        }

        if columns.is_none() {
            return Err(DEMParserError::MissingNColsHeader.into());
        }

        if rows.is_none() {
            return Err(DEMParserError::MissingNRowsHeader.into());
        }

        if cell_size.is_none() {
            return Err(DEMParserError::MissingCellSizeHeader.into());
        }

        if cell_size.unwrap() <= 0.0 {
            return Err(DEMParserError::CellSizeInvalid.into());
        }

        if (x_center.is_none() || y_center.is_none()) && (x_corner.is_none() || y_corner.is_none())
        {
            return Err(DEMParserError::MissingOrigin.into());
        }

        let origin = if x_center.is_some() && y_center.is_some() {
            Origin::Center(x_center.unwrap(), y_center.unwrap())
        } else {
            Origin::Corner(x_corner.unwrap(), y_corner.unwrap())
        };

        Ok((
            input,
            (
                columns.unwrap(),
                rows.unwrap(),
                origin,
                cell_size.unwrap(),
                no_data_value.unwrap_or(-9999.0),
            ),
        ))
    }

    pub fn parse(i: &str) -> Result<DEMRaster, DEMParserError> {
        let mut input = i;
        let (remaining_input, (columns, rows, origin, cell_size, no_data_value)) =
            DEMParser::header(input)?;
        input = remaining_input;

        let mut data: Vec<f32> = Vec::with_capacity(columns * rows);

        for row_index in 0..rows {
            if input.len() == 0 {
                return Err(DEMParserError::MissingRow.into());
            }

            let (remaining_input, ref mut vec) = DEMParser::data_line(input)?;
            input = remaining_input;

            if vec.len() < columns {
                return Err(DEMParserError::RowTooShort(row_index).into());
            }

            if vec.len() > columns {
                vec.drain(columns..);
            }

            data.append(vec);
        }

        Ok(DEMRaster::new(
            columns,
            rows,
            origin,
            cell_size,
            no_data_value,
            data,
        ))
    }
}
